use super::constants::*;
use crate::num_words;
use context_interface::{transaction::AccessListItemTr as _, Transaction, TransactionType};
use primitives::{eip7702, hardfork::SpecId, U256};

#[inline]
pub(crate) const fn log2floor(value: U256) -> u64 {
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

/// Calculate the cost of buffer per word.
#[inline]
pub const fn cost_per_word(len: usize, multiple: u64) -> Option<u64> {
    multiple.checked_mul(num_words(len) as u64)
}

/// EIP-3860: Limit and meter initcode
///
/// Apply extra gas cost of 2 for every 32-byte chunk of initcode.
///
/// This cannot overflow as the initcode length is assumed to be checked.
#[inline]
pub const fn initcode_cost(len: usize) -> u64 {
    let Some(cost) = cost_per_word(len, INITCODE_WORD_COST) else {
        panic!("initcode cost overflow")
    };
    cost
}

/// Memory expansion cost calculation for a given number of words.
#[inline]
pub const fn memory_gas(num_words: usize, linear_cost: u64, quadratic_cost: u64) -> u64 {
    let num_words = num_words as u64;
    linear_cost
        .saturating_mul(num_words)
        .saturating_add(num_words.saturating_mul(num_words) / quadratic_cost)
}

/// Init and floor gas from transaction
#[derive(Clone, Copy, Debug, Default)]
pub struct InitialAndFloorGas {
    /// Initial gas for transaction.
    pub initial_gas: u64,
    /// If transaction is a Call and Prague is enabled
    /// floor_gas is at least amount of gas that is going to be spent.
    pub floor_gas: u64,
}

impl InitialAndFloorGas {
    /// Create a new InitialAndFloorGas instance.
    #[inline]
    pub const fn new(initial_gas: u64, floor_gas: u64) -> Self {
        Self {
            initial_gas,
            floor_gas,
        }
    }
}

/// Initial gas that is deducted for transaction to be included.
/// Initial gas contains initial stipend gas, gas for access list and input data.
///
/// # Returns
///
/// - Intrinsic gas
/// - Number of tokens in calldata
pub fn calculate_initial_tx_gas(
    spec_id: SpecId,
    input: &[u8],
    is_create: bool,
    access_list_accounts: u64,
    access_list_storages: u64,
    authorization_list_num: u64,
) -> InitialAndFloorGas {
    let mut gas = InitialAndFloorGas::default();

    // Initdate stipend
    let tokens_in_calldata = get_tokens_in_calldata(input, spec_id.is_enabled_in(SpecId::ISTANBUL));

    gas.initial_gas += tokens_in_calldata * STANDARD_TOKEN_COST;

    // Get number of access list account and storages.
    gas.initial_gas += access_list_accounts * ACCESS_LIST_ADDRESS;
    gas.initial_gas += access_list_storages * ACCESS_LIST_STORAGE_KEY;

    // Base stipend
    gas.initial_gas += if is_create {
        if spec_id.is_enabled_in(SpecId::HOMESTEAD) {
            // EIP-2: Homestead Hard-fork Changes
            53000
        } else {
            21000
        }
    } else {
        21000
    };

    // TODO init gas

    // EIP-3860: Limit and meter initcode
    // Init code stipend for bytecode analysis
    if spec_id.is_enabled_in(SpecId::SHANGHAI) && is_create {
        gas.initial_gas += initcode_cost(input.len())
    }

    // EIP-7702
    if spec_id.is_enabled_in(SpecId::PRAGUE) {
        gas.initial_gas += authorization_list_num * eip7702::PER_EMPTY_ACCOUNT_COST;

        // Calculate gas floor for EIP-7623
        gas.floor_gas = calc_tx_floor_cost(tokens_in_calldata);
    }

    gas
}

/// Initial gas that is deducted for transaction to be included.
/// Initial gas contains initial stipend gas, gas for access list and input data.
///
/// # Returns
///
/// - Intrinsic gas
/// - Number of tokens in calldata
pub fn calculate_initial_tx_gas_for_tx(tx: impl Transaction, spec: SpecId) -> InitialAndFloorGas {
    let mut accounts = 0;
    let mut storages = 0;
    // legacy is only tx type that does not have access list.
    if tx.tx_type() != TransactionType::Legacy {
        (accounts, storages) = tx
            .access_list()
            .map(|al| {
                al.fold((0, 0), |(mut num_accounts, mut num_storage_slots), item| {
                    num_accounts += 1;
                    num_storage_slots += item.storage_slots().count();

                    (num_accounts, num_storage_slots)
                })
            })
            .unwrap_or_default();
    }

    calculate_initial_tx_gas(
        spec,
        tx.input(),
        tx.kind().is_create(),
        accounts as u64,
        storages as u64,
        tx.authorization_list_len() as u64,
    )
}

/// Retrieve the total number of tokens in calldata.
#[inline]
pub fn get_tokens_in_calldata(input: &[u8], is_istanbul: bool) -> u64 {
    let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
    let non_zero_data_len = input.len() as u64 - zero_data_len;
    let non_zero_data_multiplier = if is_istanbul {
        // EIP-2028: Transaction data gas cost reduction
        NON_ZERO_BYTE_MULTIPLIER_ISTANBUL
    } else {
        NON_ZERO_BYTE_MULTIPLIER
    };
    zero_data_len + non_zero_data_len * non_zero_data_multiplier
}

/// Calculate the transaction cost floor as specified in EIP-7623.
#[inline]
pub fn calc_tx_floor_cost(tokens_in_calldata: u64) -> u64 {
    tokens_in_calldata * TOTAL_COST_FLOOR_PER_TOKEN + 21_000
}
