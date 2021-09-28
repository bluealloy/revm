
pub trait NotStaticSpec {}

pub trait Spec {
    /// litle bit of magic. We can have child version of Spec that contains static flag enabled
    type STATIC: Spec;
    /// static flag used in STATIC type;
    const IS_STATIC_CALL: bool;
    /// Gas paid for extcode.
    const GAS_EXT_CODE: u64;
    /// Gas paid for extcodehash.
    const GAS_EXT_CODE_HASH: u64;
    /// Gas paid for sstore set.
    const GAS_SSTORE_SET: u64;
    /// Gas paid for sstore reset.
    const GAS_SSTORE_RESET: u64;
    /// Gas paid for sstore refund.
    const REFUND_SSTORE_CLEARS: i64;
    /// Gas paid for BALANCE opcode.
    const GAS_BALANCE: u64;
    /// Gas paid for SLOAD opcode.
    const GAS_SLOAD: u64;
    /// Gas paid for cold SLOAD opcode.
    const GAS_SLOAD_COLD: u64;
    /// Gas paid for SELFDESTRUCT opcode.
    const GAS_SELFDESTRUCT: u64;
    /// Gas paid for SELFDESTRUCT opcode when it hits a new account.
    const GAS_SELFDESTRUCT_NEW_ACCOUNT: u64;
    /// Gas paid for CALL opcode.
    const GAS_CALL: u64;
    /// Gas paid for EXP opcode for every byte.
    const GAS_EXPBYTE: u64;
    /// Gas paid for a contract creation transaction.
    const GAS_TRANSACTION_CREATE: u64;
    /// Gas paid for a message call transaction.
    const GAS_TRANSACTION_CALL: u64;
    /// Gas paid for zero data in a transaction.
    const GAS_TRANSACTION_ZERO_DATA: u64;
    /// Gas paid for non-zero data in a transaction.
    const GAS_TRANSACTION_NON_ZERO_DATA: u64;
    /// Gas paid per address in transaction access list (see EIP-2930).
    const GAS_ACCESS_LIST_ADDRESS: u64;
    /// Gas paid per storage key in transaction access list (see EIP-2930).
    const GAS_ACCESS_LIST_STORAGE_KEY: u64;
    /// Gas paid for accessing cold account.
    const GAS_ACCOUNT_ACCESS_COLD: u64;
    /// Gas paid for accessing ready storage.
    const GAS_STORAGE_READ_WARM: u64;
    /// EIP-1283.
    const SSTORE_GAS_METERING: bool;
    /// EIP-1706.
    const SSTORE_REVERT_UNDER_STIPEND: bool;
    /// EIP-2929
    const INCREASE_STATE_ACCESS_GAS: bool;
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    const ERR_ON_CALL_WITH_MORE_GAS: bool;
    /// Take l64 for callcreate after gas.
    const CALL_L64_AFTER_GAS: bool;
    /// Whether empty account is considered exists.
    const EMPTY_CONSIDERED_EXISTS: bool;
    /// Whether create transactions and create opcode increases nonce by one.
    const CREATE_INCREASE_NONCE: bool;
    /// Stack limit.
    const STACK_LIMIT: usize;
    /// Memory limit.
    const MEMORY_LIMIT: usize;
    /// Call limit.
    const CALL_STACK_LIMIT: usize;
    /// Create contract limit.
    const CREATE_CONTRACT_LIMIT: Option<usize>;
    /// Call stipend.
    const CALL_STIPEND: u64;
    /// Has delegate call.
    const HAS_DELEGATE_CALL: bool;
    /// Has create2.
    const HAS_CREATE2: bool;
    /// Has revert.
    const HAS_REVERT: bool;
    /// Has return data.
    const HAS_RETURN_DATA: bool;
    /// Has bitwise shifting.
    const HAS_BITWISE_SHIFTING: bool;
    /// Has chain ID.
    const HAS_CHAIN_ID: bool;
    /// Has self balance.
    const HAS_SELF_BALANCE: bool;
    /// Has ext code hash.
    const HAS_EXT_CODE_HASH: bool;
    /// Whether the gasometer is running in estimate mode.
    const ESTIMATE: bool;
}
