use super::constants::*;
use crate::{
    num_words,
    primitives::{AccessListItem, SpecId, U256},
    SelfDestructResult,
};

/// `const` Option `?`.
macro_rules! tri {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None,
        }
    };
}

/// `SSTORE` opcode refund calculation.
#[allow(clippy::collapsible_else_if)]
#[inline]
pub fn sstore_refund(spec_id: SpecId, original: U256, current: U256, new: U256) -> i64 {
    if spec_id.is_enabled_in(SpecId::ISTANBUL) {
        // EIP-3529: Reduction in refunds
        let sstore_clears_schedule = if spec_id.is_enabled_in(SpecId::LONDON) {
            (SSTORE_RESET - COLD_SLOAD_COST + ACCESS_LIST_STORAGE_KEY) as i64
        } else {
            REFUND_SSTORE_CLEARS
        };
        if current == new {
            0
        } else {
            if original == current && new.is_zero() {
                sstore_clears_schedule
            } else {
                let mut refund = 0;

                if !original.is_zero() {
                    if current.is_zero() {
                        refund -= sstore_clears_schedule;
                    } else if new.is_zero() {
                        refund += sstore_clears_schedule;
                    }
                }

                if original == new {
                    let (gas_sstore_reset, gas_sload) = if spec_id.is_enabled_in(SpecId::BERLIN) {
                        (SSTORE_RESET - COLD_SLOAD_COST, WARM_STORAGE_READ_COST)
                    } else {
                        (SSTORE_RESET, sload_cost(spec_id, false))
                    };
                    if original.is_zero() {
                        refund += (SSTORE_SET - gas_sload) as i64;
                    } else {
                        refund += (gas_sstore_reset - gas_sload) as i64;
                    }
                }

                refund
            }
        }
    } else {
        if !current.is_zero() && new.is_zero() {
            REFUND_SSTORE_CLEARS
        } else {
            0
        }
    }
}

/// `CREATE2` opcode cost calculation.
#[inline]
pub const fn create2_cost(len: u64) -> Option<u64> {
    CREATE.checked_add(tri!(cost_per_word(len, KECCAK256WORD)))
}

#[inline]
const fn log2floor(value: U256) -> u64 {
    let mut l: u64 = 256;
    let mut i = 3;
    loop {
        if value.as_limbs()[i] == 0u64 {
            l -= 64;
        } else {
            l -= value.as_limbs()[i].leading_zeros() as u64;
            if l == 0 {
                return l;
            } else {
                return l - 1;
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    l
}

/// `EXP` opcode cost calculation.
#[inline]
pub fn exp_cost(spec_id: SpecId, power: U256) -> Option<u64> {
    if power.is_zero() {
        Some(EXP)
    } else {
        // EIP-160: EXP cost increase
        let gas_byte = U256::from(if spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            50
        } else {
            10
        });
        let gas = U256::from(EXP)
            .checked_add(gas_byte.checked_mul(U256::from(log2floor(power) / 8 + 1))?)?;

        u64::try_from(gas).ok()
    }
}

/// `*COPY` opcodes cost calculation.
#[inline]
pub const fn verylowcopy_cost(len: u64) -> Option<u64> {
    VERYLOW.checked_add(tri!(cost_per_word(len, COPY)))
}

/// `EXTCODECOPY` opcode cost calculation.
#[inline]
pub const fn extcodecopy_cost(spec_id: SpecId, len: u64, is_cold: bool) -> Option<u64> {
    let base_gas = if spec_id.is_enabled_in(SpecId::BERLIN) {
        warm_cold_cost(is_cold)
    } else if spec_id.is_enabled_in(SpecId::TANGERINE) {
        700
    } else {
        20
    };
    base_gas.checked_add(tri!(cost_per_word(len, COPY)))
}

/// `LOG` opcode cost calculation.
#[inline]
pub const fn log_cost(n: u8, len: u64) -> Option<u64> {
    tri!(LOG.checked_add(tri!(LOGDATA.checked_mul(len)))).checked_add(LOGTOPIC * n as u64)
}

/// `KECCAK256` opcode cost calculation.
#[inline]
pub const fn keccak256_cost(len: u64) -> Option<u64> {
    KECCAK256.checked_add(tri!(cost_per_word(len, KECCAK256WORD)))
}

/// Calculate the cost of buffer per word.
#[inline]
pub const fn cost_per_word(len: u64, multiple: u64) -> Option<u64> {
    multiple.checked_mul(num_words(len))
}

/// EIP-3860: Limit and meter initcode
///
/// Apply extra gas cost of 2 for every 32-byte chunk of initcode.
///
/// This cannot overflow as the initcode length is assumed to be checked.
#[inline]
pub const fn initcode_cost(len: u64) -> u64 {
    let Some(cost) = cost_per_word(len, INITCODE_WORD_COST) else {
        panic!("initcode cost overflow")
    };
    cost
}

/// `SLOAD` opcode cost calculation.
#[inline]
pub const fn sload_cost(spec_id: SpecId, is_cold: bool) -> u64 {
    if spec_id.is_enabled_in(SpecId::BERLIN) {
        if is_cold {
            COLD_SLOAD_COST
        } else {
            WARM_STORAGE_READ_COST
        }
    } else if spec_id.is_enabled_in(SpecId::ISTANBUL) {
        // EIP-1884: Repricing for trie-size-dependent opcodes
        INSTANBUL_SLOAD_GAS
    } else if spec_id.is_enabled_in(SpecId::TANGERINE) {
        // EIP-150: Gas cost changes for IO-heavy operations
        200
    } else {
        50
    }
}

/// `SSTORE` opcode cost calculation.
#[inline]
pub fn sstore_cost(
    spec_id: SpecId,
    original: U256,
    current: U256,
    new: U256,
    gas: u64,
    is_cold: bool,
) -> Option<u64> {
    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if spec_id.is_enabled_in(SpecId::ISTANBUL) && gas <= CALL_STIPEND {
        return None;
    }

    if spec_id.is_enabled_in(SpecId::BERLIN) {
        // Berlin specification logic
        let mut gas_cost = istanbul_sstore_cost::<WARM_STORAGE_READ_COST, WARM_SSTORE_RESET>(
            original, current, new,
        );

        if is_cold {
            gas_cost += COLD_SLOAD_COST;
        }
        Some(gas_cost)
    } else if spec_id.is_enabled_in(SpecId::ISTANBUL) {
        // Istanbul logic
        Some(istanbul_sstore_cost::<INSTANBUL_SLOAD_GAS, SSTORE_RESET>(
            original, current, new,
        ))
    } else {
        // Frontier logic
        Some(frontier_sstore_cost(current, new))
    }
}

/// EIP-2200: Structured Definitions for Net Gas Metering
#[inline]
fn istanbul_sstore_cost<const SLOAD_GAS: u64, const SSTORE_RESET_GAS: u64>(
    original: U256,
    current: U256,
    new: U256,
) -> u64 {
    if new == current {
        SLOAD_GAS
    } else if original == current && original.is_zero() {
        SSTORE_SET
    } else if original == current {
        SSTORE_RESET_GAS
    } else {
        SLOAD_GAS
    }
}

/// Frontier sstore cost just had two cases set and reset values.
#[inline]
fn frontier_sstore_cost(current: U256, new: U256) -> u64 {
    if current.is_zero() && !new.is_zero() {
        SSTORE_SET
    } else {
        SSTORE_RESET
    }
}

/// `SELFDESTRUCT` opcode cost calculation.
#[inline]
pub const fn selfdestruct_cost(spec_id: SpecId, res: SelfDestructResult) -> u64 {
    // EIP-161: State trie clearing (invariant-preserving alternative)
    let should_charge_topup = if spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
        res.had_value && !res.target_exists
    } else {
        !res.target_exists
    };

    // EIP-150: Gas cost changes for IO-heavy operations
    let selfdestruct_gas_topup = if spec_id.is_enabled_in(SpecId::TANGERINE) && should_charge_topup
    {
        25000
    } else {
        0
    };

    // EIP-150: Gas cost changes for IO-heavy operations
    let selfdestruct_gas = if spec_id.is_enabled_in(SpecId::TANGERINE) {
        5000
    } else {
        0
    };

    let mut gas = selfdestruct_gas + selfdestruct_gas_topup;
    if spec_id.is_enabled_in(SpecId::BERLIN) && res.is_cold {
        gas += COLD_ACCOUNT_ACCESS_COST
    }
    gas
}

/// Calculate call gas cost for the call instruction.
///
/// There is three types of gas.
/// * Account access gas. after berlin it can be cold or warm.
/// * Transfer value gas. If value is transferred and balance of target account is updated.
/// * If account is not existing and needs to be created. After Spurious dragon
/// this is only accounted if value is transferred.
#[inline]
pub const fn call_cost(
    spec_id: SpecId,
    transfers_value: bool,
    is_cold: bool,
    new_account_accounting: bool,
) -> u64 {
    // Account access.
    let mut gas = if spec_id.is_enabled_in(SpecId::BERLIN) {
        warm_cold_cost(is_cold)
    } else if spec_id.is_enabled_in(SpecId::TANGERINE) {
        // EIP-150: Gas cost changes for IO-heavy operations
        700
    } else {
        40
    };

    // transfer value cost
    if transfers_value {
        gas += CALLVALUE;
    }

    // new account cost
    if new_account_accounting {
        // EIP-161: State trie clearing (invariant-preserving alternative)
        if spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
            // account only if there is value transferred.
            if transfers_value {
                gas += NEWACCOUNT;
            }
        } else {
            gas += NEWACCOUNT;
        }
    }

    gas
}

/// Berlin warm and cold storage access cost for account access.
#[inline]
pub const fn warm_cold_cost(is_cold: bool) -> u64 {
    if is_cold {
        COLD_ACCOUNT_ACCESS_COST
    } else {
        WARM_STORAGE_READ_COST
    }
}

/// Memory expansion cost calculation for a given memory length.
#[inline]
pub const fn memory_gas_for_len(len: usize) -> u64 {
    memory_gas(crate::interpreter::num_words(len as u64))
}

/// Memory expansion cost calculation for a given number of words.
#[inline]
pub const fn memory_gas(num_words: u64) -> u64 {
    MEMORY
        .saturating_mul(num_words)
        .saturating_add(num_words.saturating_mul(num_words) / 512)
}

/// Initial gas that is deducted for transaction to be included.
/// Initial gas contains initial stipend gas, gas for access list and input data.
pub fn validate_initial_tx_gas(
    spec_id: SpecId,
    input: &[u8],
    is_create: bool,
    access_list: &[AccessListItem],
    authorization_list_num: u64,
) -> u64 {
    let mut initial_gas = 0;
    let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
    let non_zero_data_len = input.len() as u64 - zero_data_len;

    // initdate stipend
    initial_gas += zero_data_len * TRANSACTION_ZERO_DATA;
    // EIP-2028: Transaction data gas cost reduction
    initial_gas += non_zero_data_len
        * if spec_id.is_enabled_in(SpecId::ISTANBUL) {
            16
        } else {
            68
        };

    // get number of access list account and storages.
    if spec_id.is_enabled_in(SpecId::BERLIN) {
        let accessed_slots: usize = access_list.iter().map(|item| item.storage_keys.len()).sum();
        initial_gas += access_list.len() as u64 * ACCESS_LIST_ADDRESS;
        initial_gas += accessed_slots as u64 * ACCESS_LIST_STORAGE_KEY;
    }

    // base stipend
    initial_gas += if is_create {
        if spec_id.is_enabled_in(SpecId::HOMESTEAD) {
            // EIP-2: Homestead Hard-fork Changes
            53000
        } else {
            21000
        }
    } else {
        21000
    };

    // EIP-3860: Limit and meter initcode
    // Init code stipend for bytecode analysis
    if spec_id.is_enabled_in(SpecId::SHANGHAI) && is_create {
        initial_gas += initcode_cost(input.len() as u64)
    }

    //   EIP-7702
    if spec_id.is_enabled_in(SpecId::PRAGUE) {
        initial_gas += authorization_list_num * PER_AUTH_BASE_COST;
    }

    initial_gas
}
