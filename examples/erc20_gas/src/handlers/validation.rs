use crate::TOKEN;
use alloy_sol_types::SolValue;
use revm::{
    context::Cfg,
    context_interface::{
        result::InvalidTransaction, Journal, Transaction, TransactionGetter, TransactionType,
    },
    handler::{EthValidation, EthValidationContext, EthValidationError},
    handler_interface::{InitialAndFloorGas, ValidationHandler},
    primitives::{keccak256, U256},
};
use std::cmp::Ordering;

pub struct Erc20Validation<CTX, ERROR> {
    inner: EthValidation<CTX, ERROR>,
}

impl<CTX, ERROR> Erc20Validation<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            inner: EthValidation::new(),
        }
    }
}

impl<CTX, ERROR> Default for Erc20Validation<CTX, ERROR> {
    fn default() -> Self {
        Self::new()
    }
}

impl<CTX, ERROR> ValidationHandler for Erc20Validation<CTX, ERROR>
where
    CTX: EthValidationContext,
    ERROR: EthValidationError<CTX>,
{
    type Context = CTX;
    type Error = ERROR;

    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        self.inner.validate_env(context)
    }

    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let caller = context.tx().caller();
        let caller_nonce = context.journal().load_account(caller)?.data.info.nonce;
        let token_account = context.journal().load_account(TOKEN)?.data.clone();

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

        let account_balance_slot: U256 = keccak256((caller, U256::from(3)).abi_encode()).into();
        let account_balance = token_account
            .storage
            .get(&account_balance_slot)
            .expect("Balance slot not found")
            .present_value();

        if account_balance < balance_check && !context.cfg().is_balance_check_disabled() {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(balance_check),
                balance: Box::new(account_balance),
            }
            .into());
        };

        Ok(())
    }

    fn validate_initial_tx_gas(
        &self,
        context: &Self::Context,
    ) -> Result<InitialAndFloorGas, Self::Error> {
        self.inner.validate_initial_tx_gas(context)
    }
}
