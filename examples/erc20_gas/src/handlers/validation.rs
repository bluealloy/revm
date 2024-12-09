use crate::error::Erc20Error;
use crate::keccak256;
use crate::TOKEN;
use alloy_sol_types::SolValue;
use revm::context_interface::{Transaction, TransactionGetter};
use revm::{
    context::Cfg,
    context_interface::{
        result::{EVMError, InvalidTransaction},
        transaction::Eip4844Tx,
        JournalStateGetter, TransactionType,
    },
    handler::EthValidation,
    handler_interface::ValidationHandler,
    primitives::U256,
    Context,
};
use std::cmp::Ordering;

pub struct Erc20Validation {
    inner: EthValidation<Context, Erc20Error>,
}

impl Erc20Validation {
    pub fn new() -> Self {
        Self {
            inner: EthValidation::new(),
        }
    }
}

impl ValidationHandler for Erc20Validation {
    type Context = Context;
    type Error = Erc20Error;

    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        self.inner.validate_env(context)
    }

    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let caller = context.tx().common_fields().caller();
        let caller_nonce = context.journal().load_account(caller)?.data.info.nonce;
        let token_account = context.journal().load_account(TOKEN)?.data.clone();

        if !context.cfg.is_nonce_check_disabled() {
            let tx_nonce = context.tx().common_fields().nonce();
            let state_nonce = caller_nonce;
            match tx_nonce.cmp(&state_nonce) {
                Ordering::Less => {
                    return Err(EVMError::Transaction(
                        InvalidTransaction::NonceTooLow {
                            tx: tx_nonce,
                            state: state_nonce,
                        }
                        .into(),
                    ))
                }
                Ordering::Greater => {
                    return Err(EVMError::Transaction(
                        InvalidTransaction::NonceTooHigh {
                            tx: tx_nonce,
                            state: state_nonce,
                        }
                        .into(),
                    ))
                }
                _ => (),
            }
        }

        let mut balance_check = U256::from(context.tx().common_fields().gas_limit())
            .checked_mul(U256::from(context.tx().max_fee()))
            .and_then(|gas_cost| gas_cost.checked_add(context.tx().common_fields().value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        if context.tx().tx_type() == TransactionType::Eip4844 {
            let tx = context.tx().eip4844();
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

        if account_balance < balance_check && !context.cfg.is_balance_check_disabled() {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(balance_check),
                balance: Box::new(account_balance),
            }
            .into());
        };

        Ok(())
    }

    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error> {
        self.inner.validate_initial_tx_gas(context)
    }
}
