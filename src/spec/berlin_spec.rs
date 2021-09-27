use super::Spec;

#[derive(Clone)]
pub struct BerlinSpecTemp<const STATIC_CALL: bool>;

pub type BerlinSpec = BerlinSpecTemp<true>;
pub type BerlinSpecStatic = BerlinSpecTemp<false>;

impl<const STATIC_CALL: bool> Spec for BerlinSpecTemp<STATIC_CALL> {
    const is_not_static_call: bool = STATIC_CALL;
    /// Gas paid for extcode.
    const gas_ext_code: u64 = 0;
    /// Gas paid for extcodehash.
    const gas_ext_code_hash: u64 = 0;
    /// Gas paid for sstore set.
    const gas_sstore_set: u64 = 20000;
    /// Gas paid for sstore reset.
    const gas_sstore_reset: u64 = 5000;
    /// Gas paid for sstore refund.
    const refund_sstore_clears: i64 = 15000;
    /// Gas paid for BALANCE opcode.
    const gas_balance: u64 = 0;
    /// Gas paid for SLOAD opcode.
    const gas_sload: u64 = 0;
    /// Gas paid for cold SLOAD opcode.
    const gas_sload_cold: u64 = 2100;
    /// Gas paid for SUICIDE opcode.
    const gas_suicide: u64 = 5000;
    /// Gas paid for SUICIDE opcode when it hits a new account.
    const gas_suicide_new_account: u64 = 25000;
    /// Gas paid for CALL opcode.
    const gas_call: u64 = 0;
    /// Gas paid for EXP opcode for every byte.
    const gas_expbyte: u64 = 50;
    /// Gas paid for a contract creation transaction.
    const gas_transaction_create: u64 = 53000;
    /// Gas paid for a message call transaction.
    const gas_transaction_call: u64 = 21000;
    /// Gas paid for zero data in a transaction.
    const gas_transaction_zero_data: u64 = 4;
    /// Gas paid for non-zero data in a transaction.
    const gas_transaction_non_zero_data: u64 = 16;
    /// Gas paid per address in transaction access list (see EIP-2930).
    const gas_access_list_address: u64 = 2400;
    /// Gas paid per storage key in transaction access list (see EIP-2930).
    const gas_access_list_storage_key: u64 = 1900;
    /// Gas paid for accessing cold account.
    const gas_account_access_cold: u64 = 2600;
    /// Gas paid for accessing ready storage.
    const gas_storage_read_warm: u64 = 100;
    /// EIP-1283.
    const sstore_gas_metering: bool = true;
    /// EIP-1706.
    const sstore_revert_under_stipend: bool = true;
    /// EIP-2929
    const increase_state_access_gas: bool = true;
    /// Whether to throw out of gas error when
    /// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    /// of gas.
    const err_on_call_with_more_gas: bool = false;
    /// Take l64 for callcreate after gas.
    const call_l64_after_gas: bool = false;
    /// Whether empty account is considered exists.
    const empty_considered_exists: bool = true;
    /// Whether create transactions and create opcode increases nonce by one.
    const create_increase_nonce: bool = true;
    /// Stack limit.
    const stack_limit: usize = 1024;
    /// Memory limit.
    const memory_limit: usize = usize::MAX;
    /// Call limit.
    const call_stack_limit: usize = 1024;
    /// Create contract limit. TODO set usize to MAX
    const create_contract_limit: Option<usize> = Some(0x6000);
    /// Call stipend.
    const call_stipend: u64 = 2300;
    /// Has delegate call.
    const has_delegate_call: bool = true;
    /// Has create2.
    const has_create2: bool = true;
    /// Has revert.
    const has_revert: bool = true;
    /// Has return data.
    const has_return_data: bool = true;
    /// Has bitwise shifting.
    const has_bitwise_shifting: bool = true;
    /// Has chain ID.
    const has_chain_id: bool = true;
    /// Has self balance.
    const has_self_balance: bool = true;
    /// Has ext code hash.
    const has_ext_code_hash: bool = true;
    /// Whether the gasometer is running in estimate mode.
    const estimate: bool = false;
}
