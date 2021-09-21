use super::Spec;

#[derive(Clone)]
pub struct BerlinSpec;


impl Spec for BerlinSpec {
    /// Gas paid for extcode.
	const gas_ext_code: u64 = 0;
	/// Gas paid for extcodehash.
	const gas_ext_code_hash: u64 = 0;
	/// Gas paid for sstore set.
	const gas_sstore_set: u64 = 0;
	/// Gas paid for sstore reset.
	const gas_sstore_reset: u64 = 0;
	/// Gas paid for sstore refund.
	const refund_sstore_clears: i64 = 0;
	/// Gas paid for BALANCE opcode.
	const gas_balance: u64 = 0;
	/// Gas paid for SLOAD opcode.
	const gas_sload: u64 = 0;
	/// Gas paid for cold SLOAD opcode.
	const gas_sload_cold: u64 = 0;
	/// Gas paid for SUICIDE opcode.
	const gas_suicide: u64 = 0;
	/// Gas paid for SUICIDE opcode when it hits a new account.
	const gas_suicide_new_account: u64 = 0;
	/// Gas paid for CALL opcode.
	const gas_call: u64 = 0;
	/// Gas paid for EXP opcode for every byte.
	const gas_expbyte: u64 = 0;
	/// Gas paid for a contract creation transaction.
	const gas_transaction_create: u64 = 0;
	/// Gas paid for a message call transaction.
	const gas_transaction_call: u64 = 0;
	/// Gas paid for zero data in a transaction.
	const gas_transaction_zero_data: u64 = 0;
	/// Gas paid for non-zero data in a transaction.
	const gas_transaction_non_zero_data: u64 = 0;
	/// Gas paid per address in transaction access list (see EIP-2930).
	const gas_access_list_address: u64 = 0;
	/// Gas paid per storage key in transaction access list (see EIP-2930).
	const gas_access_list_storage_key: u64 = 0;
	/// Gas paid for accessing cold account.
	const gas_account_access_cold: u64 = 0;
	/// Gas paid for accessing ready storage.
	const gas_storage_read_warm: u64 = 0;
	/// EIP-1283.
	const sstore_gas_metering: bool = false;
	/// EIP-1706.
	const sstore_revert_under_stipend: bool = false;
	/// EIP-2929
	const increase_state_access_gas: bool = false;
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	const err_on_call_with_more_gas: bool = false;
	/// Take l64 for callcreate after gas.
	const call_l64_after_gas: bool = false;
	/// Whether empty account is considered exists.
	const empty_considered_exists: bool = false;
	/// Whether create transactions and create opcode increases nonce by one.
	const create_increase_nonce: bool = false;
	/// Stack limit.
	const stack_limit: usize = 0;
	/// Memory limit.
	const memory_limit: usize = 0;
	/// Call limit.
	const call_stack_limit: usize = 0;
	/// Create contract limit. TODO set usize to MAX
	const create_contract_limit: Option<usize> = None;
	/// Call stipend.
	const call_stipend: u64 = 0;
	/// Has delegate call.
	const has_delegate_call: bool = false;
	/// Has create2.
	const has_create2: bool = false;
	/// Has revert.
	const has_revert: bool = false;
	/// Has return data.
	const has_return_data: bool = false;
	/// Has bitwise shifting.
	const has_bitwise_shifting: bool = false;
	/// Has chain ID.
	const has_chain_id: bool = false;
	/// Has self balance.
	const has_self_balance: bool = false;
	/// Has ext code hash.
	const has_ext_code_hash: bool = false;
	/// Whether the gasometer is running in estimate mode.
	const estimate: bool = false;
} 