use crate::{NotStaticSpec, precompiles::Precompiles};

use super::Spec;

#[derive(Clone)]
pub struct BerlinSpecImpl<const STATIC_CALL: bool>;


pub type BerlinSpec = BerlinSpecImpl<false>;
pub type BerlinSpecStatic = BerlinSpecImpl<true>;


impl NotStaticSpec for BerlinSpec {}


impl<const IS_STATIC_CALL: bool> Spec for BerlinSpecImpl<IS_STATIC_CALL> {
    type STATIC = BerlinSpecImpl<true>;
    
    //specification id
    const SPEC_ID: u8 = super::spec::BERLIN;
    //precompiles
    fn precompiles() -> Precompiles {
        Precompiles::new_berlin()
    }

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
    /// Gas paid for extcode.
    const GAS_EXT_CODE: u64 = 0;
    /// Gas paid for extcodehash.
    const GAS_EXT_CODE_HASH: u64 = 0;
    /// Gas paid for sstore set.
    const GAS_SSTORE_SET: u64 = 20000;
    /// Gas paid for sstore reset.
    const GAS_SSTORE_RESET: u64 = 5000;
    /// Gas paid for sstore refund.
    const REFUND_SSTORE_CLEARS: i64 = 15000;
    /// Gas paid for BALANCE opcode.
    const GAS_BALANCE: u64 = 0;
    /// Gas paid for SLOAD opcode.
    const GAS_SLOAD: u64 = 0;
    /// Gas paid for cold SLOAD opcode.
    const GAS_SLOAD_COLD: u64 = 2100;
    /// Gas paid for SELFDESTRUCT opcode.
    const GAS_SELFDESTRUCT: u64 = 5000;
    /// Gas paid for SELFDESTRUCT opcode when it hits a new account.
    const GAS_SELFDESTRUCT_NEW_ACCOUNT: u64 = 25000;
    /// Gas paid for CALL opcode.
    const GAS_CALL: u64 = 0;
    /// Gas paid for EXP opcode for every byte.
    const GAS_EXPBYTE: u64 = 50;
    /// Gas paid for a contract creation transaction.
    const GAS_TRANSACTION_CREATE: u64 = 53000;
    /// Gas paid for a message call transaction.
    const GAS_TRANSACTION_CALL: u64 = 21000;
    /// Gas paid for zero data in a transaction.
    const GAS_TRANSACTION_ZERO_DATA: u64 = 4;
    /// Gas paid for non-zero data in a transaction.
    const GAS_TRANSACTION_NON_ZERO_DATA: u64 = 16;
    /// Gas paid per address in transaction access list (see EIP-2930).
    const GAS_ACCESS_LIST_ADDRESS: u64 = 2400;
    /// Gas paid per storage key in transaction access list (see EIP-2930).
    const GAS_ACCESS_LIST_STORAGE_KEY: u64 = 1900;
    /// Gas paid for accessing cold account.
    const GAS_ACCOUNT_ACCESS_COLD: u64 = 2600;
    /// Gas paid for accessing ready storage.
    const GAS_STORAGE_READ_WARM: u64 = 100;
    /// EIP-1283.
    const SSTORE_GAS_METERING: bool = true;
    /// EIP-1706.
    const SSTORE_REVERT_UNDER_STIPEND: bool = true;
    /// EIP-2929
    const INCREASE_STATE_ACCESS_GAS: bool = true;
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    const ERR_ON_CALL_WITH_MORE_GAS: bool = false;
    /// Take l64 for callcreate after gas.
    const CALL_L64_AFTER_GAS: bool = false;
    /// Whether empty account is considered exists.
    const EMPTY_CONSIDERED_EXISTS: bool = true;
    /// Whether create transactions and create opcode increases nonce by one.
    const CREATE_INCREASE_NONCE: bool = true;
    /// Stack limit.
    const STACK_LIMIT: usize = 1024;
    /// Memory limit.
    const MEMORY_LIMIT: usize = usize::MAX;
    /// Call limit.
    const CALL_STACK_LIMIT: usize = 1024;
    /// Create contract limit. TODO set usize to MAX
    const CREATE_CONTRACT_LIMIT: Option<usize> = Some(0x6000);
    /// Call stipend.
    const CALL_STIPEND: u64 = 2300;
    /// Has delegate call.
    const HAS_DELEGATE_CALL: bool = true;
    /// Has create2.
    const HAS_CREATE2: bool = true;
    /// Has revert.
    const HAS_REVERT: bool = true;
    /// Has return data.
    const HAS_RETURN_DATA: bool = true;
    /// Has bitwise shifting.
    const HAS_BITWISE_SHIFTING: bool = true;
    /// Has chain ID.
    const HAS_CHAIN_ID: bool = true;
    /// Has self balance.
    const HAS_SELF_BALANCE: bool = true;
    /// Has ext code hash.
    const HAS_EXT_CODE_HASH: bool = true;
    /// Whether the gasometer is running in estimate mode.
    const ESTIMATE: bool = false;
}
