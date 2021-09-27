use super::constants::*;
use crate::error::ExitError;
use crate::spec::Spec;
use primitive_types::{H256, U256};

pub fn call_extra_check<SPEC: Spec>(gas: U256, after_gas: u64) -> Result<(), ExitError> {
    if SPEC::err_on_call_with_more_gas && U256::from(after_gas) < gas {
        Err(ExitError::OutOfGas)
    } else {
        Ok(())
    }
}

pub fn suicide_refund(already_removed: bool) -> i64 {
    if already_removed {
        0
    } else {
        SUICIDE
    }
}

#[allow(clippy::collapsible_else_if)]
pub fn sstore_refund<SPEC: Spec>(original: H256, current: H256, new: H256) -> i64 {
    if SPEC::sstore_gas_metering {
        if current == new {
            0
        } else {
            if original == current && new == H256::default() {
                SPEC::refund_sstore_clears
            } else {
                let mut refund = 0;

                if original != H256::default() {
                    if current == H256::default() {
                        refund -= SPEC::refund_sstore_clears;
                    } else if new == H256::default() {
                        refund += SPEC::refund_sstore_clears;
                    }
                }

                if original == new {
                    let (gas_sstore_reset, gas_sload) = if SPEC::increase_state_access_gas {
                        (
                            SPEC::gas_sstore_reset - SPEC::gas_sload_cold,
                            SPEC::gas_storage_read_warm,
                        )
                    } else {
                        (SPEC::gas_sstore_reset, SPEC::gas_sload)
                    };
                    if original == H256::default() {
                        refund += (SPEC::gas_sstore_set - gas_sload) as i64;
                    } else {
                        refund += (gas_sstore_reset - gas_sload) as i64;
                    }
                }

                refund
            }
        }
    } else {
        if current != H256::default() && new == H256::default() {
            SPEC::refund_sstore_clears
        } else {
            0
        }
    }
}

pub fn create2_cost(len: U256) -> Result<u64, ExitError> {
    let base = U256::from(CREATE);
    // ceil(len / 32.0)
    let sha_addup_base = len / U256::from(32)
        + if len % U256::from(32) == U256::zero() {
            U256::zero()
        } else {
            U256::one()
        };
    let sha_addup = U256::from(SHA3WORD)
        .checked_mul(sha_addup_base)
        .ok_or(ExitError::OutOfGas)?;
    let gas = base.checked_add(sha_addup).ok_or(ExitError::OutOfGas)?;

    if gas > U256::from(u64::MAX) {
        return Err(ExitError::OutOfGas);
    }

    Ok(gas.as_u64())
}

pub fn exp_cost<SPEC: Spec>(power: U256) -> Option<u64> {
    if power == U256::zero() {
        Some(EXP)
    } else {
        let gas = U256::from(EXP).checked_add(
            U256::from(SPEC::gas_expbyte)
                .checked_mul(U256::from(super::utils::log2floor(power) / 8 + 1))?,
        )?;

        if gas > U256::from(u64::MAX) {
            return None;
        }

        Some(gas.as_u64())
    }
}

pub fn verylowcopy_cost(len: U256) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);

    let gas = U256::from(VERYLOW).checked_add(U256::from(COPY).checked_mul(
        if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        },
    )?)?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn extcodecopy_cost<SPEC: Spec>(len: U256, is_cold: bool) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);
    let gas = U256::from(account_access_cost::<SPEC>(is_cold, SPEC::gas_ext_code)).checked_add(
        U256::from(COPY).checked_mul(if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        })?,
    )?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn log_cost(n: u8, len: U256) -> Option<u64> {
    let gas = U256::from(LOG)
        .checked_add(U256::from(LOGDATA).checked_mul(len)?)?
        .checked_add(U256::from(LOGTOPIC * n as u64))?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn sha3_cost(len: U256) -> Option<u64> {
    let wordd = len / U256::from(32);
    let wordr = len % U256::from(32);

    let gas = U256::from(SHA3).checked_add(U256::from(SHA3WORD).checked_mul(
        if wordr == U256::zero() {
            wordd
        } else {
            wordd + U256::one()
        },
    )?)?;

    if gas > U256::from(u64::MAX) {
        return None;
    }

    Some(gas.as_u64())
}

pub fn sload_cost<SPEC: Spec>(is_cold: bool) -> u64 {
    if SPEC::increase_state_access_gas {
        if is_cold {
            SPEC::gas_sload_cold
        } else {
            SPEC::gas_storage_read_warm
        }
    } else {
        SPEC::gas_sload
    }
}

#[allow(clippy::collapsible_else_if)]
pub fn sstore_cost<SPEC: Spec>(
    original: H256,
    current: H256,
    new: H256,
    gas: u64,
    is_cold: bool,
) -> Result<u64, ExitError> {
    let (gas_sload, gas_sstore_reset) = if SPEC::increase_state_access_gas {
        (
            SPEC::gas_storage_read_warm,
            SPEC::gas_sstore_reset - SPEC::gas_sload_cold,
        )
    } else {
        (SPEC::gas_sload, SPEC::gas_sstore_reset)
    };
    let gas_cost = if SPEC::sstore_gas_metering {
        if SPEC::sstore_revert_under_stipend && gas <= SPEC::call_stipend {
            return Err(ExitError::OutOfGas);
        }

        if new == current {
            gas_sload
        } else {
            if original == current {
                if original == H256::zero() {
                    SPEC::gas_sstore_set
                } else {
                    gas_sstore_reset
                }
            } else {
                gas_sload
            }
        }
    } else {
        if current == H256::zero() && new != H256::zero() {
            SPEC::gas_sstore_set
        } else {
            gas_sstore_reset
        }
    };
    Ok(
        // In EIP-2929 we charge extra if the slot has not been used yet in this transaction
        if is_cold {
            gas_cost + SPEC::gas_sload_cold
        } else {
            gas_cost
        },
    )
}

pub fn suicide_cost<SPEC: Spec>(value: U256, is_cold: bool, target_exists: bool) -> u64 {
    let eip161 = !SPEC::empty_considered_exists;
    let should_charge_topup = if eip161 {
        value != U256::zero() && !target_exists
    } else {
        !target_exists
    };

    let suicide_gas_topup = if should_charge_topup {
        SPEC::gas_suicide_new_account
    } else {
        0
    };

    let mut gas = SPEC::gas_suicide + suicide_gas_topup;
    if SPEC::increase_state_access_gas && is_cold {
        gas += SPEC::gas_account_access_cold
    }
    gas
}

pub fn call_cost<SPEC: Spec>(
    value: U256,
    is_cold: bool,
    is_call_or_callcode: bool,
    is_call_or_staticcall: bool,
    new_account: bool,
) -> u64 {
    let transfers_value = value != U256::default();
    account_access_cost::<SPEC>(is_cold, SPEC::gas_call)
        + xfer_cost(is_call_or_callcode, transfers_value)
        + new_cost::<SPEC>(is_call_or_staticcall, new_account, transfers_value)
}

#[inline(always)]
pub fn account_access_cost<SPEC: Spec>(is_cold: bool, regular_value: u64) -> u64 {
    if SPEC::increase_state_access_gas {
        if is_cold {
            SPEC::gas_account_access_cold
        } else {
            SPEC::gas_storage_read_warm
        }
    } else {
        regular_value
    }
}

fn xfer_cost(is_call_or_callcode: bool, transfers_value: bool) -> u64 {
    if is_call_or_callcode && transfers_value {
        CALLVALUE
    } else {
        0
    }
}

fn new_cost<SPEC: Spec>(
    is_call_or_staticcall: bool,
    new_account: bool,
    transfers_value: bool,
) -> u64 {
    //let eip161 = !SPEC::empty_considered_exists;
    if is_call_or_staticcall {
        if !SPEC::empty_considered_exists {
            if transfers_value && new_account {
                NEWACCOUNT
            } else {
                0
            }
        } else if new_account {
            NEWACCOUNT
        } else {
            0
        }
    } else {
        0
    }
}

// pub fn memory_gas(a: usize) -> Option<u64> {
//     let a = a as u64;
//     MEMORY.checked_mul(a)?.checked_add(a.checked_mul(a)? / 512)
// }

pub fn memory_gas(a: usize) -> Result<u64, ExitError> {
    let a = a as u64;
    MEMORY
        .checked_mul(a)
        .ok_or(ExitError::OutOfGas)?
        .checked_add(a.checked_mul(a).ok_or(ExitError::OutOfGas)? / 512)
        .ok_or(ExitError::OutOfGas)
}
