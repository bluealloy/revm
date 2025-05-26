use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, Cfg, ContextTr,
};
use core::cmp;
use interpreter::gas::{self, InitialAndFloorGas};
use primitives::{eip4844, hardfork::SpecId, B256};

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
    validate_tx_env::<CTX, InvalidTransaction>(context, spec).map_err(Into::into)
}

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<u128>,
) -> Result<(), InvalidTransaction> {
    if max_priority_fee > max_fee {
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
    max_blobs: u64,
) -> Result<(), InvalidTransaction> {
    // Ensure that the user was willing to at least pay the current blob gasprice
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
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
    if blobs.len() > max_blobs as usize {
        return Err(InvalidTransaction::TooManyBlobs {
            have: blobs.len(),
            max: max_blobs as usize,
        });
    }
    Ok(())
}

/// Validate transaction against block and configuration for mainnet.
pub fn validate_tx_env<CTX: ContextTr, Error>(
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

    match TransactionType::from(tx_type) {
        TransactionType::Legacy => {
            // Check chain_id only if it is present in the legacy transaction.
            // EIP-155: Simple replay attack protection
            if let Some(chain_id) = tx.chain_id() {
                if chain_id != context.cfg().chain_id() {
                    return Err(InvalidTransaction::InvalidChainId);
                }
            }
            // Gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if tx.gas_price() < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee);
                }
            }
        }
        TransactionType::Eip2930 => {
            // Enabled in BERLIN hardfork
            if !spec_id.is_enabled_in(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported);
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            // Gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if tx.gas_price() < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee);
                }
            }
        }
        TransactionType::Eip1559 => {
            if !spec_id.is_enabled_in(SpecId::LONDON) {
                return Err(InvalidTransaction::Eip1559NotSupported);
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;
        }
        TransactionType::Eip4844 => {
            if !spec_id.is_enabled_in(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported);
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;

            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                context.block().blob_gasprice().unwrap_or_default(),
                context.cfg().blob_max_count(spec_id),
            )?;
        }
        TransactionType::Eip7702 => {
            // Check if EIP-7702 transaction is enabled.
            if !spec_id.is_enabled_in(SpecId::PRAGUE) {
                return Err(InvalidTransaction::Eip7702NotSupported);
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;

            let auth_list_len = tx.authorization_list_len();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list_len == 0 {
                return Err(InvalidTransaction::EmptyAuthorizationList);
            }
        }
        TransactionType::Eip7873 => {
            // Check if EIP-7873 transaction is enabled.
            // TODO(EOF) EOF removed from spec.
            //if !spec_id.is_enabled_in(SpecId::OSAKA) {
            return Err(InvalidTransaction::Eip7873NotSupported);
            //}
            /*
            // validate chain id
            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            // validate initcodes.
            validate_eip7873_initcodes(tx.initcodes())?;

            // InitcodeTransaction is invalid if the to is nil.
            if tx.kind().is_create() {
                return Err(InvalidTransaction::Eip7873MissingTarget);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;
             */
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

    // EIP-3860: Limit and meter initcode
    if spec_id.is_enabled_in(SpecId::SHANGHAI) && tx.kind().is_create() {
        let max_initcode_size = context.cfg().max_code_size().saturating_mul(2);
        if context.tx().input().len() > max_initcode_size {
            return Err(InvalidTransaction::CreateInitCodeSizeLimit);
        }
    }

    Ok(())
}

/* TODO(EOF)
/// Validate Initcode Transaction initcode list, return error if any of the following conditions are met:
/// * there are zero entries in initcodes, or if there are more than MAX_INITCODE_COUNT entries.
/// * any entry in initcodes is zero length, or if any entry exceeds MAX_INITCODE_SIZE.
/// * the to is nil.
pub fn validate_eip7873_initcodes(initcodes: &[Bytes]) -> Result<(), InvalidTransaction> {
    let mut i = 0;
    for initcode in initcodes {
        // InitcodeTransaction is invalid if any entry in initcodes is zero length
        if initcode.is_empty() {
            return Err(InvalidTransaction::Eip7873EmptyInitcode { i });
        }

        // or if any entry exceeds MAX_INITCODE_SIZE.
        if initcode.len() > MAX_INITCODE_SIZE {
            return Err(InvalidTransaction::Eip7873InitcodeTooLarge {
                i,
                size: initcode.len(),
            });
        }

        i += 1;
    }

    // InitcodeTransaction is invalid if there are zero entries in initcodes,
    if i == 0 {
        return Err(InvalidTransaction::Eip7873EmptyInitcodeList);
    }

    // or if there are more than MAX_INITCODE_COUNT entries.
    if i > MAX_INITCODE_COUNT {
        return Err(InvalidTransaction::Eip7873TooManyInitcodes { size: i });
    }

    Ok(())
}
*/

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas(
    tx: impl Transaction,
    spec: SpecId,
) -> Result<InitialAndFloorGas, InvalidTransaction> {
    let gas = gas::calculate_initial_tx_gas_for_tx(&tx, spec);

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
    use crate::{ExecuteCommitEvm, MainBuilder, MainContext};
    use bytecode::opcode;
    use context::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, Output},
        Context,
    };
    use database::{CacheDB, EmptyDB};
    use primitives::{address, Address, Bytes, TxKind, MAX_INITCODE_SIZE};

    fn deploy_contract(
        bytecode: Bytes,
    ) -> Result<ExecutionResult, EVMError<core::convert::Infallible>> {
        let ctx = Context::mainnet()
            .modify_tx_chained(|tx| {
                tx.kind = TxKind::Create;
                tx.data = bytecode.clone();
            })
            .with_db(CacheDB::<EmptyDB>::default());

        let mut evm = ctx.build_mainnet();
        evm.replay_commit()
    }

    #[test]
    fn test_eip3860_initcode_size_limit_failure() {
        let large_bytecode = vec![opcode::STOP; MAX_INITCODE_SIZE + 1];
        let bytecode: Bytes = large_bytecode.into();
        let result = deploy_contract(bytecode);
        assert!(matches!(
            result,
            Err(EVMError::Transaction(
                InvalidTransaction::CreateInitCodeSizeLimit
            ))
        ));
    }

    #[test]
    fn test_eip3860_initcode_size_limit_success() {
        let large_bytecode = vec![opcode::STOP; MAX_INITCODE_SIZE];
        let bytecode: Bytes = large_bytecode.into();
        let result = deploy_contract(bytecode);
        assert!(matches!(result, Ok(ExecutionResult::Success { .. })));
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
        let result = deploy_contract(bytecode);
        assert!(matches!(
            result,
            Ok(ExecutionResult::Halt {
                reason: HaltReason::CreateContractSizeLimit,
                ..
            },)
        ));
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
        let result = deploy_contract(bytecode);
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
        let factory_result =
            deploy_contract(factory_bytecode).expect("factory contract deployment failed");

        // get factory contract address
        let factory_address = match &factory_result {
            ExecutionResult::Success { output, .. } => match output {
                Output::Create(bytes, _) | Output::Call(bytes) => Address::from_slice(&bytes[..20]),
            },
            _ => panic!("factory contract deployment failed"),
        };

        // call factory contract to create sub contract
        let tx_caller = address!("0x0000000000000000000000000000000000100000");
        let call_result = Context::mainnet()
            .modify_tx_chained(|tx| {
                tx.caller = tx_caller;
                tx.kind = TxKind::Call(factory_address);
                tx.data = Bytes::new();
            })
            .with_db(CacheDB::<EmptyDB>::default())
            .build_mainnet()
            .replay_commit()
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
        let factory_result =
            deploy_contract(factory_bytecode).expect("factory contract deployment failed");
        // get factory contract address
        let factory_address = match &factory_result {
            ExecutionResult::Success { output, .. } => match output {
                Output::Create(bytes, _) | Output::Call(bytes) => Address::from_slice(&bytes[..20]),
            },
            _ => panic!("factory contract deployment failed"),
        };

        // call factory contract to create sub contract
        let tx_caller = address!("0x0000000000000000000000000000000000100000");
        let call_result = Context::mainnet()
            .modify_tx_chained(|tx| {
                tx.caller = tx_caller;
                tx.kind = TxKind::Call(factory_address);
                tx.data = Bytes::new();
            })
            .with_db(CacheDB::<EmptyDB>::default())
            .build_mainnet()
            .replay_commit()
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
}
