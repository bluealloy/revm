//! Handler related to Optimism chain

use crate::{
    transaction::{
        abstraction::OpTxGetter, deposit::DepositTransaction, OpTransactionType, OpTxTrait,
    },
    L1BlockInfoGetter, OpSpec, OpSpecId, OpTransactionError, OptimismHaltReason,
    BASE_FEE_RECIPIENT, L1_FEE_RECIPIENT,
};
use core::ops::Mul;
use revm::{
    context_interface::{
        result::{ExecutionResult, FromStringError, InvalidTransaction, ResultAndState},
        transaction::CommonTxFields,
        Block, Cfg, CfgGetter, DatabaseGetter, JournaledState, Transaction, TransactionGetter,
    },
    handler::{
        EthExecution, EthExecutionContext, EthExecutionError, EthFrame, EthFrameContext,
        EthFrameError, EthPostExecution, EthPostExecutionContext, EthPostExecutionError,
        EthPreExecution, EthPreExecutionContext, EthPreExecutionError, EthPrecompileProvider,
        EthValidation, EthValidationContext, EthValidationError, FrameResult,
    },
    handler_interface::{
        util::FrameOrFrameResult, ExecutionHandler, Frame, PostExecutionHandler,
        PreExecutionHandler, ValidationHandler,
    },
    interpreter::{
        interpreter::{EthInstructionProvider, EthInterpreter},
        Gas,
    },
    primitives::{hash_map::HashMap, U256},
    specification::hardfork::SpecId,
    state::Account,
    Database,
};

pub struct OpValidationHandler<CTX, ERROR> {
    pub eth: EthValidation<CTX, ERROR>,
}

impl<CTX, ERROR> ValidationHandler for OpValidationHandler<CTX, ERROR>
where
    CTX: EthValidationContext + OpTxGetter,
    // Have Cfg with OpSpec
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    // Have transaction with OpTransactionType
    <CTX as TransactionGetter>::Transaction: Transaction<TransactionType = OpTransactionType>,
    // Add additional error type.
    ERROR: EthValidationError<CTX> + From<OpTransactionError>,
{
    type Context = CTX;
    type Error = ERROR;

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        // Do not perform any extra validation for deposit transactions, they are pre-verified on L1.
        let tx_type = context.tx().tx_type();
        if tx_type == OpTransactionType::Deposit {
            let tx = context.op_tx().deposit();
            // Do not allow for a system transaction to be processed if Regolith is enabled.
            // TODO check if this is correct.
            if tx.is_system_transaction() && context.cfg().spec().is_enabled_in(OpSpecId::REGOLITH)
            {
                return Err(OpTransactionError::DepositSystemTxPostRegolith.into());
            }
            return Ok(());
        }
        self.eth.validate_env(context)
    }

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        if context.tx().tx_type() == OpTransactionType::Deposit {
            return Ok(());
        }
        self.eth.validate_tx_against_state(context)
    }

    /// Validate initial gas.
    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error> {
        self.eth.validate_initial_tx_gas(context)
    }
}

pub struct OpPreExecution<CTX, ERROR> {
    pub eth: EthPreExecution<CTX, ERROR>,
}

impl<CTX, ERROR> PreExecutionHandler for OpPreExecution<CTX, ERROR>
where
    CTX: EthPreExecutionContext + DatabaseGetter + OpTxGetter + L1BlockInfoGetter,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    <CTX as TransactionGetter>::Transaction: Transaction<TransactionType = OpTransactionType>,
    ERROR: EthPreExecutionError<CTX> + From<<<CTX as DatabaseGetter>::Database as Database>::Error>,
{
    type Context = CTX;
    type Error = ERROR;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        // the L1-cost fee is only computed for Optimism non-deposit transactions.
        let spec = context.cfg().spec().into();
        if context.tx().tx_type() != OpTransactionType::Deposit {
            let l1_block_info: crate::L1BlockInfo =
                super::L1BlockInfo::try_fetch(context.db(), spec)?;

            // storage l1 block info for later use.
            *context.l1_block_info_mut() = l1_block_info;
        }

        self.eth.load_accounts(context)
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        self.eth.apply_eip7702_auth_list(context)
    }

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let caller = context.tx().common_fields().caller();
        let is_deposit = context.tx().tx_type() == OpTransactionType::Deposit;

        // If the transaction is a deposit with a `mint` value, add the mint value
        // in wei to the caller's balance. This should be persisted to the database
        // prior to the rest of execution.
        let mut tx_l1_cost = U256::ZERO;
        if is_deposit {
            let tx = context.op_tx().deposit();
            if let Some(mint) = tx.mint() {
                let mut caller_account = context.journal().load_account(caller)?;
                caller_account.info.balance += U256::from(mint);
            }
        } else {
            let enveloped_tx = context
                .op_tx()
                .enveloped_tx()
                .expect("all not deposit tx have enveloped tx")
                .clone();
            tx_l1_cost = context
                .l1_block_info()
                .calculate_tx_l1_cost(&enveloped_tx, context.cfg().spec());
        }

        // We deduct caller max balance after minting and before deducing the
        // l1 cost, max values is already checked in pre_validate but l1 cost wasn't.
        self.eth.deduct_caller(context)?;

        // If the transaction is not a deposit transaction, subtract the L1 data fee from the
        // caller's balance directly after minting the requested amount of ETH.
        if !is_deposit {
            let mut caller_account = context.journal().load_account(caller)?;

            if tx_l1_cost > caller_account.info.balance {
                return Err(InvalidTransaction::LackOfFundForMaxFee {
                    fee: tx_l1_cost.into(),
                    balance: caller_account.info.balance.into(),
                }
                .into());
            }
            caller_account.info.balance = caller_account.info.balance.saturating_sub(tx_l1_cost);
        }
        Ok(())
    }
}

pub struct OpExecution<CTX, ERROR> {
    pub eth: EthExecution<CTX, ERROR>,
}

impl<CTX, ERROR> ExecutionHandler for OpExecution<CTX, ERROR>
where
    CTX: EthExecutionContext<ERROR> + EthFrameContext<ERROR> + OpTxGetter,
    ERROR: EthExecutionError<CTX> + EthFrameError<CTX>,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    <CTX as TransactionGetter>::Transaction: Transaction<TransactionType = OpTransactionType>,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = EthFrame<
        CTX,
        ERROR,
        EthInterpreter<()>,
        EthPrecompileProvider<CTX, ERROR>,
        EthInstructionProvider<EthInterpreter, CTX>,
    >;
    type ExecResult = FrameResult;

    fn init_first_frame(
        &mut self,
        context: &mut Self::Context,
        gas_limit: u64,
    ) -> Result<FrameOrFrameResult<Self::Frame>, Self::Error> {
        self.eth.init_first_frame(context, gas_limit)
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        mut frame_result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error> {
        let tx = context.tx();
        let is_deposit = tx.tx_type() == OpTransactionType::Deposit;
        let tx_gas_limit = tx.common_fields().gas_limit();
        let is_regolith = context.cfg().spec().is_enabled_in(OpSpecId::REGOLITH);

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
                let tx = context.op_tx().deposit();
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
        Ok(frame_result)
    }
}

pub struct OpPostExecution<CTX, ERROR> {
    pub eth: EthPostExecution<CTX, ERROR, OptimismHaltReason>,
}

pub trait IsTxError {
    fn is_tx_error(&self) -> bool;
}

impl<CTX, ERROR> PostExecutionHandler for OpPostExecution<CTX, ERROR>
where
    CTX: EthPostExecutionContext<ERROR> + OpTxGetter + L1BlockInfoGetter + DatabaseGetter,
    ERROR: EthPostExecutionError<CTX>
        + EthFrameError<CTX>
        + From<OpTransactionError>
        + FromStringError
        + IsTxError,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    <CTX as TransactionGetter>::Transaction: Transaction<TransactionType = OpTransactionType>,
{
    type Context = CTX;
    type Error = ERROR;
    type ExecResult = FrameResult;
    type Output = ResultAndState<OptimismHaltReason>;

    fn refund(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
        eip7702_refund: i64,
    ) {
        exec_result.gas_mut().record_refund(eip7702_refund);

        let is_deposit = context.tx().tx_type() == OpTransactionType::Deposit;
        let is_regolith = context.cfg().spec().is_enabled_in(OpSpecId::REGOLITH);

        // Prior to Regolith, deposit transactions did not receive gas refunds.
        let is_gas_refund_disabled = is_deposit && !is_regolith;
        if !is_gas_refund_disabled {
            exec_result
                .gas_mut()
                .set_final_refund(context.cfg().spec().is_enabled_in(SpecId::LONDON));
        }
    }

    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        self.eth.reimburse_caller(context, exec_result)
    }

    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        self.eth.reward_beneficiary(context, exec_result)?;

        let is_deposit = context.tx().tx_type() == OpTransactionType::Deposit;

        // transfer fee to coinbase/beneficiary.
        if !is_deposit {
            self.eth.reward_beneficiary(context, exec_result)?;
            let basefee = *context.block().basefee();

            // If the transaction is not a deposit transaction, fees are paid out
            // to both the Base Fee Vault as well as the L1 Fee Vault.
            let l1_block_info = context.l1_block_info();

            let Some(enveloped_tx) = &context.op_tx().enveloped_tx() else {
                return Err(ERROR::from_string(
                    "[OPTIMISM] Failed to load enveloped transaction.".into(),
                ));
            };

            let l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, context.cfg().spec());

            // Send the L1 cost of the transaction to the L1 Fee Vault.
            let mut l1_fee_vault_account = context.journal().load_account(L1_FEE_RECIPIENT)?;
            l1_fee_vault_account.mark_touch();
            l1_fee_vault_account.info.balance += l1_cost;

            // Send the base fee of the transaction to the Base Fee Vault.
            let mut base_fee_vault_account = context.journal().load_account(BASE_FEE_RECIPIENT)?;
            base_fee_vault_account.mark_touch();
            base_fee_vault_account.info.balance += basefee.mul(U256::from(
                exec_result.gas().spent() - exec_result.gas().refunded() as u64,
            ));
        }
        Ok(())
    }

    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error> {
        let result = self.eth.output(context, result)?;
        if result.result.is_halt() {
            // Post-regolith, if the transaction is a deposit transaction and it halts,
            // we bubble up to the global return handler. The mint value will be persisted
            // and the caller nonce will be incremented there.
            let is_deposit = context.tx().tx_type() == OpTransactionType::Deposit;
            if is_deposit && context.cfg().spec().is_enabled_in(OpSpecId::REGOLITH) {
                return Err(ERROR::from(OpTransactionError::HaltedDepositPostRegolith));
            }
        }
        Ok(result)
    }

    fn clear(&self, context: &mut Self::Context) {
        self.eth.clear(context);
    }

    fn end(
        &self,
        context: &mut Self::Context,
        end_output: Result<Self::Output, Self::Error>,
    ) -> Result<Self::Output, Self::Error> {
        //end_output

        let is_deposit = context.tx().tx_type() == OpTransactionType::Deposit;
        end_output.or_else(|err| {
            if err.is_tx_error() && is_deposit {
                let spec = context.cfg().spec();
                let tx = context.op_tx().deposit();
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
                        context
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
}

// /// Load precompiles for Optimism chain.
// #[inline]
// pub fn load_precompiles<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
// ) -> ContextPrecompiles<EvmWiringT> {
//     let mut precompiles = ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID));

//     if SPEC::optimism_enabled(OptimismSpecId::FJORD) {
//         precompiles.extend([
//             // EIP-7212: secp256r1 P256verify
//             secp256r1::P256VERIFY,
//         ])
//     }

//     if SPEC::optimism_enabled(OptimismSpecId::GRANITE) {
//         precompiles.extend([
//             // Restrict bn256Pairing input size
//             crate::bn128::pair::GRANITE,
//         ])
//     }

//     precompiles
// }

// /// Optimism end handle changes output if the transaction is a deposit transaction.
// /// Deposit transaction can't be reverted and is always successful.
// #[inline]
// pub fn end<EvmWiringT: OptimismWiring, SPEC: OptimismSpec>(
//     context: &mut Context<EvmWiringT>,
//     evm_output: EVMResult<EvmWiringT>,
// ) -> EVMResult<EvmWiringT> {

// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         context_interface::OptimismEvmWiring, transaction::deposit::TxDeposit, BedrockSpec,
//         L1BlockInfo, LatestSpec, OpTransaction, RegolithSpec,
//     };
//     use database::InMemoryDB;
//     use revm::{
//         context_interface::default::{block::BlockEnv, Env, TxEnv},
//         database_interface::EmptyDB,
//         interpreter::{CallOutcome, InstructionResult, InterpreterResult},
//         primitives::{bytes, Address, Bytes, B256},
//         state::AccountInfo,
//     };
//     use std::boxed::Box;

//     type TestEmptyOpWiring = OptimismEvmWiring<EmptyDB, ()>;
//     type TestMemOpWiring = OptimismEvmWiring<InMemoryDB, ()>;

//     /// Creates frame result.
//     fn call_last_frame_return<SPEC>(
//         env: EnvWiring<TestEmptyOpWiring>,
//         instruction_result: InstructionResult,
//         gas: Gas,
//     ) -> Gas
//     where
//         SPEC: OptimismSpec,
//     {
//         let mut context = Context::<TestEmptyOpWiring>::new_with_db(EmptyDB::default());
//         context.evm.inner.env = Box::new(env);
//         let mut first_frame = FrameResult::Call(CallOutcome::new(
//             InterpreterResult {
//                 result: instruction_result,
//                 output: Bytes::new(),
//                 gas,
//             },
//             0..0,
//         ));
//         last_frame_return::<TestEmptyOpWiring, SPEC>(&mut context, &mut first_frame).unwrap();
//         refund::<TestEmptyOpWiring, SPEC>(&mut context, first_frame.gas_mut(), 0);
//         *first_frame.gas()
//     }

//     #[test]
//     fn test_revert_gas() {
//         let mut env = Envcontext_interface::<TestEmptyOpWiring>::default();
//         let tx = TxEnv {
//             gas_limit: 100,
//             ..Default::default()
//         };
//         env.tx = OpTransaction::Base {
//             tx,
//             enveloped_tx: None,
//         };

//         let gas =
//             call_last_frame_return::<BedrockSpec>(env, InstructionResult::Revert, Gas::new(90));
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_consume_gas() {
//         let mut env = Envcontext_interface::<TestEmptyOpWiring>::default();
//         //env.tx.base.gas_limit = 100;
//         //env.tx.source_hash = Some(B256::ZERO);

//         let deposit = TxDeposit {
//             gas_limit: 100,
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         env.tx = OpTransaction::Deposit(deposit);

//         let gas =
//             call_last_frame_return::<RegolithSpec>(env, InstructionResult::Stop, Gas::new(90));
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_consume_gas_with_refund() {
//         let mut env = Envcontext_interface::<TestEmptyOpWiring>::default();
//         //env.tx.base.gas_limit = 100;
//         //env.tx.source_hash = Some(B256::ZERO);
//         let deposit = TxDeposit {
//             gas_limit: 100,
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         env.tx = OpTransaction::Deposit(deposit);

//         let mut ret_gas = Gas::new(90);
//         ret_gas.record_refund(20);

//         let gas =
//             call_last_frame_return::<RegolithSpec>(env.clone(), InstructionResult::Stop, ret_gas);
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 2); // min(20, 10/5)

//         let gas = call_last_frame_return::<RegolithSpec>(env, InstructionResult::Revert, ret_gas);
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_consume_gas_sys_deposit_tx() {
//         let mut env = Envcontext_interface::<TestEmptyOpWiring>::default();
//         //env.tx.base.gas_limit = 100;
//         //env.tx.source_hash = Some(B256::ZERO);

//         let deposit = TxDeposit {
//             gas_limit: 100,
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         env.tx = OpTransaction::Deposit(deposit);

//         let gas = call_last_frame_return::<BedrockSpec>(env, InstructionResult::Stop, Gas::new(90));
//         assert_eq!(gas.remaining(), 0);
//         assert_eq!(gas.spent(), 100);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_commit_mint_value() {
//         let caller = Address::ZERO;
//         let mut db = InMemoryDB::default();
//         db.insert_account_info(
//             caller,
//             AccountInfo {
//                 balance: U256::from(1000),
//                 ..Default::default()
//             },
//         );

//         let mut context = Context::<TestMemOpWiring>::new_with_db(db);
//         *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
//             l1_base_fee: U256::from(1_000),
//             l1_fee_overhead: Some(U256::from(1_000)),
//             l1_base_fee_scalar: U256::from(1_000),
//             ..Default::default()
//         });
//         // // Enveloped needs to be some but it will deduce zero fee.
//         // context.evm.inner.env.tx.enveloped_tx = Some(bytes!(""));
//         // // added mint value is 10.
//         // context.evm.inner.env.tx.mint = Some(10);

//         let deposit = TxDeposit {
//             gas_limit: 100,
//             mint: Some(10),
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         context.evm.inner.env.tx = OpTransaction::Deposit(deposit);

//         deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

//         // Check the account balance is updated.
//         let account = context
//             .evm
//             .inner
//             .journaled_state
//             .load_account(caller, &mut context.evm.inner.db)
//             .unwrap();
//         assert_eq!(account.info.balance, U256::from(1010));
//     }

//     #[test]
//     fn test_remove_l1_cost_non_deposit() {
//         let caller = Address::ZERO;
//         let mut db = InMemoryDB::default();
//         db.insert_account_info(
//             caller,
//             AccountInfo {
//                 balance: U256::from(1000),
//                 ..Default::default()
//             },
//         );
//         let mut context = Context::<TestMemOpWiring>::new_with_db(db);
//         *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
//             l1_base_fee: U256::from(1_000),
//             l1_fee_overhead: Some(U256::from(1_000)),
//             l1_base_fee_scalar: U256::from(1_000),
//             ..Default::default()
//         });
//         // // l1block cost is 1048 fee.
//         // context.evm.inner.env.tx.enveloped_tx = Some(bytes!("FACADE"));
//         // // added mint value is 10.
//         // context.evm.inner.env.tx.mint = Some(10);
//         // // Putting source_hash to some makes it a deposit transaction.
//         // // so enveloped_tx gas cost is ignored.
//         // context.evm.inner.env.tx.source_hash = Some(B256::ZERO);

//         let deposit = TxDeposit {
//             mint: Some(10),
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         context.evm.inner.env.tx = OpTransaction::Deposit(deposit);

//         deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

//         // Check the account balance is updated.
//         let account = context
//             .evm
//             .inner
//             .journaled_state
//             .load_account(caller, &mut context.evm.inner.db)
//             .unwrap();
//         assert_eq!(account.info.balance, U256::from(1010));
//     }

//     #[test]
//     fn test_remove_l1_cost() {
//         let caller = Address::ZERO;
//         let mut db = InMemoryDB::default();
//         db.insert_account_info(
//             caller,
//             AccountInfo {
//                 balance: U256::from(1049),
//                 ..Default::default()
//             },
//         );
//         let mut context = Context::<TestMemOpWiring>::new_with_db(db);
//         *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
//             l1_base_fee: U256::from(1_000),
//             l1_fee_overhead: Some(U256::from(1_000)),
//             l1_base_fee_scalar: U256::from(1_000),
//             ..Default::default()
//         });
//         // l1block cost is 1048 fee.
//         context.evm.inner.env.tx = OpTransaction::Base {
//             tx: TxEnv::default(),
//             enveloped_tx: Some(bytes!("FACADE")),
//         };
//         deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context).unwrap();

//         // Check the account balance is updated.
//         let account = context
//             .evm
//             .inner
//             .journaled_state
//             .load_account(caller, &mut context.evm.inner.db)
//             .unwrap();
//         assert_eq!(account.info.balance, U256::from(1));
//     }

//     #[test]
//     fn test_remove_l1_cost_lack_of_funds() {
//         let caller = Address::ZERO;
//         let mut db = InMemoryDB::default();
//         db.insert_account_info(
//             caller,
//             AccountInfo {
//                 balance: U256::from(48),
//                 ..Default::default()
//             },
//         );
//         let mut context = Context::<TestMemOpWiring>::new_with_db(db);
//         *context.evm.chain.l1_block_info_mut() = Some(L1BlockInfo {
//             l1_base_fee: U256::from(1_000),
//             l1_fee_overhead: Some(U256::from(1_000)),
//             l1_base_fee_scalar: U256::from(1_000),
//             ..Default::default()
//         });
//         // l1block cost is 1048 fee.
//         context.evm.inner.env.tx = OpTransaction::Base {
//             tx: TxEnv::default(),
//             enveloped_tx: Some(bytes!("FACADE")),
//         };

//         assert_eq!(
//             deduct_caller::<TestMemOpWiring, RegolithSpec>(&mut context),
//             Err(EVMError::Transaction(
//                 InvalidTransaction::LackOfFundForMaxFee {
//                     fee: Box::new(U256::from(1048)),
//                     balance: Box::new(U256::from(48)),
//                 }
//                 .into(),
//             ))
//         );
//     }

//     #[test]
//     fn test_validate_sys_tx() {
//         // mark the tx as a system transaction.
//         // Set source hash.
//         let tx = TxDeposit {
//             is_system_transaction: true,
//             ..Default::default()
//         };
//         let env = Env::<BlockEnv, OpTransaction<TxEnv>> {
//             tx: OpTransaction::Deposit(tx),
//             ..Default::default()
//         };

//         assert_eq!(
//             validate_env::<TestEmptyOpWiring, RegolithSpec>(&env),
//             Err(EVMError::Transaction(
//                 OpTransactionError::DepositSystemTxPostRegolith
//             ))
//         );

//         // Pre-regolith system transactions should be allowed.
//         assert!(validate_env::<TestEmptyOpWiring, BedrockSpec>(&env).is_ok());
//     }

//     #[test]
//     fn test_validate_deposit_tx() {
//         // Set source hash.
//         let tx = TxDeposit {
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         let env = Env::<BlockEnv, OpTransaction<TxEnv>> {
//             tx: OpTransaction::Deposit(tx),
//             ..Default::default()
//         };
//         assert!(validate_env::<TestEmptyOpWiring, RegolithSpec>(&env).is_ok());
//     }

//     #[test]
//     fn test_validate_tx_against_state_deposit_tx() {
//         // Set source hash.
//         let tx = TxDeposit {
//             source_hash: B256::ZERO,
//             ..Default::default()
//         };
//         let env = Env::<BlockEnv, OpTransaction<TxEnv>> {
//             tx: OpTransaction::Deposit(tx),
//             ..Default::default()
//         };

//         // Nonce and balance checks should be skipped for deposit transactions.
//         assert!(validate_env::<TestEmptyOpWiring, LatestSpec>(&env).is_ok());
//     }
// }
