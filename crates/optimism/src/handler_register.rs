//! Handler related to Optimism chain

use super::{
    optimism_spec_to_generic, OptimismContext, OptimismHaltReason, OptimismInvalidTransaction,
    OptimismSpec, OptimismSpecId, OptimismTransaction, OptimismWiring,
};
use crate::{BASE_FEE_RECIPIENT, L1_FEE_RECIPIENT};
use core::ops::Mul;
use revm::{
    handler::{
        mainnet::{self, deduct_caller_inner},
        register::EvmHandler,
    },
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    precompile::{secp256r1, PrecompileSpecId},
    primitives::{
        db::Database, Account, Block, EVMError, EVMResult, EVMResultGeneric, EnvWiring,
        ExecutionResult, HashMap, InvalidTransaction, ResultAndState, Transaction, U256,
    },
    Context, ContextPrecompiles, FrameResult,
};
use std::sync::Arc;

pub fn optimism_handle_register<EvmWiringT>(handler: &mut EvmHandler<'_, EvmWiringT>)
where
    EvmWiringT: OptimismWiring,
{
    optimism_spec_to_generic!(handler.spec_id, {
        // validate environment
        handler.validation.env = Arc::new(validate_env::<EvmWiringT, SPEC>);
        // Validate transaction against state.
        handler.validation.tx_against_state =
            Arc::new(validate_tx_against_state::<EvmWiringT, SPEC>);
        // Load additional precompiles for the given chain spec.
        handler.pre_execution.load_precompiles = Arc::new(load_precompiles::<EvmWiringT, SPEC>);
        // load l1 data
        handler.pre_execution.load_accounts = Arc::new(load_accounts::<EvmWiringT, SPEC>);
        // An estimated batch cost is charged from the caller and added to L1 Fee Vault.
        handler.pre_execution.deduct_caller = Arc::new(deduct_caller::<EvmWiringT, SPEC>);
        // Refund is calculated differently then mainnet.
        handler.execution.last_frame_return = Arc::new(last_frame_return::<EvmWiringT, SPEC>);
        handler.post_execution.refund = Arc::new(refund::<EvmWiringT, SPEC>);
        handler.post_execution.reward_beneficiary =
            Arc::new(reward_beneficiary::<EvmWiringT, SPEC>);
        // In case of halt of deposit transaction return Error.
        handler.post_execution.output = Arc::new(output::<EvmWiringT, SPEC>);
        handler.post_execution.end = Arc::new(end::<EvmWiringT, SPEC>);
    });
}

/// Validate environment for the Optimism chain.
pub fn validate_env<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    env: &EnvWiring<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    // Do not perform any extra validation for deposit transactions, they are pre-verified on L1.
    if env.tx.source_hash().is_some() {
        return Ok(());
    }

    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;

    // Do not allow for a system transaction to be processed if Regolith is enabled.
    if env.tx.is_system_transaction().unwrap_or(false)
        && SPEC::optimism_enabled(OptimismSpecId::REGOLITH)
    {
        return Err(OptimismInvalidTransaction::DepositSystemTxPostRegolith.into());
    }

    env.validate_tx::<SPEC>()
        .map_err(OptimismInvalidTransaction::Base)?;

    Ok(())
}

/// Don not perform any extra validation for deposit transactions, they are pre-verified on L1.
pub fn validate_tx_against_state<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    if context.evm.inner.env.tx.source_hash().is_some() {
        return Ok(());
    }
    mainnet::validate_tx_against_state::<EvmWiringT, SPEC>(context)
}

/// Handle output of the transaction
#[inline]
pub fn last_frame_return<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
    frame_result: &mut FrameResult,
) -> EVMResultGeneric<(), EvmWiringT> {
    let env = context.evm.inner.env();
    let is_deposit = env.tx.source_hash().is_some();
    let tx_system = env.tx.is_system_transaction();
    let tx_gas_limit = env.tx.gas_limit();
    let is_regolith = SPEC::optimism_enabled(OptimismSpecId::REGOLITH);

    let instruction_result = frame_result.interpreter_result().result;
    let gas = frame_result.gas_mut();
    let remaining = gas.remaining();
    let refunded = gas.refunded();
    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    *gas = Gas::new_spent(tx_gas_limit);

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
    Ok(())
}

/// Record Eip-7702 refund and calculate final refund.
#[inline]
pub fn refund<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
    gas: &mut Gas,
    eip7702_refund: i64,
) {
    gas.record_refund(eip7702_refund);

    let env = context.evm.inner.env();
    let is_deposit = env.tx.source_hash().is_some();
    let is_regolith = SPEC::optimism_enabled(OptimismSpecId::REGOLITH);

    // Prior to Regolith, deposit transactions did not receive gas refunds.
    let is_gas_refund_disabled = env.cfg.is_gas_refund_disabled() || (is_deposit && !is_regolith);
    if !is_gas_refund_disabled {
        gas.set_final_refund(SPEC::OPTIMISM_SPEC_ID.is_enabled_in(OptimismSpecId::LONDON));
    }
}

/// Load precompiles for Optimism chain.
#[inline]
pub fn load_precompiles<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
) -> ContextPrecompiles<EvmWiringT> {
    let mut precompiles = ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID));

    if SPEC::optimism_enabled(OptimismSpecId::FJORD) {
        precompiles.extend([
            // EIP-7212: secp256r1 P256verify
            secp256r1::P256VERIFY,
        ])
    }

    if SPEC::optimism_enabled(OptimismSpecId::GRANITE) {
        precompiles.extend([
            // Restrict bn256Pairing input size
            crate::bn128::pair::GRANITE,
        ])
    }

    precompiles
}

/// Load account (make them warm) and l1 data from database.
#[inline]
pub fn load_accounts<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    // the L1-cost fee is only computed for Optimism non-deposit transactions.

    if context.evm.env.tx.source_hash().is_none() {
        let l1_block_info =
            super::L1BlockInfo::try_fetch(&mut context.evm.inner.db, SPEC::OPTIMISM_SPEC_ID)
                .map_err(EVMError::Database)?;

        // storage l1 block info for later use.
        *context.evm.chain.l1_block_info_mut() = Some(l1_block_info);
    }

    mainnet::load_accounts::<EvmWiringT, SPEC>(context)
}

/// Deduct max balance from caller
#[inline]
pub fn deduct_caller<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT> {
    // load caller's account.
    let mut caller_account = context
        .evm
        .inner
        .journaled_state
        .load_account(
            *context.evm.inner.env.tx.caller(),
            &mut context.evm.inner.db,
        )
        .map_err(EVMError::Database)?;

    // If the transaction is a deposit with a `mint` value, add the mint value
    // in wei to the caller's balance. This should be persisted to the database
    // prior to the rest of execution.
    if let Some(mint) = context.evm.inner.env.tx.mint() {
        caller_account.info.balance += U256::from(*mint);
    }

    // We deduct caller max balance after minting and before deducing the
    // l1 cost, max values is already checked in pre_validate but l1 cost wasn't.
    deduct_caller_inner::<EvmWiringT, SPEC>(caller_account.data, &context.evm.inner.env);

    // If the transaction is not a deposit transaction, subtract the L1 data fee from the
    // caller's balance directly after minting the requested amount of ETH.
    if context.evm.inner.env.tx.source_hash().is_none() {
        // get envelope
        let Some(enveloped_tx) = &context.evm.inner.env.tx.enveloped_tx() else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".into(),
            ));
        };

        let tx_l1_cost = context
            .evm
            .inner
            .chain
            .l1_block_info()
            .expect("L1BlockInfo should be loaded")
            .calculate_tx_l1_cost(enveloped_tx, SPEC::OPTIMISM_SPEC_ID);
        if tx_l1_cost.gt(&caller_account.info.balance) {
            return Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: tx_l1_cost.into(),
                    balance: caller_account.info.balance.into(),
                }
                .into(),
            ));
        }
        caller_account.info.balance = caller_account.info.balance.saturating_sub(tx_l1_cost);
    }
    Ok(())
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
    gas: &Gas,
) -> EVMResultGeneric<(), EvmWiringT> {
    let is_deposit = context.evm.inner.env.tx.source_hash().is_some();

    // transfer fee to coinbase/beneficiary.
    if !is_deposit {
        mainnet::reward_beneficiary::<EvmWiringT, SPEC>(context, gas)?;
    }

    if !is_deposit {
        // If the transaction is not a deposit transaction, fees are paid out
        // to both the Base Fee Vault as well as the L1 Fee Vault.
        let l1_block_info = context
            .evm
            .chain
            .l1_block_info()
            .expect("L1BlockInfo should be loaded");

        let Some(enveloped_tx) = &context.evm.inner.env.tx.enveloped_tx() else {
            return Err(EVMError::Custom(
                "[OPTIMISM] Failed to load enveloped transaction.".into(),
            ));
        };

        let l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, SPEC::OPTIMISM_SPEC_ID);

        // Send the L1 cost of the transaction to the L1 Fee Vault.
        let mut l1_fee_vault_account = context
            .evm
            .inner
            .journaled_state
            .load_account(L1_FEE_RECIPIENT, &mut context.evm.inner.db)
            .map_err(EVMError::Database)?;
        l1_fee_vault_account.mark_touch();
        l1_fee_vault_account.info.balance += l1_cost;

        // Send the base fee of the transaction to the Base Fee Vault.
        let mut base_fee_vault_account = context
            .evm
            .inner
            .journaled_state
            .load_account(BASE_FEE_RECIPIENT, &mut context.evm.inner.db)
            .map_err(EVMError::Database)?;
        base_fee_vault_account.mark_touch();
        base_fee_vault_account.info.balance += context
            .evm
            .inner
            .env
            .block
            .basefee()
            .mul(U256::from(gas.spent() - gas.refunded() as u64));
    }
    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn output<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
    frame_result: FrameResult,
) -> EVMResult<EvmWiringT> {
    let result = mainnet::output::<EvmWiringT>(context, frame_result)?;

    if result.result.is_halt() {
        // Post-regolith, if the transaction is a deposit transaction and it halts,
        // we bubble up to the global return handler. The mint value will be persisted
        // and the caller nonce will be incremented there.
        let is_deposit = context.evm.inner.env.tx.source_hash().is_some();
        if is_deposit && SPEC::optimism_enabled(OptimismSpecId::REGOLITH) {
            return Err(EVMError::Transaction(
                OptimismInvalidTransaction::HaltedDepositPostRegolith,
            ));
        }
    }
    Ok(result)
}
/// Optimism end handle changes output if the transaction is a deposit transaction.
/// Deposit transaction can't be reverted and is always successful.
#[inline]
pub fn end<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
    context: &mut Context<EvmWiringT>,
    evm_output: EVMResult<EvmWiringT>,
) -> EVMResult<EvmWiringT> {
    evm_output.or_else(|err| {
        if matches!(err, EVMError::Transaction(_))
            && context.evm.inner.env().tx.source_hash().is_some()
        {
            // If the transaction is a deposit transaction and it failed
            // for any reason, the caller nonce must be bumped, and the
            // gas reported must be altered depending on the Hardfork. This is
            // also returned as a special Halt variant so that consumers can more
            // easily distinguish between a failed deposit and a failed
            // normal transaction.
            let caller = *context.evm.inner.env().tx.caller();

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
                    context.evm.inner.env().tx.mint().cloned().unwrap_or(0),
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
                .is_system_transaction()
                .unwrap_or(false);
            let gas_used = if SPEC::optimism_enabled(OptimismSpecId::REGOLITH) || !is_system_tx {
                context.evm.inner.env().tx.gas_limit()
            } else {
                0
            };

            Ok(ResultAndState {
                result: ExecutionResult::Halt {
                    reason: OptimismHaltReason::FailedDeposit,
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
    use super::*;
    use crate::{BedrockSpec, L1BlockInfo, LatestSpec, OptimismEvmWiring, RegolithSpec};
    use revm::{
        db::{EmptyDB, InMemoryDB},
        interpreter::{CallOutcome, InterpreterResult},
        primitives::{bytes, state::AccountInfo, Address, Bytes, B256},
    };
    use std::boxed::Box;

    type TestEmptyOpWiring = OptimismEvmWiring<EmptyDB, ()>;
    type TestMemOpWiring = OptimismEvmWiring<InMemoryDB, ()>;

    /// Creates frame result.
    fn call_last_frame_return<SPEC>(
        env: EnvWiring<TestEmptyOpWiring>,
        instruction_result: InstructionResult,
        gas: Gas,
    ) -> Gas
    where
        SPEC: OptimismSpec,
    {
        let mut ctx = Context::<TestEmptyOpWiring>::new_with_db(EmptyDB::default());
        ctx.evm.inner.env = Box::new(env);
        let mut first_frame = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));
        last_frame_return::<TestEmptyOpWiring, SPEC>(&mut ctx, &mut first_frame).unwrap();
        refund::<TestEmptyOpWiring, SPEC>(&mut ctx, first_frame.gas_mut(), 0);
        *first_frame.gas()
    }

    #[test]
    fn test_revert_gas() {
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.base.gas_limit = 100;
        env.tx.source_hash = None;

        let gas =
            call_last_frame_return::<BedrockSpec>(env, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas() {
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.base.gas_limit = 100;
        env.tx.source_hash = Some(B256::ZERO);

        let gas =
            call_last_frame_return::<RegolithSpec>(env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.base.gas_limit = 100;
        env.tx.source_hash = Some(B256::ZERO);

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
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.base.gas_limit = 100;
        env.tx.source_hash = Some(B256::ZERO);

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

        let mut context = Context::<TestMemOpWiring>::new_with_db(db);
        *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // Enveloped needs to be some but it will deduce zero fee.
        context.evm.inner.env.tx.enveloped_tx = Some(bytes!(""));
        // added mint value is 10.
        context.evm.inner.env.tx.mint = Some(10);

        deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

        // Check the account balance is updated.
        let account = context
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
        let mut context = Context::<TestMemOpWiring>::new_with_db(db);
        *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.enveloped_tx = Some(bytes!("FACADE"));
        // added mint value is 10.
        context.evm.inner.env.tx.mint = Some(10);
        // Putting source_hash to some makes it a deposit transaction.
        // so enveloped_tx gas cost is ignored.
        context.evm.inner.env.tx.source_hash = Some(B256::ZERO);

        deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

        // Check the account balance is updated.
        let account = context
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
        let mut context = Context::<TestMemOpWiring>::new_with_db(db);
        *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.enveloped_tx = Some(bytes!("FACADE"));
        deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

        // Check the account balance is updated.
        let account = context
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
        let mut context = Context::<TestMemOpWiring>::new_with_db(db);
        *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
            l1_base_fee: U256::from(1_000),
            l1_fee_overhead: Some(U256::from(1_000)),
            l1_base_fee_scalar: U256::from(1_000),
            ..Default::default()
        });
        // l1block cost is 1048 fee.
        context.evm.inner.env.tx.enveloped_tx = Some(bytes!("FACADE"));

        assert_eq!(
            deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context),
            Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: Box::new(U256::from(1048)),
                    balance: Box::new(U256::from(48)),
                }
                .into(),
            ))
        );
    }

    #[test]
    fn test_validate_sys_tx() {
        // mark the tx as a system transaction.
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.is_system_transaction = Some(true);
        assert_eq!(
            validate_env::<TestEmptyOpWiring, RegolithSpec>(&env),
            Err(EVMError::Transaction(
                OptimismInvalidTransaction::DepositSystemTxPostRegolith
            ))
        );

        // Pre-regolith system transactions should be allowed.
        assert!(validate_env::<TestEmptyOpWiring, BedrockSpec>(&env).is_ok());
    }

    #[test]
    fn test_validate_deposit_tx() {
        // Set source hash.
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.source_hash = Some(B256::ZERO);
        assert!(validate_env::<TestEmptyOpWiring, RegolithSpec>(&env).is_ok());
    }

    #[test]
    fn test_validate_tx_against_state_deposit_tx() {
        // Set source hash.
        let mut env = EnvWiring::<TestEmptyOpWiring>::default();
        env.tx.source_hash = Some(B256::ZERO);

        // Nonce and balance checks should be skipped for deposit transactions.
        assert!(validate_env::<TestEmptyOpWiring, LatestSpec>(&env).is_ok());
    }
}
