//! Gas constants and functions for gas calculation.

use crate::{transaction::AccessListItemTr as _, Transaction, TransactionType};
use primitives::{eip7702, hardfork::SpecId, U256};

/// Gas cost for operations that consume zero gas.
pub const ZERO: u64 = 0;
/// Base gas cost for basic operations.
pub const BASE: u64 = 2;

/// Gas cost for very low-cost operations.
pub const VERYLOW: u64 = 3;
/// Gas cost for DATALOADN instruction.
pub const DATA_LOADN_GAS: u64 = 3;

/// Gas cost for conditional jump instructions.
pub const CONDITION_JUMP_GAS: u64 = 4;
/// Gas cost for RETF instruction.
pub const RETF_GAS: u64 = 3;
/// Gas cost for DATALOAD instruction.
pub const DATA_LOAD_GAS: u64 = 4;

/// Gas cost for low-cost operations.
pub const LOW: u64 = 5;
/// Gas cost for medium-cost operations.
pub const MID: u64 = 8;
/// Gas cost for high-cost operations.
pub const HIGH: u64 = 10;
/// Gas cost for JUMPDEST instruction.
pub const JUMPDEST: u64 = 1;
/// Gas cost for REFUND SELFDESTRUCT instruction.
pub const SELFDESTRUCT_REFUND: i64 = 24000;
/// Gas cost for CREATE instruction.
pub const CREATE: u64 = 32000;
/// Additional gas cost when a call transfers value.
pub const CALLVALUE: u64 = 9000;
/// Gas cost for creating a new account.
pub const NEWACCOUNT: u64 = 25000;
/// Base gas cost for EXP instruction.
pub const EXP: u64 = 10;
/// Gas cost per word for memory operations.
pub const MEMORY: u64 = 3;
/// Base gas cost for LOG instructions.
pub const LOG: u64 = 375;
/// Gas cost per byte of data in LOG instructions.
pub const LOGDATA: u64 = 8;
/// Gas cost per topic in LOG instructions.
pub const LOGTOPIC: u64 = 375;
/// Base gas cost for KECCAK256 instruction.
pub const KECCAK256: u64 = 30;
/// Gas cost per word for KECCAK256 instruction.
pub const KECCAK256WORD: u64 = 6;
/// Gas cost per word for copy operations.
pub const COPY: u64 = 3;
/// Gas cost for BLOCKHASH instruction.
pub const BLOCKHASH: u64 = 20;
/// Gas cost per byte for code deposit during contract creation.
pub const CODEDEPOSIT: u64 = 200;

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub const ISTANBUL_SLOAD_GAS: u64 = 800;
/// Gas cost for SSTORE when setting a storage slot from zero to non-zero.
pub const SSTORE_SET: u64 = 20000;
/// Gas cost for SSTORE when modifying an existing non-zero storage slot.
pub const SSTORE_RESET: u64 = 5000;
/// Gas refund for SSTORE when clearing a storage slot (setting to zero).
pub const REFUND_SSTORE_CLEARS: i64 = 15000;

/// The standard cost of calldata token.
pub const STANDARD_TOKEN_COST: u64 = 4;
/// The cost of a non-zero byte in calldata.
pub const NON_ZERO_BYTE_DATA_COST: u64 = 68;
/// The multiplier for a non zero byte in calldata.
pub const NON_ZERO_BYTE_MULTIPLIER: u64 = NON_ZERO_BYTE_DATA_COST / STANDARD_TOKEN_COST;
/// The cost of a non-zero byte in calldata adjusted by [EIP-2028](https://eips.ethereum.org/EIPS/eip-2028).
pub const NON_ZERO_BYTE_DATA_COST_ISTANBUL: u64 = 16;
/// The multiplier for a non zero byte in calldata adjusted by [EIP-2028](https://eips.ethereum.org/EIPS/eip-2028).
pub const NON_ZERO_BYTE_MULTIPLIER_ISTANBUL: u64 =
    NON_ZERO_BYTE_DATA_COST_ISTANBUL / STANDARD_TOKEN_COST;
/// The cost floor per token as defined by EIP-2028.
pub const TOTAL_COST_FLOOR_PER_TOKEN: u64 = 10;

/// Gas cost for EOF CREATE instruction.
pub const EOF_CREATE_GAS: u64 = 32000;

// Berlin EIP-2929/EIP-2930 constants
/// Gas cost for accessing an address in the access list (EIP-2930).
pub const ACCESS_LIST_ADDRESS: u64 = 2400;
/// Gas cost for accessing a storage key in the access list (EIP-2930).
pub const ACCESS_LIST_STORAGE_KEY: u64 = 1900;

/// Gas cost for SLOAD when accessing a cold storage slot (EIP-2929).
pub const COLD_SLOAD_COST: u64 = 2100;
/// Gas cost for accessing a cold account (EIP-2929).
pub const COLD_ACCOUNT_ACCESS_COST: u64 = 2600;
/// Additional gas cost for accessing a cold account.
pub const COLD_ACCOUNT_ACCESS_COST_ADDITIONAL: u64 =
    COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST;
/// Gas cost for reading from a warm storage slot (EIP-2929).
pub const WARM_STORAGE_READ_COST: u64 = 100;
/// Gas cost for SSTORE reset operation on a warm storage slot.
pub const WARM_SSTORE_RESET: u64 = SSTORE_RESET - COLD_SLOAD_COST;

/// EIP-3860 : Limit and meter initcode
pub const INITCODE_WORD_COST: u64 = 2;

/// Gas stipend provided to the recipient of a CALL with value transfer.
pub const CALL_STIPEND: u64 = 2300;

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// Returns number of words what would fit to provided number of bytes,
/// i.e. it rounds up the number bytes to number of words.
#[inline]
pub const fn num_words(len: usize) -> usize {
    len.div_ceil(32)
}
