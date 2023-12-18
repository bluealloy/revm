//! Mainnet related handlers.
use revm_interpreter::primitives::EVMError;

use crate::{
    interpreter::{return_ok, return_revert, Gas, InstructionResult},
    primitives::{db::Database, Env, Spec, SpecId::LONDON, U256},
    EVMData,
};

#[inline]
pub fn handle_reimburse_caller<SPEC: Spec, DB: Database>(
    data: &mut EVMData<'_, DB>,
    gas: &Gas,
    gas_refund: u64,
) -> Result<(), EVMError<DB::Error>> {
    let _ = data;
    if data.env.tx.taiko.is_anchor {
        return Ok(());
    }
    let caller = data.env.tx.caller;
    let effective_gas_price = data.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) = data
        .journaled_state
        .load_account(caller, data.db)
        .map_err(EVMError::Database)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas_refund));

    Ok(())
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, DB: Database>(
    data: &mut EVMData<'_, DB>,
    gas: &Gas,
    gas_refund: u64,
) -> Result<(), EVMError<DB::Error>> {
    if data.env.tx.taiko.is_anchor {
        return Ok(());
    }
    let beneficiary = data.env.block.coinbase;
    let effective_gas_price = data.env.effective_gas_price();

    // transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if SPEC::enabled(LONDON) {
        effective_gas_price.saturating_sub(data.env.block.basefee)
    } else {
        effective_gas_price
    };

    let (coinbase_account, _) = data
        .journaled_state
        .load_account(beneficiary, data.db)
        .map_err(EVMError::Database)?;

    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spend() - gas_refund));

    let treasury = data.env.tx.taiko.treasury;
    let basefee = data.env.block.basefee;

    let (treasury_account, _) = data
        .journaled_state
        .load_account(treasury, data.db)
        .map_err(EVMError::Database)?;

    treasury_account.mark_touch();
    treasury_account.info.balance = treasury_account
        .info
        .balance
        .saturating_add(basefee * U256::from(gas.spend() - gas_refund));
    Ok(())
}
