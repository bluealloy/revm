//! Gas constants and functions for gas calculation.

use crate::{cfg::GasParams, transaction::AccessListItemTr as _, Transaction, TransactionType};
use primitives::hardfork::SpecId;

/// Tracker for gas during execution.
///
/// This is used to track the gas during execution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GasTracker {
    /// Gas Limit,
    gas_limit: u64,
    /// Regular gas remaining (`gas_left`). Reservoir is tracked separately.
    remaining: u64,
    /// State gas reservoir (gas exceeding TX_MAX_GAS_LIMIT). Starts as `execution_gas - min(execution_gas, regular_gas_budget)`.
    /// When 0, all remaining gas is regular gas with hard cap at `TX_MAX_GAS_LIMIT`.
    reservoir: u64,
    /// Total state gas spent so far.
    state_gas_spent: u64,
    /// Refunded gas. Used to refund the gas to the caller at the end of execution.
    refunded: i64,
}

impl GasTracker {
    /// Creates a new `GasTracker` with the given remaining gas and reservoir.
    #[inline]
    pub const fn new(gas_limit: u64, remaining: u64, reservoir: u64) -> Self {
        Self {
            gas_limit,
            remaining,
            reservoir,
            state_gas_spent: 0,
            refunded: 0,
        }
    }

    /// Creates a new `GasTracker` with the given used gas and reservoir.
    #[inline]
    pub const fn new_used_gas(gas_limit: u64, used_gas: u64, reservoir: u64) -> Self {
        Self::new(gas_limit, gas_limit - used_gas, reservoir)
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.gas_limit
    }

    /// Sets the gas limit.
    #[inline]
    pub const fn set_limit(&mut self, val: u64) {
        self.gas_limit = val;
    }

    /// Returns the remaining gas.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Sets the remaining gas.
    #[inline]
    pub const fn set_remaining(&mut self, val: u64) {
        self.remaining = val;
    }

    /// Returns the reservoir gas.
    #[inline]
    pub const fn reservoir(&self) -> u64 {
        self.reservoir
    }

    /// Sets the reservoir gas.
    #[inline]
    pub const fn set_reservoir(&mut self, val: u64) {
        self.reservoir = val;
    }

    /// Returns the state gas spent.
    #[inline]
    pub const fn state_gas_spent(&self) -> u64 {
        self.state_gas_spent
    }

    /// Sets the state gas spent.
    #[inline]
    pub const fn set_state_gas_spent(&mut self, val: u64) {
        self.state_gas_spent = val;
    }

    /// Returns the refunded gas.
    #[inline]
    pub const fn refunded(&self) -> i64 {
        self.refunded
    }

    /// Sets the refunded gas.
    #[inline]
    pub const fn set_refunded(&mut self, val: i64) {
        self.refunded = val;
    }

    /// Records a regular gas cost.
    ///
    /// Deducts from `remaining`. Returns `false` if insufficient gas.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_regular_cost(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            return true;
        }
        false
    }

    /// Records a state gas cost (EIP-8037 reservoir model).
    ///
    /// State gas charges deduct from the reservoir first. If the reservoir is exhausted,
    /// remaining charges spill into `remaining` (requiring `remaining >= cost`).
    /// Tracks state gas spent.
    ///
    /// Returns `false` if total remaining gas is insufficient.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_state_cost(&mut self, cost: u64) -> bool {
        if self.reservoir >= cost {
            self.state_gas_spent = self.state_gas_spent.saturating_add(cost);
            self.reservoir -= cost;
            return true;
        }

        let spill = cost - self.reservoir;

        let success = self.record_regular_cost(spill);
        if success {
            self.state_gas_spent = self.state_gas_spent.saturating_add(cost);
            self.reservoir = 0;
        }
        success
    }

    /// Records a refund value.
    #[inline]
    pub const fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Erases a gas cost from remaining (returns gas from child frame).
    #[inline]
    pub const fn erase_cost(&mut self, returned: u64) {
        self.remaining += returned;
    }

    /// Spends all remaining gas excluding the reservoir.
    #[inline]
    pub const fn spend_all(&mut self) {
        self.remaining = 0;
    }
}

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

/// Init and floor gas from transaction
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialAndFloorGas {
    /// Initial gas for transaction.
    pub initial_total_gas: u64,
    /// State gas component of initial_gas (subset of initial_total_gas).
    /// Under EIP-8037, this includes:
    /// - EIP-7702 auth list state gas (per-auth account creation + metadata costs)
    /// - For CREATE transactions: `create_state_gas` (account creation + contract metadata)
    /// - For CALL transactions: 0 (state gas is unpredictable at validation time)
    pub initial_state_gas: u64,
    /// If transaction is a Call and Prague is enabled
    /// floor_gas is at least amount of gas that is going to be spent.
    pub floor_gas: u64,
    /// EIP-7702 state gas refund for existing authorities.
    /// Added to the reservoir after initial_state_gas is deducted.
    /// In the Python spec, set_delegation adds this back to state_gas_reservoir
    /// rather than reducing initial_state_gas, so the refunded gas stays as
    /// reservoir gas (not regular gas).
    pub eip7702_reservoir_refund: u64,
}

impl InitialAndFloorGas {
    /// Create a new InitialAndFloorGas instance.
    #[inline]
    pub const fn new(initial_total_gas: u64, floor_gas: u64) -> Self {
        Self {
            initial_total_gas,
            initial_state_gas: 0,
            floor_gas,
            eip7702_reservoir_refund: 0,
        }
    }

    /// Create a new InitialAndFloorGas instance with state gas tracking.
    #[inline]
    pub const fn new_with_state_gas(
        initial_total_gas: u64,
        initial_state_gas: u64,
        floor_gas: u64,
    ) -> Self {
        Self {
            initial_total_gas,
            initial_state_gas,
            floor_gas,
            eip7702_reservoir_refund: 0,
        }
    }

    /// Regular (non-state) portion of the initial intrinsic gas.
    ///
    /// Under EIP-8037, this is the part constrained by `TX_MAX_GAS_LIMIT`;
    /// state gas uses its own reservoir and is not subject to that cap.
    #[inline]
    pub const fn initial_regular_gas(&self) -> u64 {
        self.initial_total_gas - self.initial_state_gas
    }

    /// Computes the regular gas budget and reservoir for the initial call frame.
    ///
    /// EIP-8037 reservoir model:
    ///   execution_gas = tx.gas_limit - intrinsic_gas  (= gas_limit parameter)
    ///   regular_gas_budget = min(execution_gas, TX_MAX_GAS_LIMIT - intrinsic_gas)
    ///   reservoir = execution_gas - regular_gas_budget
    ///
    /// Initial state gas is then deducted from the reservoir (spilling into the
    /// regular budget when the reservoir is insufficient), and the EIP-7702
    /// refund for existing authorities is added back to the reservoir.
    ///
    /// On mainnet (state gas disabled), reservoir = 0 and gas_limit is unchanged.
    ///
    /// Returns `(gas_limit, reservoir)`.
    pub fn initial_gas_and_reservoir(
        &self,
        tx_gas_limit: u64,
        tx_gas_limit_cap: u64,
        is_eip8037: bool,
    ) -> (u64, u64) {
        let execution_gas = tx_gas_limit - self.initial_regular_gas();

        // System calls pass InitialAndFloorGas with all zeros and should not be
        // subject to the TX_MAX_GAS_LIMIT cap.
        let regular_gas_cap = if self.initial_total_gas == 0 {
            u64::MAX
        } else if is_eip8037 {
            tx_gas_limit_cap.saturating_sub(self.initial_regular_gas())
        } else {
            tx_gas_limit_cap
        };

        let mut gas_limit = core::cmp::min(execution_gas, regular_gas_cap);
        let mut reservoir = execution_gas - gas_limit;

        // Deduct initial state gas from the reservoir. When the reservoir is
        // insufficient, the deficit is charged from the regular gas budget.
        if self.initial_state_gas > 0 {
            if reservoir >= self.initial_state_gas {
                reservoir -= self.initial_state_gas;
            } else {
                gas_limit -= self.initial_state_gas - reservoir;
                reservoir = 0;
            }
        }

        // EIP-7702 state gas refund for existing authorities goes directly to
        // the reservoir. In the Python spec, set_delegation adds this refund to
        // state_gas_reservoir so it stays as state gas (not regular gas).
        if self.eip7702_reservoir_refund > 0 {
            reservoir += self.eip7702_reservoir_refund;
        }

        (gas_limit, reservoir)
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
    GasParams::new_spec(spec_id).initial_tx_gas(
        input,
        is_create,
        access_list_accounts,
        access_list_storages,
        authorization_list_num,
    )
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
pub fn get_tokens_in_calldata_istanbul(input: &[u8]) -> u64 {
    get_tokens_in_calldata(input, NON_ZERO_BYTE_MULTIPLIER_ISTANBUL)
}

/// Retrieve the total number of tokens in calldata.
#[inline]
pub fn get_tokens_in_calldata(input: &[u8], non_zero_data_multiplier: u64) -> u64 {
    let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
    let non_zero_data_len = input.len() as u64 - zero_data_len;
    zero_data_len + non_zero_data_len * non_zero_data_multiplier
}
