//! Handler related to Optimism chain

use crate::{
    handler::{
        mainnet::{self, deduct_caller_inner},
        register::EvmHandler,
    },
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    optimism,
    primitives::{
        db::Database, spec_to_generic, Account, EVMError, ExecutionResult, HaltReason, HashMap,
        InvalidTransaction, ResultAndState, Spec, SpecId, SpecId::REGOLITH, U256,
    },
    Context, FrameResult,
};
use alloc::sync::Arc;
use core::ops::Mul;

pub fn optimism_handle_register<DB: Database, EXT>(handler: &mut EvmHandler<'_, EXT, DB>) {
    spec_to_generic!(handler.spec_id, {
        // Refund is calculated differently then mainnet.
        handler.execution.last_frame_return = Arc::new(last_frame_return::<SPEC, EXT, DB>);
        // An estimated batch cost is charged from the caller and added to L1 Fee Vault.
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<SPEC, EXT, DB>);
        handler.post_execution.reward_beneficiary = Arc::new(reward_beneficiary::<SPEC, EXT, DB>);
        // In case of halt of deposit transaction return Error.
        handler.post_execution.output = Arc::new(output::<SPEC, EXT, DB>);
        handler.post_execution.end = Arc::new(end::<SPEC, EXT, DB>);
    });
}

/// Handle output of the transaction
#[inline]
pub fn last_frame_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame_result: &mut FrameResult,
) {
    let env = context.evm.env();
    let is_deposit = env.tx.optimism.source_hash.is_some();
    let is_optimism = env.cfg.optimism;
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
            if is_optimism && (!is_deposit || is_regolith) {
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
            if is_optimism && (!is_deposit || is_regolith) {
                gas.erase_cost(remaining);
            }
        }
        _ => {}
    }
    // Prior to Regolith, deposit transactions did not receive gas refunds.
    let is_gas_refund_disabled = is_optimism && is_deposit && !is_regolith;
    if !is_gas_refund_disabled {
        gas.set_final_refund::<SPEC>();
    }
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(context.evm.env.tx.caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    // If the transaction is a deposit with a `mint` value, add the mint value
    // in wei to the caller's balance. This should be persisted to the database
    // prior to the rest of execution.
    if let Some(mint) = context.evm.env.tx.optimism.mint {
        caller_account.info.balance += U256::from(mint);
    }

    // We deduct caller max balance after minting and before deducing the
    // l1 cost, max values is already checked in pre_validate but l1 cost wasn't.
    deduct_caller_inner::<SPEC>(caller_account, &context.evm.env);

    // If the transaction is not a deposit transaction, subtract the L1 data fee from the
    // caller's balance directly after minting the requested amount of ETH.
    if context.evm.env.tx.optimism.source_hash.is_none() {
        // get envelope
        let Some(enveloped_tx) = context.evm.env.tx.optimism.enveloped_tx.clone() else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".to_string(),
            ));
        };

        let tx_l1_cost = context
            .evm
            .l1_block_info
            .as_ref()
            .expect("L1BlockInfo should be loaded")
            .calculate_tx_l1_cost(&enveloped_tx, SPEC::SPEC_ID);
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
    let is_deposit =
        context.evm.env.cfg.optimism && context.evm.env.tx.optimism.source_hash.is_some();
    let disable_coinbase_tip = context.evm.env.cfg.optimism && is_deposit;

    // transfer fee to coinbase/beneficiary.
    if !disable_coinbase_tip {
        mainnet::reward_beneficiary::<SPEC, EXT, DB>(context, gas)?;
    }

    if context.evm.env.cfg.optimism && !is_deposit {
        // If the transaction is not a deposit transaction, fees are paid out
        // to both the Base Fee Vault as well as the L1 Fee Vault.
        let Some(l1_block_info) = context.evm.l1_block_info.clone() else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load L1 block information.".to_string(),
            ));
        };

        let Some(enveloped_tx) = &context.evm.env.tx.optimism.enveloped_tx else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".to_string(),
            ));
        };

        let l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, SPEC::SPEC_ID);

        // Send the L1 cost of the transaction to the L1 Fee Vault.
        let Ok((l1_fee_vault_account, _)) = context
            .evm
            .journaled_state
            .load_account(optimism::L1_FEE_RECIPIENT, &mut context.evm.db)
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
            .journaled_state
            .load_account(optimism::BASE_FEE_RECIPIENT, &mut context.evm.db)
        else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load Base Fee Vault account.".to_string(),
            ));
        };
        base_fee_vault_account.mark_touch();
        base_fee_vault_account.info.balance += context
            .evm
            .env
            .block
            .basefee
            .mul(U256::from(gas.spend() - gas.refunded() as u64));
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
        let is_deposit = context.evm.env.tx.optimism.source_hash.is_some();
        let optimism_regolith = context.evm.env.cfg.optimism && SPEC::enabled(REGOLITH);
        if is_deposit && optimism_regolith {
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
            && context.evm.env().cfg.optimism
            && context.evm.env().tx.optimism.source_hash.is_some()
        {
            // If the transaction is a deposit transaction and it failed
            // for any reason, the caller nonce must be bumped, and the
            // gas reported must be altered depending on the Hardfork. This is
            // also returned as a special Halt variant so that consumers can more
            // easily distinguish between a failed deposit and a failed
            // normal transaction.
            let caller = context.evm.env().tx.caller;

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
                acc.info.balance = acc
                    .info
                    .balance
                    .saturating_add(U256::from(context.evm.env().tx.optimism.mint.unwrap_or(0)));
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
                context.evm.env().tx.gas_limit
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
        db::InMemoryDB,
        primitives::{
            bytes, state::AccountInfo, Address, BedrockSpec, Bytes, Env, RegolithSpec, B256,
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
        ctx.evm.env = Box::new(env);
        let mut first_frame = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));
        last_frame_return::<SPEC, _, _>(&mut ctx, &mut first_frame);
        *first_frame.gas()
    }

    #[test]
    fn test_revert_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.cfg.optimism = true;
        env.tx.optimism.source_hash = None;

        let gas =
            call_last_frame_return::<BedrockSpec>(env, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_revert_gas_non_optimism() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.cfg.optimism = false;
        env.tx.optimism.source_hash = None;

        let gas =
            call_last_frame_return::<BedrockSpec>(env, InstructionResult::Revert, Gas::new(90));
        // else branch takes all gas.
        assert_eq!(gas.remaining(), 0);
        assert_eq!(gas.spend(), 100);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.cfg.optimism = true;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let gas =
            call_last_frame_return::<RegolithSpec>(env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.cfg.optimism = true;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let mut ret_gas = Gas::new(90);
        ret_gas.record_refund(20);

        let gas =
            call_last_frame_return::<RegolithSpec>(env.clone(), InstructionResult::Stop, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 2); // min(20, 10/5)

        let gas = call_last_frame_return::<RegolithSpec>(env, InstructionResult::Revert, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_sys_deposit_tx() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;
        env.cfg.optimism = true;
        env.tx.optimism.source_hash = Some(B256::ZERO);

        let gas = call_last_frame_return::<BedrockSpec>(env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 0);
        assert_eq!(gas.spend(), 100);
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
        context.evm.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // Enveloped needs to be some but it will deduce zero fee.
        context.evm.env.tx.optimism.enveloped_tx = Some(bytes!(""));
        // added mint value is 10.
        context.evm.env.tx.optimism.mint = Some(10);

        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .journaled_state
            .load_account(caller, &mut context.evm.db)
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
        context.evm.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));
        // added mint value is 10.
        context.evm.env.tx.optimism.mint = Some(10);
        // Putting source_hash to some makes it a deposit transaction.
        // so enveloped_tx gas cost is ignored.
        context.evm.env.tx.optimism.source_hash = Some(B256::ZERO);

        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .journaled_state
            .load_account(caller, &mut context.evm.db)
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
        context.evm.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));
        deduct_caller::<RegolithSpec, (), _>(&mut context).unwrap();

        // Check the account balance is updated.
        let (account, _) = context
            .evm
            .journaled_state
            .load_account(caller, &mut context.evm.db)
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
        context.evm.l1_block_info = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.env.tx.optimism.enveloped_tx = Some(bytes!("FACADE"));

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
}
