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
/// Gas cost for SELFDESTRUCT instruction.
pub const SELFDESTRUCT: i64 = 24000;
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
// The cost floor per token as defined by [EIP-2028](https://eips.ethereum.org/EIPS/eip-2028).
/// The cost floor per token as defined by EIP-2028.
pub const TOTAL_COST_FLOOR_PER_TOKEN: u64 = 10;

/// Gas cost for EOF CREATE instruction.
pub const EOF_CREATE_GAS: u64 = 32000;

// Berlin eip2929 constants
/// Gas cost for accessing an address in the access list (EIP-2929).
pub const ACCESS_LIST_ADDRESS: u64 = 2400;
/// Gas cost for accessing a storage key in the access list (EIP-2929).
pub const ACCESS_LIST_STORAGE_KEY: u64 = 1900;
/// Gas cost for SLOAD when accessing a cold storage slot (EIP-2929).
pub const COLD_SLOAD_COST: u64 = 2100;
/// Gas cost for accessing a cold account (EIP-2929).
pub const COLD_ACCOUNT_ACCESS_COST: u64 = 2600;
/// Gas cost for reading from a warm storage slot (EIP-2929).
pub const WARM_STORAGE_READ_COST: u64 = 100;
/// Gas cost for SSTORE reset operation on a warm storage slot.
pub const WARM_SSTORE_RESET: u64 = SSTORE_RESET - COLD_SLOAD_COST;

/// EIP-3860 : Limit and meter initcode
pub const INITCODE_WORD_COST: u64 = 2;

/// Gas stipend provided to the recipient of a CALL with value transfer.
pub const CALL_STIPEND: u64 = 2300;
/// Minimum gas that must be provided to a callee.
pub const MIN_CALLEE_GAS: u64 = CALL_STIPEND;

/// A fuel denomination rate for rWasm vs EVM opcodes
pub const FUEL_DENOM_RATE: u64 = fluentbase_sdk::FUEL_DENOM_RATE;
