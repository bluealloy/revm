use crate::tx_validation::{self, validate_block_header, validate_tx, ValidationParams};
use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::Transaction,
    ContextTr,
};
use interpreter::InitialAndFloorGas;
use primitives::{hardfork::SpecId, B256};

/// Validates the execution environment including block and transaction parameters.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
pub fn validate_env<CTX: ContextTr, ERROR: From<InvalidHeader> + From<InvalidTransaction>>(
    context: CTX,
) -> Result<(), ERROR> {
    let params = ValidationParams::from_cfg_and_block(context.cfg(), context.block());

    validate_block_header(params.spec, context.block(), params.validation_kind)?;
    validate_tx(context.tx(), &params).map_err(Into::into)
}

/// Validate legacy transaction gas price against basefee.
#[inline]
pub fn validate_legacy_gas_price(
    gas_price: u128,
    base_fee: Option<u128>,
) -> Result<(), InvalidTransaction> {
    tx_validation::validate_legacy_gas_price(gas_price, base_fee)
}

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<u128>,
    disable_priority_fee_check: bool,
) -> Result<(), InvalidTransaction> {
    tx_validation::validate_priority_fee(max_fee, max_priority_fee, base_fee, disable_priority_fee_check)
}

/// Validate EIP-4844 transaction.
pub fn validate_eip4844_tx(
    blobs: &[B256],
    max_blob_fee: u128,
    block_blob_gas_price: u128,
    max_blobs: Option<u64>,
) -> Result<(), InvalidTransaction> {
    tx_validation::validate_eip4844_tx(blobs, max_blob_fee, block_blob_gas_price, max_blobs)
}

/// Validate transaction against block and configuration for mainnet.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
pub fn validate_tx_env<CTX: ContextTr>(
    context: CTX,
    _spec_id: SpecId,
) -> Result<(), InvalidTransaction> {
    let params = ValidationParams::from_cfg_and_block(context.cfg(), context.block());
    validate_tx(context.tx(), &params)
}

/// Validate initial transaction gas.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
pub fn validate_initial_tx_gas(
    tx: impl Transaction,
    spec: SpecId,
    is_eip7623_disabled: bool,
) -> Result<InitialAndFloorGas, InvalidTransaction> {
    tx_validation::calculate_initial_gas(&tx, spec, is_eip7623_disabled)
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
                    c.set_spec_and_mainnet_gas_params(spec_id);
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
