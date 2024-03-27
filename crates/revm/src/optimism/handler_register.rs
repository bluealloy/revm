//! Handler related to Optimism chain

use crate::{
    handler::{
        mainnet::{self, deduct_caller_inner},
        register::EvmHandler,
    },
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    optimism,
    primitives::{
        db::Database, spec_to_generic, Account, EVMError, Env, ExecutionResult, HaltReason,
        HashMap, InvalidTransaction, ResultAndState, Spec, SpecId, SpecId::REGOLITH, U256,
    },
    Context, FrameResult,
};
use core::ops::Mul;
use std::string::ToString;
use std::sync::Arc;

pub fn optimism_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.cfg.spec_id, {
        // validate environment
        handler.validation.env = Arc::new(validate_env::<SPEC, DB>);
        // Validate transaction against state.
        handler.validation.tx_against_state = Arc::new(validate_tx_against_state::<SPEC, EXT, DB>);
        // load l1 data
        handler.pre_execution.load_accounts = Arc::new(load_accounts::<SPEC, EXT, DB>);
        // An estimated batch cost is charged from the caller and added to L1 Fee Vault.
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<SPEC, EXT, DB>);
        // Refund is calculated differently then mainnet.
        handler.execution.last_frame_return = Arc::new(last_frame_return::<SPEC, EXT, DB>);
        handler.post_execution.reward_beneficiary = Arc::new(reward_beneficiary::<SPEC, EXT, DB>);
        // In case of halt of deposit transaction return Error.
        handler.post_execution.output = Arc::new(output::<SPEC, EXT, DB>);
        handler.post_execution.end = Arc::new(end::<SPEC, EXT, DB>);
    });
}

/// Validate environment for the Optimism chain.
pub fn validate_env<SPEC: Spec, DB: Database>(env: &Env) -> Result<(), EVMError<DB::Error>> {
    // Do not perform any extra validation for deposit transactions, they are pre-verified on L1.
    if env.tx.optimism.source_hash.is_some() {
        return Ok(());
    }
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;

    // Do not allow for a system transaction to be processed if Regolith is enabled.
    let tx = &env.tx.optimism;
    if tx.is_system_transaction.unwrap_or(false) && SPEC::enabled(SpecId::REGOLITH) {
        return Err(InvalidTransaction::DepositSystemTxPostRegolith.into());
    }

    env.validate_tx::<SPEC>()?;
    Ok(())
}

/// Don not perform any extra validation for deposit transactions, they are pre-verified on L1.
pub fn validate_tx_against_state<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    if context.evm.inner.env.tx.optimism.source_hash.is_some() {
        return Ok(());
    }
    mainnet::validate_tx_against_state::<SPEC, EXT, DB>(context)
}

/// Handle output of the transaction
#[inline]
pub fn last_frame_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame_result: &mut FrameResult,
) -> Result<(), EVMError<DB::Error>> {
    let env = context.evm.inner.env();
    let is_deposit = env.tx.optimism.source_hash.is_some();
    let tx_system = env.tx.optimism.is_system_transaction;
    let tx_gas_limit = env.tx.gas_limit;
    let is_regolith = SPEC::enabled(REGOLITH);

    let instruction_result = frame_result.interpreter_result().result;
    let gas = frame_result.gas_mut();
    let remaining = gas.remaining();
    let refunded = gas.refunded();
    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    *gas = Gas::new(tx_gas_limit);
    gas.record_cost(tx_gas_limit);

    match instruction_result {
        return_ok!() => {
            // On Optimism, deposit transactions report gas usage uniquely to other
            // transactions due to them being pre-paid on L1.
            //
            // Hardfork Behavior:
            // - Bedrock (success path):
            //   - Deposit transactions (non-system) report their gas limit as the usage.
            //     No refunds.
            //   - Deposit transactions (system) report 0 gas used. No refunds.
            //   - Regular transactions report gas usage as normal.
            // - Regolith (success path):
            //   - Deposit transactions (all) report their gas used as normal. Refunds
            //     enabled.
            //   - Regular transactions report their gas used as normal.
            if !is_deposit || is_regolith {
                // For regular transactions prior to Regolith and all transactions after
                // Regolith, gas is reported as normal.
                gas.erase_cost(remaining);
                gas.record_refund(refunded);
            } else if is_deposit && tx_system.unwrap_or(false) {
                // System transactions were a special type of deposit transaction in
                // the Bedrock hardfork that did not incur any gas costs.
                gas.erase_cost(tx_gas_limit);
            }
        }
        return_revert!() => {
            // On Optimism, deposit transactions report gas usage uniquely to other
            // transactions due to them being pre-paid on L1.
            //
            // Hardfork Behavior:
            // - Bedrock (revert path):
            //   - Deposit transactions (all) report the gas limit as the amount of gas
            //     used on failure. No refunds.
            //   - Regular transactions receive a refund on remaining gas as normal.
            // - Regolith (revert path):
            //   - Deposit transactions (all) report the actual gas used as the amount of
            //     gas used on failure. Refunds on remaining gas enabled.
            //   - Regular transactions receive a refund on remaining gas as normal.
            if !is_deposit || is_regolith {
                gas.erase_cost(remaining);
            }
        }
        _ => {}
    }
    // Prior to Regolith, deposit transactions did not receive gas refunds.
    let is_gas_refund_disabled = env.cfg.is_gas_refund_disabled() || (is_deposit && !is_regolith);
    if !is_gas_refund_disabled {
        gas.set_final_refund::<SPEC>();
    }
    Ok(())
}

/// Load account (make them warm) and l1 data from database.
#[inline]
pub fn load_accounts<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // the L1-cost fee is only computed for Optimism non-deposit transactions.

    if context.evm.inner.env.tx.optimism.source_hash.is_none() {
        let l1_block_info =
            crate::optimism::L1BlockInfo::try_fetch(&mut context.evm.inner.db, SPEC::SPEC_ID)
                .map_err(EVMError::Database)?;

        // storage l1 block info for later use.
        context.evm.inner.l1_block_info = Some(l1_block_info);
    }

    mainnet::load_accounts::<SPEC, EXT, DB>(context)
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

    // If the transaction is a deposit with a `mint` value, add the mint value
    // in wei to the caller's balance. This should be persisted to the database
    // prior to the rest of execution.
    if let Some(mint) = context.evm.inner.env.tx.optimism.mint {
        caller_account.info.balance += U256::from(mint);
    }

    // We deduct caller max balance after minting and before deducing the
    // l1 cost, max values is already checked in pre_validate but l1 cost wasn't.
    deduct_caller_inner::<SPEC>(caller_account, &context.evm.inner.env);

    // If the transaction is not a deposit transaction, subtract the L1 data fee from the
    // caller's balance directly after minting the requested amount of ETH.
    if context.evm.inner.env.tx.optimism.source_hash.is_none() {
        // get envelope
        let Some(enveloped_tx) = &context.evm.inner.env.tx.optimism.enveloped_tx else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".to_string(),
            ));
        };

        let tx_l1_cost = context
            .evm
            .inner
            .l1_block_info
            .as_ref()
            .expect("L1BlockInfo should be loaded")
            .calculate_tx_l1_cost(enveloped_tx, SPEC::SPEC_ID);
        if tx_l1_cost.gt(&caller_account.info.balance) {
            return Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: tx_l1_cost.into(),
                    balance: caller_account.info.balance.into(),
                },
            ));
        }
        caller_account.info.balance = caller_account.info.balance.saturating_sub(tx_l1_cost);
    }
    Ok(())
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let is_deposit = context.evm.inner.env.tx.optimism.source_hash.is_some();

    // transfer fee to coinbase/beneficiary.
    if !is_deposit {
        mainnet::reward_beneficiary::<SPEC, EXT, DB>(context, gas)?;
    }

    if !is_deposit {
        // If the transaction is not a deposit transaction, fees are paid out
        // to both the Base Fee Vault as well as the L1 Fee Vault.
        let Some(l1_block_info) = &context.evm.inner.l1_block_info else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load L1 block information.".to_string(),
            ));
        };

        let Some(enveloped_tx) = &context.evm.inner.env.tx.optimism.enveloped_tx else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".to_string(),
            ));
        };

        let l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, SPEC::SPEC_ID);

        // Send the L1 cost of the transaction to the L1 Fee Vault.
        let Ok((l1_fee_vault_account, _)) = context
            .evm
            .inner
            .journaled_state
            .load_account(optimism::L1_FEE_RECIPIENT, &mut context.evm.inner.db)
        else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load L1 Fee Vault account.".to_string(),
            ));
        };
        l1_fee_vault_account.mark_touch();
        l1_fee_vault_account.info.balance += l1_cost;

        // Send the base fee of the transaction to the Base Fee Vault.
        let Ok((base_fee_vault_account, _)) = context
            .evm
            .inner
            .journaled_state
            .load_account(optimism::BASE_FEE_RECIPIENT, &mut context.evm.inner.db)
        else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load Base Fee Vault account.".to_string(),
            ));
        };
        base_fee_vault_account.mark_touch();
        base_fee_vault_account.info.balance += context
            .evm
            .inner
            .env
            .block
            .basefee
            .mul(U256::from(gas.spent() - gas.refunded() as u64));
    }
    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn output<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame_result: FrameResult,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    let result = mainnet::output::<EXT, DB>(context, frame_result)?;

    if result.result.is_halt() {
        // Post-regolith, if the transaction is a deposit transaction and it halts,
        // we bubble up to the global return handler. The mint value will be persisted
        // and the caller nonce will be incremented there.
        let is_deposit = context.evm.inner.env.tx.optimism.source_hash.is_some();
        if is_deposit && SPEC::enabled(REGOLITH) {
            return Err(EVMError::Transaction(
                InvalidTransaction::HaltedDepositPostRegolith,
            ));
        }
    }
    Ok(result)
}
/// Optimism end handle changes output if the transaction is a deposit transaction.
/// Deposit transaction can't be reverted and is always successful.
#[inline]
pub fn end<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    evm_output: Result<ResultAndState, EVMError<DB::Error>>,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    evm_output.or_else(|err| {
        if matches!(err, EVMError::Transaction(_))
            && context.evm.inner.env().tx.optimism.source_hash.is_some()
        {
            // If the transaction is a deposit transaction and it failed
            // for any reason, the caller nonce must be bumped, and the
            // gas reported must be altered depending on the Hardfork. This is
            // also returned as a special Halt variant so that consumers can more
            // easily distinguish between a failed deposit and a failed
            // normal transaction.
            let caller = context.evm.inner.env().tx.caller;

            // Increment sender nonce and account balance for the mint amount. Deposits
            // always persist the mint amount, even if the transaction fails.
            let account = {
                let mut acc = Account::from(
                    context
                        .evm
                        .db
                        .basic(caller)
                        .unwrap_or_default()
                        .unwrap_or_default(),
                );
                acc.info.nonce = acc.info.nonce.saturating_add(1);
                acc.info.balance = acc.info.balance.saturating_add(U256::from(
                    context.evm.inner.env().tx.optimism.mint.unwrap_or(0),
                ));
                acc.mark_touch();
                acc
            };
            let state = HashMap::from([(caller, account)]);

            // The gas used of a failed deposit post-regolith is the gas
            // limit of the transaction. pre-regolith, it is the gas limit
            // of the transaction for non system transactions and 0 for system
            // transactions.
            let is_system_tx = context
                .evm
                .env()
                .tx
                .optimism
                .is_system_transaction
                .unwrap_or(false);
            let gas_used = if SPEC::enabled(REGOLITH) || !is_system_tx {
                context.evm.inner.env().tx.gas_limit
            } else {
                0
            };

            Ok(ResultAndState {
                result: ExecutionResult::Halt {
                    reason: HaltReason::FailedDeposit,
                    gas_used,
                },
                state,
            })
        } else {
            Err(err)
        }
    })
}

#[cfg(test)]
mod tests {
    use revm_interpreter::{CallOutcome, InterpreterResult};

    use super::*;
    use crate::{
        db::{EmptyDB, InMemoryDB},
        primitives::{
            bytes, state::AccountInfo, Address, BedrockSpec, Bytes, Env, LatestSpec, RegolithSpec,
            B256,
        },
        L1BlockInfo,
    };

    /// Creates frame result.
    fn call_last_frame_return<SPEC: Spec>(
        env: Env,
        instruction_result: InstructionResult,
        gas: Gas,
    ) -> Gas {
        let mut ctx = Context::new_empty();
        ctx.evm.inner.env = Box::new(env);
        let mut first_frame = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));
        last_frame_return::<SPEC, _, _>(&mut ctx, &mut first_frame).unwrap();
        *first_frame.gas()
    }

    #[test]
    fn test_revert_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.tx.optimism.source_hash = None;

        let gas =
            call_last_frame_return::<BedrockSpec>(env, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let gas =
            call_last_frame_return::<RegolithSpec>(env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let mut ret_gas = Gas::new(90);
        ret_gas.record_refund(20);

        let gas =
            call_last_frame_return::<RegolithSpec>(env.clone(), InstructionResult::Stop, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 2); // min(20, 10/5)

        let gas = call_last_frame_return::<RegolithSpec>(env, InstructionResult::Revert, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_sys_deposit_tx() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let gas = call_last_frame_return::<BedrockSpec>(env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 0);
        assert_eq!(gas.spent(), 100);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_commit_mint_value() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(1000),
                ..Default::default()
            },
        );
        let mut context: Context<(), InMemoryDB> = Context::new_with_db(db);
        context.evm.inner.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // Enveloped needs to be some but it will deduce zero fee.
        context.evm.inner.env.tx.optimism.enveloped_tx = Some(bytes!(""));
        // added mint value is 10.
        context.evm.inner.env.tx.optimism.mint = Some(10);

        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .inner
            .journaled_state
            .load_account(caller, &mut context.evm.inner.db)
            .unwrap();
        assert_eq!(account.info.balance, U256::from(1010));
    }

    #[test]
    fn test_remove_l1_cost_non_deposit() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(1000),
                ..Default::default()
            },
        );
        let mut context: Context<(), InMemoryDB> = Context::new_with_db(db);
        context.evm.inner.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));
        // added mint value is 10.
        context.evm.inner.env.tx.optimism.mint = Some(10);
        // Putting source_hash to some makes it a deposit transaction.
        // so enveloped_tx gas cost is ignored.
        context.evm.inner.env.tx.optimism.source_hash = Some(B256::ZERO);

        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .inner
            .journaled_state
            .load_account(caller, &mut context.evm.inner.db)
            .unwrap();
        assert_eq!(account.info.balance, U256::from(1010));
    }

    #[test]
    fn test_remove_l1_cost() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(1049),
                ..Default::default()
            },
        );
        let mut context: Context<(), InMemoryDB> = Context::new_with_db(db);
        context.evm.inner.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));
        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .inner
            .journaled_state
            .load_account(caller, &mut context.evm.inner.db)
            .unwrap();
        assert_eq!(account.info.balance, U256::from(1));
    }

    #[test]
    fn test_remove_l1_cost_lack_of_funds() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(48),
                ..Default::default()
            },
        );
        let mut context: Context<(), InMemoryDB> = Context::new_with_db(db);
        context.evm.inner.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));

        assert_eq!(
            deduct_caller::<RegolithSpec, (), _>(&mut context),
            Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: Box::new(U256::from(1048)),
                    balance: Box::new(U256::from(48)),
                },
            ))
        );
    }

    #[test]
    fn test_validate_sys_tx() {
        // mark the tx as a system transaction.
        let mut env = Env::default();
        env.tx.optimism.is_system_transaction = Some(true);
        assert_eq!(
            validate_env::<RegolithSpec, EmptyDB>(&env),
            Err(EVMError::Transaction(
                InvalidTransaction::DepositSystemTxPostRegolith
            ))
        );

        // Pre-regolith system transactions should be allowed.
        assert!(validate_env::<BedrockSpec, EmptyDB>(&env).is_ok());
    }

    #[test]
    fn test_validate_deposit_tx() {
        // Set source hash.
        let mut env = Env::default();
        env.tx.optimism.source_hash = Some(B256::ZERO);
        assert!(validate_env::<RegolithSpec, EmptyDB>(&env).is_ok());
    }

    #[test]
    fn test_validate_tx_against_state_deposit_tx() {
        // Set source hash.
        let mut env = Env::default();
        env.tx.optimism.source_hash = Some(B256::ZERO);

        // Nonce and balance checks should be skipped for deposit transactions.
        assert!(validate_env::<LatestSpec, EmptyDB>(&env).is_ok());
    }
}
