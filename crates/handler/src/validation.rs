use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, Cfg, ContextTr,
};
use core::cmp;
use interpreter::gas::{self, InitialAndFloorGas};
use primitives::{eip4844, hardfork::SpecId, B256};

/// Validates the execution environment including block and transaction parameters.
pub fn validate_env<CTX: ContextTr, ERROR: From<InvalidHeader> + From<InvalidTransaction>>(
    context: CTX,
) -> Result<(), ERROR> {
    let spec = context.cfg().spec().into();
    // `prevrandao` is required for the merge
    if spec.is_enabled_in(SpecId::MERGE) && context.block().prevrandao().is_none() {
        return Err(InvalidHeader::PrevrandaoNotSet.into());
    }
    // `excess_blob_gas` is required for Cancun
    if spec.is_enabled_in(SpecId::CANCUN) && context.block().blob_excess_gas_and_price().is_none() {
        return Err(InvalidHeader::ExcessBlobGasNotSet.into());
    }
    validate_tx_env::<CTX>(context, spec).map_err(Into::into)
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

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<u128>,
    disable_priority_fee_check: bool,
) -> Result<(), InvalidTransaction> {
    if !disable_priority_fee_check && max_priority_fee > max_fee {
        // Or gas_max_fee for eip1559
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    // Check minimal cost against basefee
    if let Some(base_fee) = base_fee {
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

/// Validate transaction against block and configuration for mainnet.
pub fn validate_tx_env<CTX: ContextTr>(
    context: CTX,
    spec_id: SpecId,
) -> Result<(), InvalidTransaction> {
    // Check if the transaction's chain id is correct
    let tx_type = context.tx().tx_type();
    let tx = context.tx();

    let base_fee = if context.cfg().is_base_fee_check_disabled() {
        None
    } else {
        Some(context.block().basefee() as u128)
    };

    let tx_type = TransactionType::from(tx_type);

    // Check chain_id if config is enabled.
    // EIP-155: Simple replay attack protection
    if context.cfg().tx_chain_id_check() {
        if let Some(chain_id) = tx.chain_id() {
            if chain_id != context.cfg().chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }
        } else if !tx_type.is_legacy() && !tx_type.is_custom() {
            // Legacy transaction are the only one that can omit chain_id.
            return Err(InvalidTransaction::MissingChainId);
        }
    }

    // EIP-7825: Transaction Gas Limit Cap
    let cap = context.cfg().tx_gas_limit_cap();
    if tx.gas_limit() > cap {
        return Err(InvalidTransaction::TxGasLimitGreaterThanCap {
            gas_limit: tx.gas_limit(),
            cap,
        });
    }

    let disable_priority_fee_check = context.cfg().is_priority_fee_check_disabled();

    match tx_type {
        TransactionType::Legacy => {
            validate_legacy_gas_price(tx.gas_price(), base_fee)?;
        }
        TransactionType::Eip2930 => {
            // Enabled in BERLIN hardfork
            if !spec_id.is_enabled_in(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported);
            }
            validate_legacy_gas_price(tx.gas_price(), base_fee)?;
        }
        TransactionType::Eip1559 => {
            if !spec_id.is_enabled_in(SpecId::LONDON) {
                return Err(InvalidTransaction::Eip1559NotSupported);
            }
            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
                disable_priority_fee_check,
            )?;
        }
        TransactionType::Eip4844 => {
            if !spec_id.is_enabled_in(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
                disable_priority_fee_check,
            )?;

            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                context.block().blob_gasprice().unwrap_or_default(),
                context.cfg().max_blobs_per_tx(),
            )?;
        }
        TransactionType::Eip7702 => {
            // Check if EIP-7702 transaction is enabled.
            if !spec_id.is_enabled_in(SpecId::PRAGUE) {
                return Err(InvalidTransaction::Eip7702NotSupported);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
                disable_priority_fee_check,
            )?;

            let auth_list_len = tx.authorization_list_len();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list_len == 0 {
                return Err(InvalidTransaction::EmptyAuthorizationList);
            }
        }
        TransactionType::Custom => {
            // Custom transaction type check is not done here.
        }
    };

    // Check if gas_limit is more than block_gas_limit
    if !context.cfg().is_block_gas_limit_disabled() && tx.gas_limit() > context.block().gas_limit()
    {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
    }

    // EIP-3860: Limit and meter initcode. Still valid with EIP-7907 and increase of initcode size.
    if spec_id.is_enabled_in(SpecId::SHANGHAI)
        && tx.kind().is_create()
        && context.tx().input().len() > context.cfg().max_initcode_size()
    {
        return Err(InvalidTransaction::CreateInitCodeSizeLimit);
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
    use primitives::{
        address, eip3860, eip7907, hardfork::SetSpecTr, hardfork::SpecId, Bytes, TxKind, B256,
    };
    use state::{AccountInfo, Bytecode};

    fn deploy_contract(
        bytecode: Bytes,
        spec_id: Option<SpecId>,
    ) -> Result<ExecutionResult, EVMError<core::convert::Infallible>> {
        let ctx = Context::mainnet()
            .modify_cfg_chained(|c| {
                if let Some(spec_id) = spec_id {
                    c.set_spec(spec_id);
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
