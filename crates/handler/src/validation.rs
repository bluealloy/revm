use context::transaction::tx_validation::{ValidationChecks, ValidationKind};
use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, Cfg, ContextTr,
};
use core::cmp;
use interpreter::gas::{self, InitialAndFloorGas};
use primitives::{eip4844, hardfork::SpecId, B256};

/// Parameters extracted from context for validation.
#[derive(Debug)]
pub struct ValidationParams {
    spec_id: SpecId,
    base_fee: Option<u128>,
    disable_priority_fee_check: bool,
    tx_chain_id_check: bool,
    cfg_chain_id: u64,
    tx_gas_limit_cap: u64,
    is_block_gas_limit_disabled: bool,
    block_gas_limit: u64,
    max_initcode_size: usize,
    block_blob_gas_price: u128,
    max_blobs_per_tx: Option<u64>,
}

impl ValidationParams {
    /// Creates validation parameters from context
    fn from_context<CTX: ContextTr>(context: &CTX, spec_id: SpecId) -> Self {
        let cfg = context.cfg();
        let block = context.block();

        Self {
            spec_id,
            base_fee: if cfg.is_base_fee_check_disabled() {
                None
            } else {
                Some(block.basefee() as u128)
            },
            disable_priority_fee_check: cfg.is_priority_fee_check_disabled(),
            tx_chain_id_check: cfg.tx_chain_id_check(),
            cfg_chain_id: cfg.chain_id(),
            tx_gas_limit_cap: cfg.tx_gas_limit_cap(),
            is_block_gas_limit_disabled: cfg.is_block_gas_limit_disabled(),
            block_gas_limit: block.gas_limit(),
            max_initcode_size: cfg.max_initcode_size(),
            block_blob_gas_price: block.blob_gasprice().unwrap_or_default(),
            max_blobs_per_tx: cfg.max_blobs_per_tx(),
        }
    }
}

/// Validates the execution environment including block and transaction parameters.
pub fn validate_env<CTX: ContextTr, ERROR: From<InvalidHeader> + From<InvalidTransaction>>(
    context: CTX,
) -> Result<(), ERROR> {
    let spec = context.cfg().spec().into();
    validate_block_header::<CTX, ERROR>(&context, spec)?;
    validate_tx_env::<CTX>(context, spec).map_err(Into::into)
}

/// Validate block header requirements for the spec.
fn validate_block_header<CTX: ContextTr, ERROR: From<InvalidHeader>>(
    context: &CTX,
    spec: SpecId,
) -> Result<(), ERROR> {
    if spec.is_enabled_in(SpecId::MERGE) && context.block().prevrandao().is_none() {
        return Err(InvalidHeader::PrevrandaoNotSet.into());
    }
    if spec.is_enabled_in(SpecId::CANCUN) && context.block().blob_excess_gas_and_price().is_none() {
        return Err(InvalidHeader::ExcessBlobGasNotSet.into());
    }
    Ok(())
}

/// Validate transaction against block and configuration.
pub fn validate_tx_env<CTX: ContextTr>(
    context: CTX,
    spec_id: SpecId,
) -> Result<(), InvalidTransaction> {
    let tx = context.tx();
    let tx_type = TransactionType::from(tx.tx_type());
    let params = ValidationParams::from_context(&context, spec_id);

    match tx.validation_kind() {
        ValidationKind::None => Ok(()),
        ValidationKind::ByTxType => validate_all_for_tx_type(&params, &tx_type, tx),
        ValidationKind::Custom(checks) => validate_custom(&params, &tx_type, tx, checks),
    }
}

/// Run all validations appropriate for the transaction type.
fn validate_all_for_tx_type(
    params: &ValidationParams,
    tx_type: &TransactionType,
    tx: impl Transaction,
) -> Result<(), InvalidTransaction> {
    // Check chain_id if config is enabled.
    // EIP-155: Simple replay attack protection
    validate_chain_id(params, tx.chain_id(), tx_type)?;

    // EIP-7825: Transaction Gas Limit Cap
    validate_tx_gas_limit(tx.gas_limit(), params.tx_gas_limit_cap)?;

    // Type-specific validations (Legacy, Eip2930, Eip1559, Eip4844, Eip7702, Custom)
    validate_type_specific(params, tx_type, &tx)?;

    // Post type-specific common validations
    validate_block_gas_limit(params, tx.gas_limit())?;
    validate_initcode_size(params, tx.kind().is_create(), tx.input().len())?;

    Ok(())
}

/// Type-specific validation logic.
fn validate_type_specific(
    params: &ValidationParams,
    tx_type: &TransactionType,
    tx: &impl Transaction,
) -> Result<(), InvalidTransaction> {
    match tx_type {
        TransactionType::Legacy => validate_legacy_gas_price(tx.gas_price(), params.base_fee),
        TransactionType::Eip2930 => {
            require_spec(
                params.spec_id,
                SpecId::BERLIN,
                InvalidTransaction::Eip2930NotSupported,
            )?;
            validate_legacy_gas_price(tx.gas_price(), params.base_fee)
        }
        TransactionType::Eip1559 => {
            require_spec(
                params.spec_id,
                SpecId::LONDON,
                InvalidTransaction::Eip1559NotSupported,
            )?;
            validate_priority_fee_tx(params, tx)
        }
        TransactionType::Eip4844 => {
            require_spec(
                params.spec_id,
                SpecId::CANCUN,
                InvalidTransaction::Eip4844NotSupported,
            )?;
            validate_priority_fee_tx(params, tx)?;
            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                params.block_blob_gas_price,
                params.max_blobs_per_tx,
            )
        }
        TransactionType::Eip7702 => {
            require_spec(
                params.spec_id,
                SpecId::PRAGUE,
                InvalidTransaction::Eip7702NotSupported,
            )?;
            validate_priority_fee_tx(params, tx)?;
            validate_auth_list_not_empty(tx.authorization_list_len())
        }
        TransactionType::Custom => Ok(()),
    }
}

/// Validate legacy transaction gas price against basefee.
#[inline]
pub fn validate_legacy_gas_price(
    gas_price: u128,
    base_fee: Option<u128>,
) -> Result<(), InvalidTransaction> {
    // Gas price must be at least the basefee.
    if let Some(base_fee) = base_fee {
        if gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }
    Ok(())
}

/// Validate transaction that has EIP-1559 priority fee (used by EIP-1559, EIP-4844, EIP-7702).
#[inline]
pub fn validate_priority_fee_tx(
    params: &ValidationParams,
    tx: &impl Transaction,
) -> Result<(), InvalidTransaction> {
    let max_fee = tx.max_fee_per_gas();
    let max_priority_fee = tx.max_priority_fee_per_gas().unwrap_or_default();

    if !params.disable_priority_fee_check && max_priority_fee > max_fee {
        // Or gas_max_fee for eip1559
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    // Check minimal cost against basefee
    if let Some(base_fee) = params.base_fee {
        let effective_gas_price = cmp::min(max_fee, base_fee.saturating_add(max_priority_fee));
        if effective_gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }

    Ok(())
}

/// Validate EIP-4844 transaction.
pub fn validate_eip4844_tx(
    blobs: &[B256],
    max_blob_fee: u128,
    block_blob_gas_price: u128,
    max_blobs: Option<u64>,
) -> Result<(), InvalidTransaction> {
    // Ensure that the user was willing to at least pay the current blob gasprice
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax {
            block_blob_gas_price,
            tx_max_fee_per_blob_gas: max_blob_fee,
        });
    }

    // There must be at least one blob
    if blobs.is_empty() {
        return Err(InvalidTransaction::EmptyBlobs);
    }

    // All versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
    for blob in blobs {
        if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
            return Err(InvalidTransaction::BlobVersionNotSupported);
        }
    }

    // Ensure the total blob gas spent is at most equal to the limit
    // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
    if let Some(max_blobs) = max_blobs {
        if blobs.len() > max_blobs as usize {
            return Err(InvalidTransaction::TooManyBlobs {
                have: blobs.len(),
                max: max_blobs as usize,
            });
        }
    }
    Ok(())
}

/// Check that the spec is enabled, returning the error if not.
#[inline]
fn require_spec(
    current: SpecId,
    required: SpecId,
    error: InvalidTransaction,
) -> Result<(), InvalidTransaction> {
    if !current.is_enabled_in(required) {
        return Err(error);
    }
    Ok(())
}

/// Validate custom selection of checks.
fn validate_custom(
    params: &ValidationParams,
    tx_type: &TransactionType,
    tx: impl Transaction,
    checks: ValidationChecks,
) -> Result<(), InvalidTransaction> {
    if checks.contains(ValidationChecks::CHAIN_ID) {
        validate_chain_id(params, tx.chain_id(), tx_type)?;
    }
    if checks.contains(ValidationChecks::TX_GAS_LIMIT) {
        validate_tx_gas_limit(tx.gas_limit(), params.tx_gas_limit_cap)?;
    }
    if checks.contains(ValidationChecks::BASE_FEE) {
        validate_legacy_gas_price(tx.gas_price(), params.base_fee)?;
    }
    if checks.contains(ValidationChecks::PRIORITY_FEE) {
        validate_priority_fee_tx(params, &tx)?;
    }
    if checks.contains(ValidationChecks::BLOB_FEE) {
        validate_eip4844_tx(
            tx.blob_versioned_hashes(),
            tx.max_fee_per_blob_gas(),
            params.block_blob_gas_price,
            params.max_blobs_per_tx,
        )?;
    }
    if checks.contains(ValidationChecks::AUTH_LIST) {
        validate_auth_list_not_empty(tx.authorization_list_len())?;
    }
    if checks.contains(ValidationChecks::BLOCK_GAS_LIMIT) {
        validate_block_gas_limit(params, tx.gas_limit())?;
    }
    if checks.contains(ValidationChecks::MAX_INITCODE_SIZE) {
        validate_initcode_size(params, tx.kind().is_create(), tx.input().len())?;
    }
    Ok(())
}

/// Validate chain ID matches.
fn validate_chain_id(
    params: &ValidationParams,
    tx_chain_id: Option<u64>,
    tx_type: &TransactionType,
) -> Result<(), InvalidTransaction> {
    if !params.tx_chain_id_check {
        return Ok(());
    }

    match tx_chain_id {
        Some(chain_id) if chain_id != params.cfg_chain_id => {
            Err(InvalidTransaction::InvalidChainId)
        }
        None if !tx_type.is_legacy() && !tx_type.is_custom() => {
            Err(InvalidTransaction::MissingChainId)
        }
        _ => Ok(()),
    }
}

/// Validate transaction gas limit against cap.
#[inline]
fn validate_tx_gas_limit(tx_gas_limit: u64, cap: u64) -> Result<(), InvalidTransaction> {
    if tx_gas_limit > cap {
        return Err(InvalidTransaction::TxGasLimitGreaterThanCap {
            gas_limit: tx_gas_limit,
            cap,
        });
    }
    Ok(())
}

/// Validate block gas limit.
#[inline]
fn validate_block_gas_limit(
    params: &ValidationParams,
    tx_gas_limit: u64,
) -> Result<(), InvalidTransaction> {
    if !params.is_block_gas_limit_disabled && tx_gas_limit > params.block_gas_limit {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
    }
    Ok(())
}

/// Validate initcode size for contract creation.
#[inline]
fn validate_initcode_size(
    params: &ValidationParams,
    is_create: bool,
    input_len: usize,
) -> Result<(), InvalidTransaction> {
    if params.spec_id.is_enabled_in(SpecId::SHANGHAI)
        && is_create
        && input_len > params.max_initcode_size
    {
        return Err(InvalidTransaction::CreateInitCodeSizeLimit);
    }
    Ok(())
}

/// Validate authorization list is not empty.
#[inline]
fn validate_auth_list_not_empty(len: usize) -> Result<(), InvalidTransaction> {
    if len == 0 {
        return Err(InvalidTransaction::EmptyAuthorizationList);
    }
    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas(
    tx: impl Transaction,
    spec: SpecId,
    is_eip7623_disabled: bool,
) -> Result<InitialAndFloorGas, InvalidTransaction> {
    let mut gas = gas::calculate_initial_tx_gas_for_tx(&tx, spec);

    if is_eip7623_disabled {
        gas.floor_gas = 0
    }

    // Additional check to see if limit is big enough to cover initial gas.
    if gas.initial_gas > tx.gas_limit() {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit {
            gas_limit: tx.gas_limit(),
            initial_gas: gas.initial_gas,
        });
    }

    // EIP-7623: Increase calldata cost
    // floor gas should be less than gas limit.
    if spec.is_enabled_in(SpecId::PRAGUE) && gas.floor_gas > tx.gas_limit() {
        return Err(InvalidTransaction::GasFloorMoreThanGasLimit {
            gas_floor: gas.floor_gas,
            gas_limit: tx.gas_limit(),
        });
    };

    Ok(gas)
}

#[cfg(test)]
mod tests {
    use crate::{api::ExecuteEvm, ExecuteCommitEvm, MainBuilder, MainContext};
    use bytecode::opcode;
    use context::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, Output},
        Context, ContextTr, TxEnv,
    };
    use database::{CacheDB, EmptyDB};
    use primitives::{address, eip3860, eip7907, hardfork::SpecId, Bytes, TxKind, B256};
    use state::{AccountInfo, Bytecode};

    fn deploy_contract(
        bytecode: Bytes,
        spec_id: Option<SpecId>,
    ) -> Result<ExecutionResult, EVMError<core::convert::Infallible>> {
        let ctx = Context::mainnet()
            .modify_cfg_chained(|c| {
                if let Some(spec_id) = spec_id {
                    c.spec = spec_id;
                }
            })
            .with_db(CacheDB::<EmptyDB>::default());

        let mut evm = ctx.build_mainnet();
        evm.transact_commit(
            TxEnv::builder()
                .kind(TxKind::Create)
                .data(bytecode.clone())
                .build()
                .unwrap(),
        )
    }

    #[test]
    fn test_eip3860_initcode_size_limit_failure() {
        let large_bytecode = vec![opcode::STOP; eip3860::MAX_INITCODE_SIZE + 1];
        let bytecode: Bytes = large_bytecode.into();
        let result = deploy_contract(bytecode, Some(SpecId::PRAGUE));
        assert!(matches!(
            result,
            Err(EVMError::Transaction(
                InvalidTransaction::CreateInitCodeSizeLimit
            ))
        ));
    }

    #[test]
    fn test_eip3860_initcode_size_limit_success_prague() {
        let large_bytecode = vec![opcode::STOP; eip3860::MAX_INITCODE_SIZE];
        let bytecode: Bytes = large_bytecode.into();
        let result = deploy_contract(bytecode, Some(SpecId::PRAGUE));
        assert!(matches!(result, Ok(ExecutionResult::Success { .. })));
    }

    #[test]
    fn test_eip7907_initcode_size_limit_failure_osaka() {
        let large_bytecode = vec![opcode::STOP; eip7907::MAX_INITCODE_SIZE + 1];
        let bytecode: Bytes = large_bytecode.into();
        let result = deploy_contract(bytecode, Some(SpecId::OSAKA));
        assert!(matches!(
            result,
            Err(EVMError::Transaction(
                InvalidTransaction::CreateInitCodeSizeLimit
            ))
        ));
    }

    #[test]
    fn test_eip7907_code_size_limit_failure() {
        // EIP-7907: MAX_CODE_SIZE = 0x40000
        // use the simplest method to return a contract code size greater than 0x40000
        // PUSH3 0x40001 (greater than 0x40000) - return size
        // PUSH1 0x00 - memory position 0
        // RETURN - return uninitialized memory, will be filled with 0
        let init_code = vec![
            0x62, 0x04, 0x00, 0x01, // PUSH3 0x40001 (greater than 0x40000)
            0x60, 0x00, // PUSH1 0
            0xf3, // RETURN
        ];
        let bytecode: Bytes = init_code.into();
        let result = deploy_contract(bytecode, Some(SpecId::OSAKA));
        assert!(
            matches!(
                result,
                Ok(ExecutionResult::Halt {
                    reason: HaltReason::CreateContractSizeLimit,
                    ..
                },)
            ),
            "{result:?}"
        );
    }

    #[test]
    fn test_eip170_code_size_limit_failure() {
        // use the simplest method to return a contract code size greater than 0x6000
        // PUSH3 0x6001 (greater than 0x6000) - return size
        // PUSH1 0x00 - memory position 0
        // RETURN - return uninitialized memory, will be filled with 0
        let init_code = vec![
            0x62, 0x00, 0x60, 0x01, // PUSH3 0x6001 (greater than 0x6000)
            0x60, 0x00, // PUSH1 0
            0xf3, // RETURN
        ];
        let bytecode: Bytes = init_code.into();
        let result = deploy_contract(bytecode, Some(SpecId::PRAGUE));
        assert!(
            matches!(
                result,
                Ok(ExecutionResult::Halt {
                    reason: HaltReason::CreateContractSizeLimit,
                    ..
                },)
            ),
            "{result:?}"
        );
    }

    #[test]
    fn test_eip170_code_size_limit_success() {
        // use the  simplest method to return a contract code size equal to 0x6000
        // PUSH3 0x6000 - return size
        // PUSH1 0x00 - memory position 0
        // RETURN - return uninitialized memory, will be filled with 0
        let init_code = vec![
            0x62, 0x00, 0x60, 0x00, // PUSH3 0x6000
            0x60, 0x00, // PUSH1 0
            0xf3, // RETURN
        ];
        let bytecode: Bytes = init_code.into();
        let result = deploy_contract(bytecode, None);
        assert!(matches!(result, Ok(ExecutionResult::Success { .. },)));
    }

    #[test]
    fn test_eip170_create_opcode_size_limit_failure() {
        // 1. create a "factory" contract, which will use the CREATE opcode to create another large contract
        // 2. because the sub contract exceeds the EIP-170 limit, the CREATE operation should fail

        // the bytecode of the factory contract:
        // PUSH1 0x01      - the value for MSTORE
        // PUSH1 0x00      - the memory position
        // MSTORE          - store a non-zero value at the beginning of memory

        // PUSH3 0x6001    - the return size (exceeds 0x6000)
        // PUSH1 0x00      - the memory offset
        // PUSH1 0x00      - the amount of ETH sent
        // CREATE          - create contract instruction (create contract from current memory)

        // PUSH1 0x00      - the return value storage position
        // MSTORE          - store the address returned by CREATE to the memory position 0
        // PUSH1 0x20      - the return size (32 bytes)
        // PUSH1 0x00      - the return offset
        // RETURN          - return the result

        let factory_code = vec![
            // 1. store a non-zero value at the beginning of memory
            0x60, 0x01, // PUSH1 0x01
            0x60, 0x00, // PUSH1 0x00
            0x52, // MSTORE
            // 2. prepare to create a large contract
            0x62, 0x00, 0x60, 0x01, // PUSH3 0x6001 (exceeds 0x6000)
            0x60, 0x00, // PUSH1 0x00 (the memory offset)
            0x60, 0x00, // PUSH1 0x00 (the amount of ETH sent)
            0xf0, // CREATE
            // 3. store the address returned by CREATE to the memory position 0
            0x60, 0x00, // PUSH1 0x00
            0x52, // MSTORE (store the address returned by CREATE to the memory position 0)
            // 4. return the result
            0x60, 0x20, // PUSH1 0x20 (32 bytes)
            0x60, 0x00, // PUSH1 0x00
            0xf3, // RETURN
        ];

        // deploy factory contract
        let factory_bytecode: Bytes = factory_code.into();
        let factory_result = deploy_contract(factory_bytecode, Some(SpecId::PRAGUE))
            .expect("factory contract deployment failed");

        // get factory contract address
        let factory_address = match &factory_result {
            ExecutionResult::Success {
                output: Output::Create(_, Some(addr)),
                ..
            } => *addr,
            _ => panic!("factory contract deployment failed: {factory_result:?}"),
        };

        // call factory contract to create sub contract
        let tx_caller = address!("0x0000000000000000000000000000000000100000");
        let call_result = Context::mainnet()
            .with_db(CacheDB::<EmptyDB>::default())
            .build_mainnet()
            .transact_commit(
                TxEnv::builder()
                    .caller(tx_caller)
                    .kind(TxKind::Call(factory_address))
                    .data(Bytes::new())
                    .build()
                    .unwrap(),
            )
            .expect("call factory contract failed");

        match &call_result {
            ExecutionResult::Success { output, .. } => match output {
                Output::Call(bytes) => {
                    if !bytes.is_empty() {
                        assert!(
                            bytes.iter().all(|&b| b == 0),
                            "When CREATE operation failed, it should return all zero address"
                        );
                    }
                }
                _ => panic!("unexpected output type"),
            },
            _ => panic!("execution result is not Success"),
        }
    }

    #[test]
    fn test_eip170_create_opcode_size_limit_success() {
        // 1. create a "factory" contract, which will use the CREATE opcode to create another contract
        // 2. the sub contract generated by the factory contract does not exceed the EIP-170 limit, so it should be created successfully

        // the bytecode of the factory contract:
        // PUSH1 0x01      - the value for MSTORE
        // PUSH1 0x00      - the memory position
        // MSTORE          - store a non-zero value at the beginning of memory

        // PUSH3 0x6000    - the return size (0x6000)
        // PUSH1 0x00      - the memory offset
        // PUSH1 0x00      - the amount of ETH sent
        // CREATE          - create contract instruction (create contract from current memory)

        // PUSH1 0x00      - the return value storage position
        // MSTORE          - store the address returned by CREATE to the memory position 0
        // PUSH1 0x20      - the return size (32 bytes)
        // PUSH1 0x00      - the return offset
        // RETURN          - return the result

        let factory_code = vec![
            // 1. store a non-zero value at the beginning of memory
            0x60, 0x01, // PUSH1 0x01
            0x60, 0x00, // PUSH1 0x00
            0x52, // MSTORE
            // 2. prepare to create a contract
            0x62, 0x00, 0x60, 0x00, // PUSH3 0x6000 (0x6000)
            0x60, 0x00, // PUSH1 0x00 (the memory offset)
            0x60, 0x00, // PUSH1 0x00 (the amount of ETH sent)
            0xf0, // CREATE
            // 3. store the address returned by CREATE to the memory position 0
            0x60, 0x00, // PUSH1 0x00
            0x52, // MSTORE (store the address returned by CREATE to the memory position 0)
            // 4. return the result
            0x60, 0x20, // PUSH1 0x20 (32 bytes)
            0x60, 0x00, // PUSH1 0x00
            0xf3, // RETURN
        ];

        // deploy factory contract
        let factory_bytecode: Bytes = factory_code.into();
        let factory_result = deploy_contract(factory_bytecode, Some(SpecId::PRAGUE))
            .expect("factory contract deployment failed");
        // get factory contract address
        let factory_address = match &factory_result {
            ExecutionResult::Success {
                output: Output::Create(_, Some(addr)),
                ..
            } => *addr,
            _ => panic!("factory contract deployment failed: {factory_result:?}"),
        };

        // call factory contract to create sub contract
        let tx_caller = address!("0x0000000000000000000000000000000000100000");
        let call_result = Context::mainnet()
            .with_db(CacheDB::<EmptyDB>::default())
            .build_mainnet()
            .transact_commit(
                TxEnv::builder()
                    .caller(tx_caller)
                    .kind(TxKind::Call(factory_address))
                    .data(Bytes::new())
                    .build()
                    .unwrap(),
            )
            .expect("call factory contract failed");

        match &call_result {
            ExecutionResult::Success { output, .. } => {
                match output {
                    Output::Call(bytes) => {
                        // check if CREATE operation is successful (return non-zero address)
                        if !bytes.is_empty() {
                            assert!(bytes.iter().any(|&b| b != 0), "create sub contract failed");
                        }
                    }
                    _ => panic!("unexpected output type"),
                }
            }
            _ => panic!("execution result is not Success"),
        }
    }

    #[test]
    fn test_transact_many_with_transaction_index_error() {
        use context::result::TransactionIndexedError;

        let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = ctx.build_mainnet();

        // Create a transaction that will fail (invalid gas limit)
        let invalid_tx = TxEnv::builder()
            .gas_limit(0) // This will cause a validation error
            .build()
            .unwrap();

        // Create a valid transaction
        let valid_tx = TxEnv::builder().gas_limit(100000).build().unwrap();

        // Test that the first transaction fails with index 0
        let result = evm.transact_many([invalid_tx.clone()].into_iter());
        assert!(matches!(
            result,
            Err(TransactionIndexedError {
                transaction_index: 0,
                ..
            })
        ));

        // Test that the second transaction fails with index 1
        let result = evm.transact_many([valid_tx, invalid_tx].into_iter());
        assert!(matches!(
            result,
            Err(TransactionIndexedError {
                transaction_index: 1,
                ..
            })
        ));
    }

    #[test]
    fn test_transact_many_success() {
        use primitives::{address, U256};

        let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = ctx.build_mainnet();

        // Add balance to the caller account
        let caller = address!("0x0000000000000000000000000000000000000001");
        evm.db_mut().insert_account_info(
            caller,
            AccountInfo::new(
                U256::from(1000000000000000000u64),
                0,
                B256::ZERO,
                Bytecode::new(),
            ),
        );

        // Create valid transactions with proper data
        let tx1 = TxEnv::builder()
            .caller(caller)
            .gas_limit(100000)
            .gas_price(20_000_000_000u128)
            .nonce(0)
            .build()
            .unwrap();

        let tx2 = TxEnv::builder()
            .caller(caller)
            .gas_limit(100000)
            .gas_price(20_000_000_000u128)
            .nonce(1)
            .build()
            .unwrap();

        // Test that all transactions succeed
        let result = evm.transact_many([tx1, tx2].into_iter());
        if let Err(e) = &result {
            println!("Error: {e:?}");
        }
        let outputs = result.expect("All transactions should succeed");
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_transact_many_finalize_with_error() {
        use context::result::TransactionIndexedError;

        let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = ctx.build_mainnet();

        // Create transactions where the second one fails
        let valid_tx = TxEnv::builder().gas_limit(100000).build().unwrap();

        let invalid_tx = TxEnv::builder()
            .gas_limit(0) // This will cause a validation error
            .build()
            .unwrap();

        // Test that transact_many_finalize returns the error with correct index
        let result = evm.transact_many_finalize([valid_tx, invalid_tx].into_iter());
        assert!(matches!(
            result,
            Err(TransactionIndexedError {
                transaction_index: 1,
                ..
            })
        ));
    }

    #[test]
    fn test_transact_many_commit_with_error() {
        use context::result::TransactionIndexedError;

        let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());
        let mut evm = ctx.build_mainnet();

        // Create transactions where the first one fails
        let invalid_tx = TxEnv::builder()
            .gas_limit(0) // This will cause a validation error
            .build()
            .unwrap();

        let valid_tx = TxEnv::builder().gas_limit(100000).build().unwrap();

        // Test that transact_many_commit returns the error with correct index
        let result = evm.transact_many_commit([invalid_tx, valid_tx].into_iter());
        assert!(matches!(
            result,
            Err(TransactionIndexedError {
                transaction_index: 0,
                ..
            })
        ));
    }
}
