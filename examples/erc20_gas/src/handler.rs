use revm::{
    context::Cfg,
    context_interface::{
        result::{HaltReason, InvalidTransaction},
        Block, CfgGetter, Journal, Transaction, TransactionType,
    },
    handler::{
        instructions::EthInstructions, EthContext, EthError, EthFrame, EthHandler,
        EthPrecompiles, FrameContext,
    },
    handler_interface::Frame,
    interpreter::{interpreter::EthInterpreter, Host},
    precompile::PrecompileErrors,
    primitives::U256,
    specification::hardfork::SpecId,
};
use std::cmp::Ordering;

use crate::{erc_address_storage, token_operation, TOKEN, TREASURY};

pub struct Erc20MainetHandler<CTX, ERROR> {
    _phantom: core::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> Erc20MainetHandler<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX: CfgGetter + Host, ERROR: From<PrecompileErrors>> Default
    for Erc20MainetHandler<CTX, ERROR>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<CTX, ERROR> EthHandler for Erc20MainetHandler<CTX, ERROR>
where
    CTX: EthContext,
    ERROR: EthError<CTX>,
{
    type Context = CTX;
    type Error = ERROR;
    type Precompiles = EthPrecompiles<CTX, Self::Error>;
    type Instructions = EthInstructions<EthInterpreter, Self::Context>;
    type Frame =
        EthFrame<CTX, ERROR, EthInterpreter, FrameContext<Self::Precompiles, Self::Instructions>>;
    type HaltReason = HaltReason;

    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
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

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
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
        token_operation::<CTX, ERROR>(context, caller, TREASURY, U256::from(gas_cost))?;

        Ok(())
    }

    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);
        token_operation::<CTX, ERROR>(context, TREASURY, caller, U256::from(reimbursement))?;

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
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
        token_operation::<CTX, ERROR>(context, TREASURY, beneficiary, U256::from(reward))?;

        Ok(())
    }
}
