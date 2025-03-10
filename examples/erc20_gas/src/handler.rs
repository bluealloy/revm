use core::cmp::Ordering;
use revm::{
    context::{Cfg, JournalOutput},
    context_interface::{
        result::{HaltReason, InvalidTransaction},
        Block, ContextTr, JournalTr, Transaction, TransactionType,
    },
    handler::{EvmTr, EvmTrError, Frame, FrameResult, Handler},
    interpreter::FrameInput,
    primitives::{hardfork::SpecId, U256},
};

use crate::{erc_address_storage, token_operation, TOKEN, TREASURY};

pub struct Erc20MainetHandler<EVM, ERROR, FRAME> {
    _phantom: core::marker::PhantomData<(EVM, ERROR, FRAME)>,
}

impl<CTX, ERROR, FRAME> Erc20MainetHandler<CTX, ERROR, FRAME> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM, ERROR, FRAME> Default for Erc20MainetHandler<EVM, ERROR, FRAME> {
    fn default() -> Self {
        Self::new()
    }
}

impl<EVM, ERROR, FRAME> Handler for Erc20MainetHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>>,
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
    ERROR: EvmTrError<EVM>,
{
    type Evm = EVM;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = HaltReason;

    fn validate_tx_against_state(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let caller = context.tx().caller();
        let caller_nonce = context.journal().load_account(caller)?.data.info.nonce;
        let _ = context.journal().load_account(TOKEN)?.data.clone();

        if !context.cfg().is_nonce_check_disabled() {
            let tx_nonce = context.tx().nonce();
            let state_nonce = caller_nonce;
            match tx_nonce.cmp(&state_nonce) {
                Ordering::Less => {
                    return Err(ERROR::from(InvalidTransaction::NonceTooLow {
                        tx: tx_nonce,
                        state: state_nonce,
                    }))
                }
                Ordering::Greater => {
                    return Err(ERROR::from(InvalidTransaction::NonceTooHigh {
                        tx: tx_nonce,
                        state: state_nonce,
                    }))
                }
                _ => (),
            }
        }

        let mut balance_check = U256::from(context.tx().gas_limit())
            .checked_mul(U256::from(context.tx().max_fee_per_gas()))
            .and_then(|gas_cost| gas_cost.checked_add(context.tx().value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        if context.tx().tx_type() == TransactionType::Eip4844 {
            let tx = context.tx();
            let data_fee = tx.calc_max_data_fee();
            balance_check = balance_check
                .checked_add(data_fee)
                .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
        }

        let account_balance_slot = erc_address_storage(caller);
        let account_balance = context
            .journal()
            .sload(TOKEN, account_balance_slot)
            .map(|v| v.data)
            .unwrap_or_default();

        if account_balance < balance_check && !context.cfg().is_balance_check_disabled() {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(balance_check),
                balance: Box::new(account_balance),
            }
            .into());
        };

        Ok(())
    }

    fn deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        let context = evm.ctx();
        // load and touch token account
        let _ = context.journal().load_account(TOKEN)?.data;
        context.journal().touch_account(TOKEN);

        let basefee = context.block().basefee() as u128;
        let blob_price = context.block().blob_gasprice().unwrap_or_default();
        let effective_gas_price = context.tx().effective_gas_price(basefee);

        let mut gas_cost = (context.tx().gas_limit() as u128).saturating_mul(effective_gas_price);

        if context.tx().tx_type() == TransactionType::Eip4844 {
            let blob_gas = context.tx().total_blob_gas() as u128;
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        let caller = context.tx().caller();
        println!("Deduct caller: {:?} for amount: {gas_cost:?}", caller);
        token_operation::<EVM::Context, ERROR>(context, caller, TREASURY, U256::from(gas_cost))?;

        Ok(())
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);
        token_operation::<EVM::Context, ERROR>(
            context,
            TREASURY,
            caller,
            U256::from(reimbursement),
        )?;

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let tx = context.tx();
        let beneficiary = context.block().beneficiary();
        let basefee = context.block().basefee() as u128;
        let effective_gas_price = tx.effective_gas_price(basefee);
        let gas = exec_result.gas();

        let coinbase_gas_price = if context.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let reward =
            coinbase_gas_price.saturating_mul((gas.spent() - gas.refunded() as u64) as u128);
        token_operation::<EVM::Context, ERROR>(context, TREASURY, beneficiary, U256::from(reward))?;

        Ok(())
    }
}
