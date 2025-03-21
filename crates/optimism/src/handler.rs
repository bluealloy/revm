//!Handler related to Optimism chain
use crate::{
    api::exec::OpContextTr,
    constants::{BASE_FEE_RECIPIENT, L1_FEE_RECIPIENT, OPERATOR_FEE_RECIPIENT},
    transaction::{deposit::DEPOSIT_TRANSACTION_TYPE, OpTransactionError, OpTxTr},
    L1BlockInfo, OpHaltReason, OpSpecId,
};
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, FromStringError, ResultAndState},
        Block, Cfg, ContextTr, JournalTr, Transaction,
    },
    handler::{
        handler::EvmTrError, validation::validate_tx_against_account, EvmTr, Frame, FrameResult,
        Handler, MainnetHandler,
    },
    inspector::{Inspector, InspectorEvmTr, InspectorFrame, InspectorHandler},
    interpreter::{interpreter::EthInterpreter, FrameInput, Gas},
    primitives::hardfork::SpecId,
    primitives::{HashMap, U256},
    state::Account,
    Database,
};

pub struct OpHandler<EVM, ERROR, FRAME> {
    pub mainnet: MainnetHandler<EVM, ERROR, FRAME>,
    pub _phantom: core::marker::PhantomData<(EVM, ERROR, FRAME)>,
}

impl<EVM, ERROR, FRAME> OpHandler<EVM, ERROR, FRAME> {
    pub fn new() -> Self {
        Self {
            mainnet: MainnetHandler::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM, ERROR, FRAME> Default for OpHandler<EVM, ERROR, FRAME> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IsTxError {
    fn is_tx_error(&self) -> bool;
}

impl<DB, TX> IsTxError for EVMError<DB, TX> {
    fn is_tx_error(&self) -> bool {
        matches!(self, EVMError::Transaction(_))
    }
}

impl<EVM, ERROR, FRAME> Handler for OpHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: OpContextTr>,
    ERROR: EvmTrError<EVM> + From<OpTransactionError> + FromStringError + IsTxError,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
{
    type Evm = EVM;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = OpHaltReason;

    fn validate_env(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        // Do not perform any extra validation for deposit transactions, they are pre-verified on L1.
        let ctx = evm.ctx();
        let tx = ctx.tx();
        let tx_type = tx.tx_type();
        if tx_type == DEPOSIT_TRANSACTION_TYPE {
            // Do not allow for a system transaction to be processed if Regolith is enabled.
            if tx.is_system_transaction()
                && evm.ctx().cfg().spec().is_enabled_in(OpSpecId::REGOLITH)
            {
                return Err(OpTransactionError::DepositSystemTxPostRegolith.into());
            }
            return Ok(());
        }
        self.mainnet.validate_env(evm)
    }

    fn validate_tx_against_state(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let spec = context.cfg().spec();
        let block_number = context.block().number();
        if context.tx().tx_type() == DEPOSIT_TRANSACTION_TYPE {
            return Ok(());
        } else {
            // The L1-cost fee is only computed for Optimism non-deposit transactions.
            if context.chain().l2_block != block_number {
                // L1 block info is stored in the context for later use.
                // and it will be reloaded from the database if it is not for the current block.
                *context.chain() = L1BlockInfo::try_fetch(context.db(), block_number, spec)?;
            }
        }

        let enveloped_tx = context
            .tx()
            .enveloped_tx()
            .expect("all not deposit tx have enveloped tx")
            .clone();

        // compute L1 cost
        let mut additional_cost = context.chain().calculate_tx_l1_cost(&enveloped_tx, spec);

        if spec.is_enabled_in(OpSpecId::ISTHMUS) {
            let gas_limit = U256::from(context.tx().gas_limit());
            let operator_fee_charge = context
                .chain()
                .operator_fee_charge(&enveloped_tx, gas_limit);

            additional_cost = additional_cost.saturating_add(operator_fee_charge);
        }

        let tx_caller = context.tx().caller();

        // Load acc
        let account = context.journal().load_account_code(tx_caller)?;
        let account = account.data.info.clone();

        validate_tx_against_account(&account, context, additional_cost)?;
        Ok(())
    }

    fn deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        let ctx = evm.ctx();
        let spec = ctx.cfg().spec();
        let caller = ctx.tx().caller();
        let is_deposit = ctx.tx().tx_type() == DEPOSIT_TRANSACTION_TYPE;

        // If the transaction is a deposit with a `mint` value, add the mint value
        // in wei to the caller's balance. This should be persisted to the database
        // prior to the rest of execution.
        let mut tx_l1_cost = U256::ZERO;
        if is_deposit {
            let tx = ctx.tx();
            if let Some(mint) = tx.mint() {
                let mut caller_account = ctx.journal().load_account(caller)?;
                caller_account.info.balance += U256::from(mint);
            }
        } else {
            let enveloped_tx = ctx
                .tx()
                .enveloped_tx()
                .expect("all not deposit tx have enveloped tx")
                .clone();
            tx_l1_cost = ctx.chain().calculate_tx_l1_cost(&enveloped_tx, spec);
        }

        // We deduct caller max balance after minting and before deducing the
        // L1 cost, max values is already checked in pre_validate but L1 cost wasn't.
        self.mainnet.deduct_caller(evm)?;

        // If the transaction is not a deposit transaction, subtract the L1 data fee from the
        // caller's balance directly after minting the requested amount of ETH.
        // Additionally deduct the operator fee from the caller's account.
        if !is_deposit {
            let ctx = evm.ctx();

            // Deduct the operator fee from the caller's account.
            let gas_limit = U256::from(ctx.tx().gas_limit());
            let enveloped_tx = ctx
                .tx()
                .enveloped_tx()
                .expect("all not deposit tx have enveloped tx")
                .clone();

            let mut operator_fee_charge = U256::ZERO;
            if spec.is_enabled_in(OpSpecId::ISTHMUS) {
                operator_fee_charge = ctx.chain().operator_fee_charge(&enveloped_tx, gas_limit);
            }

            let mut caller_account = ctx.journal().load_account(caller)?;
            caller_account.info.balance = caller_account
                .info
                .balance
                .saturating_sub(tx_l1_cost.saturating_add(operator_fee_charge));
        }
        Ok(())
    }

    fn last_frame_result(
        &self,
        evm: &mut Self::Evm,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let ctx = evm.ctx();
        let tx = ctx.tx();
        let is_deposit = tx.tx_type() == DEPOSIT_TRANSACTION_TYPE;
        let tx_gas_limit = tx.gas_limit();
        let is_regolith = ctx.cfg().spec().is_enabled_in(OpSpecId::REGOLITH);

        let instruction_result = frame_result.interpreter_result().result;
        let gas = frame_result.gas_mut();
        let remaining = gas.remaining();
        let refunded = gas.refunded();

        // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
        *gas = Gas::new_spent(tx_gas_limit);

        if instruction_result.is_ok() {
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
            } else if is_deposit {
                let tx = ctx.tx();
                if tx.is_system_transaction() {
                    // System transactions were a special type of deposit transaction in
                    // the Bedrock hardfork that did not incur any gas costs.
                    gas.erase_cost(tx_gas_limit);
                }
            }
        } else if instruction_result.is_revert() {
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
        Ok(())
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        self.mainnet.reimburse_caller(evm, exec_result)?;

        let context = evm.ctx();
        if matches!(context.tx().source_hash(), Some(source_hash) if source_hash.is_zero()) {
            let caller = context.tx().caller();
            let spec = context.cfg().spec();
            let operator_fee_refund = context.chain().operator_fee_refund(exec_result.gas(), spec);

            let caller_account = context.journal().load_account(caller)?;

            // In additional to the normal transaction fee, additionally refund the caller
            // for the operator fee.
            caller_account.data.info.balance = caller_account
                .data
                .info
                .balance
                .saturating_add(operator_fee_refund);
        }

        Ok(())
    }

    fn refund(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        eip7702_refund: i64,
    ) {
        exec_result.gas_mut().record_refund(eip7702_refund);

        let is_deposit = evm.ctx().tx().tx_type() == DEPOSIT_TRANSACTION_TYPE;
        let is_regolith = evm.ctx().cfg().spec().is_enabled_in(OpSpecId::REGOLITH);

        // Prior to Regolith, deposit transactions did not receive gas refunds.
        let is_gas_refund_disabled = is_deposit && !is_regolith;
        if !is_gas_refund_disabled {
            exec_result.gas_mut().set_final_refund(
                evm.ctx()
                    .cfg()
                    .spec()
                    .into_eth_spec()
                    .is_enabled_in(SpecId::LONDON),
            );
        }
    }

    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let is_deposit = evm.ctx().tx().tx_type() == DEPOSIT_TRANSACTION_TYPE;

        // Transfer fee to coinbase/beneficiary.
        if !is_deposit {
            self.mainnet.reward_beneficiary(evm, exec_result)?;
            let basefee = evm.ctx().block().basefee() as u128;

            // If the transaction is not a deposit transaction, fees are paid out
            // to both the Base Fee Vault as well as the L1 Fee Vault.
            let ctx = evm.ctx();
            let enveloped = ctx.tx().enveloped_tx().cloned();
            let spec = ctx.cfg().spec();
            let l1_block_info = ctx.chain();

            let Some(enveloped_tx) = &enveloped else {
                return Err(ERROR::from_string(
                    "[OPTIMISM] Failed to load enveloped transaction.".into(),
                ));
            };

            let l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, spec);
            let mut operator_fee_cost = U256::ZERO;
            if spec.is_enabled_in(OpSpecId::ISTHMUS) {
                operator_fee_cost = l1_block_info.operator_fee_charge(
                    enveloped_tx,
                    U256::from(exec_result.gas().spent() - exec_result.gas().refunded() as u64),
                );
            }
            // Send the L1 cost of the transaction to the L1 Fee Vault.
            let mut l1_fee_vault_account = ctx.journal().load_account(L1_FEE_RECIPIENT)?;
            l1_fee_vault_account.mark_touch();
            l1_fee_vault_account.info.balance += l1_cost;

            // Send the base fee of the transaction to the Base Fee Vault.
            let mut base_fee_vault_account =
                evm.ctx().journal().load_account(BASE_FEE_RECIPIENT)?;
            base_fee_vault_account.mark_touch();
            base_fee_vault_account.info.balance += U256::from(basefee.saturating_mul(
                (exec_result.gas().spent() - exec_result.gas().refunded() as u64) as u128,
            ));

            // Send the operator fee of the transaction to the coinbase.
            let mut operator_fee_vault_account =
                evm.ctx().journal().load_account(OPERATOR_FEE_RECIPIENT)?;
            operator_fee_vault_account.mark_touch();
            operator_fee_vault_account.data.info.balance += operator_fee_cost;
        }
        Ok(())
    }

    fn output(
        &self,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let result = self.mainnet.output(evm, result)?;
        let result = result.map_haltreason(OpHaltReason::Base);
        if result.result.is_halt() {
            // Post-regolith, if the transaction is a deposit transaction and it halts,
            // we bubble up to the global return handler. The mint value will be persisted
            // and the caller nonce will be incremented there.
            let is_deposit = evm.ctx().tx().tx_type() == DEPOSIT_TRANSACTION_TYPE;
            if is_deposit && evm.ctx().cfg().spec().is_enabled_in(OpSpecId::REGOLITH) {
                return Err(ERROR::from(OpTransactionError::HaltedDepositPostRegolith));
            }
        }
        evm.ctx().chain().clear_tx_l1_cost();
        Ok(result)
    }

    fn catch_error(
        &self,
        evm: &mut Self::Evm,
        error: Self::Error,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let is_deposit = evm.ctx().tx().tx_type() == DEPOSIT_TRANSACTION_TYPE;
        let output = if error.is_tx_error() && is_deposit {
            let ctx = evm.ctx();
            let spec = ctx.cfg().spec();
            let tx = ctx.tx();
            let caller = tx.caller();
            let mint = tx.mint();
            let is_system_tx = tx.is_system_transaction();
            let gas_limit = tx.gas_limit();
            // If the transaction is a deposit transaction and it failed
            // for any reason, the caller nonce must be bumped, and the
            // gas reported must be altered depending on the Hardfork. This is
            // also returned as a special Halt variant so that consumers can more
            // easily distinguish between a failed deposit and a failed
            // normal transaction.

            // Increment sender nonce and account balance for the mint amount. Deposits
            // always persist the mint amount, even if the transaction fails.
            let account = {
                let mut acc = Account::from(
                    evm.ctx()
                        .db()
                        .basic(caller)
                        .unwrap_or_default()
                        .unwrap_or_default(),
                );
                acc.info.nonce = acc.info.nonce.saturating_add(1);
                acc.info.balance = acc
                    .info
                    .balance
                    .saturating_add(U256::from(mint.unwrap_or_default()));
                acc.mark_touch();
                acc
            };
            let state = HashMap::from_iter([(caller, account)]);

            // The gas used of a failed deposit post-regolith is the gas
            // limit of the transaction. pre-regolith, it is the gas limit
            // of the transaction for non system transactions and 0 for system
            // transactions.
            let gas_used = if spec.is_enabled_in(OpSpecId::REGOLITH) || !is_system_tx {
                gas_limit
            } else {
                0
            };
            // clear the journal
            Ok(ResultAndState {
                result: ExecutionResult::Halt {
                    reason: OpHaltReason::FailedDeposit,
                    gas_used,
                },
                state,
            })
        } else {
            Err(error)
        };
        // do cleanup
        evm.ctx().chain().clear_tx_l1_cost();
        evm.ctx().journal().clear();

        output
    }
}

impl<EVM, ERROR, FRAME> InspectorHandler for OpHandler<EVM, ERROR, FRAME>
where
    EVM: InspectorEvmTr<
        Context: OpContextTr,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM> + From<OpTransactionError> + FromStringError + IsTxError,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: InspectorFrame<
        Evm = EVM,
        Error = ERROR,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
        IT = EthInterpreter,
    >,
{
    type IT = EthInterpreter;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::default_ctx::OpContext, DefaultOp, OpBuilder};
    use revm::{
        context::Context,
        context_interface::result::InvalidTransaction,
        database::InMemoryDB,
        database_interface::EmptyDB,
        handler::EthFrame,
        interpreter::{CallOutcome, InstructionResult, InterpreterResult},
        primitives::{bytes, Address, Bytes, B256},
        state::AccountInfo,
    };
    use std::boxed::Box;

    /// Creates frame result.
    fn call_last_frame_return(
        ctx: OpContext<EmptyDB>,
        instruction_result: InstructionResult,
        gas: Gas,
    ) -> Gas {
        let mut evm = ctx.build_op();

        let mut exec_result = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));

        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        handler
            .last_frame_result(&mut evm, &mut exec_result)
            .unwrap();
        handler.refund(&mut evm, &mut exec_result, 0);
        *exec_result.gas()
    }

    #[test]
    fn test_revert_gas() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.enveloped_tx = None;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::BEDROCK);

        let gas = call_last_frame_return(ctx, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.deposit.source_hash = B256::ZERO;
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let gas = call_last_frame_return(ctx, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.deposit.source_hash = B256::ZERO;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let mut ret_gas = Gas::new(90);
        ret_gas.record_refund(20);

        let gas = call_last_frame_return(ctx.clone(), InstructionResult::Stop, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 2); // min(20, 10/5)

        let gas = call_last_frame_return(ctx, InstructionResult::Revert, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_deposit_tx() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.base.gas_limit = 100;
                tx.deposit.source_hash = B256::ZERO;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::BEDROCK);
        let gas = call_last_frame_return(ctx, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 0);
        assert_eq!(gas.spent(), 100);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_sys_deposit_tx() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.base.gas_limit = 100;
                tx.deposit.source_hash = B256::ZERO;
                tx.deposit.is_system_transaction = true;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::BEDROCK);
        let gas = call_last_frame_return(ctx, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 100);
        assert_eq!(gas.spent(), 0);
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

        let mut ctx = Context::op()
            .with_db(db)
            .with_chain(L1BlockInfo {
                l1_base_fee: U256::from(1_000),
                l1_fee_overhead: Some(U256::from(1_000)),
                l1_base_fee_scalar: U256::from(1_000),
                ..Default::default()
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);
        ctx.modify_tx(|tx| {
            tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
            tx.deposit.source_hash = B256::ZERO;
            tx.deposit.mint = Some(10);
        });

        let mut evm = ctx.build_op();

        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();
        handler.deduct_caller(&mut evm).unwrap();

        // Check the account balance is updated.
        let account = evm.ctx().journal().load_account(caller).unwrap();
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
        let ctx = Context::op()
            .with_db(db)
            .with_chain(L1BlockInfo {
                l1_base_fee: U256::from(1_000),
                l1_fee_overhead: Some(U256::from(1_000)),
                l1_base_fee_scalar: U256::from(1_000),
                ..Default::default()
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH)
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.deposit.mint = Some(10);
                tx.enveloped_tx = Some(bytes!("FACADE"));
                tx.deposit.source_hash = B256::ZERO;
            });

        let mut evm = ctx.build_op();

        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();
        handler.deduct_caller(&mut evm).unwrap();

        // Check the account balance is updated.
        let account = evm.ctx().journal().load_account(caller).unwrap();
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
        let ctx = Context::op()
            .with_db(db)
            .with_chain(L1BlockInfo {
                l1_base_fee: U256::from(1_000),
                l1_fee_overhead: Some(U256::from(1_000)),
                l1_base_fee_scalar: U256::from(1_000),
                ..Default::default()
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH)
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.deposit.source_hash = B256::ZERO;
                tx.enveloped_tx = Some(bytes!("FACADE"));
            });

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        // l1block cost is 1048 fee.
        handler.deduct_caller(&mut evm).unwrap();

        // Check the account balance is updated.
        let account = evm.ctx().journal().load_account(caller).unwrap();
        assert_eq!(account.info.balance, U256::from(1));
    }

    #[test]
    fn test_remove_operator_cost() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(151),
                ..Default::default()
            },
        );
        let ctx = Context::op()
            .with_db(db)
            .with_chain(L1BlockInfo {
                operator_fee_scalar: Some(U256::from(10_000_000)),
                operator_fee_constant: Some(U256::from(50)),
                ..Default::default()
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::ISTHMUS)
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 10;
                tx.enveloped_tx = Some(bytes!("FACADE"));
            });

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        // operator fee cost is operator_fee_scalar * gas_limit / 1e6 + operator_fee_constant
        // 10_000_000 * 10 / 1_000_000 + 50 = 150
        handler.deduct_caller(&mut evm).unwrap();

        // Check the account balance is updated.
        let account = evm.ctx().journal().load_account(caller).unwrap();
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
        let ctx = Context::op()
            .with_db(db)
            .with_chain(L1BlockInfo {
                l1_base_fee: U256::from(1_000),
                l1_fee_overhead: Some(U256::from(1_000)),
                l1_base_fee_scalar: U256::from(1_000),
                ..Default::default()
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH)
            .modify_tx_chained(|tx| {
                tx.enveloped_tx = Some(bytes!("FACADE"));
            });

        // l1block cost is 1048 fee.
        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        // l1block cost is 1048 fee.
        assert_eq!(
            handler.validate_tx_against_state(&mut evm),
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
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.deposit.is_system_transaction = true;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        assert_eq!(
            handler.validate_env(&mut evm),
            Err(EVMError::Transaction(
                OpTransactionError::DepositSystemTxPostRegolith
            ))
        );

        evm.ctx().modify_cfg(|cfg| cfg.spec = OpSpecId::BEDROCK);

        // Pre-regolith system transactions should be allowed.
        assert!(handler.validate_env(&mut evm).is_ok());
    }

    #[test]
    fn test_validate_deposit_tx() {
        // Set source hash.
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.deposit.source_hash = B256::ZERO;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        assert!(handler.validate_env(&mut evm).is_ok());
    }

    #[test]
    fn test_validate_tx_against_state_deposit_tx() {
        // Set source hash.
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
                tx.deposit.source_hash = B256::ZERO;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        // Nonce and balance checks should be skipped for deposit transactions.
        assert!(handler.validate_env(&mut evm).is_ok());
    }

    #[test]
    fn test_halted_deposit_tx_post_regolith() {
        let ctx = Context::op()
            .modify_tx_chained(|tx| {
                tx.base.tx_type = DEPOSIT_TRANSACTION_TYPE;
            })
            .modify_cfg_chained(|cfg| cfg.spec = OpSpecId::REGOLITH);

        let mut evm = ctx.build_op();
        let handler = OpHandler::<_, EVMError<_, OpTransactionError>, EthFrame<_, _, _>>::new();

        assert_eq!(
            handler.output(
                &mut evm,
                FrameResult::Call(CallOutcome {
                    result: InterpreterResult {
                        result: InstructionResult::OutOfGas,
                        output: Default::default(),
                        gas: Default::default(),
                    },
                    memory_offset: Default::default(),
                })
            ),
            Err(EVMError::Transaction(
                OpTransactionError::HaltedDepositPostRegolith
            ))
        )
    }
}
