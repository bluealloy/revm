use primitive_types::{H160, H256, U256};

use crate::{Context, CreateScheme, Transfer, error::{ExitError, ExitReason}, opcode::OpCode, stack::Stack};

/// EVM context handler.
pub trait Handler {

	/// Get balance of address.
	fn balance(&self, address: H160) -> U256;
	/// Get code size of address.
	fn code_size(&self, address: H160) -> U256;
	/// Get code hash of address.
	fn code_hash(&self, address: H160) -> H256;
	/// Get code of address.
	fn code(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	fn storage(&self, address: H160, index: H256) -> H256;
	/// Get original storage value of address at index.
	fn original_storage(&self, address: H160, index: H256) -> H256;

	/// Get the gas left value.
	fn gas_left(&self) -> U256;
	/// Get the gas price value.
	fn gas_price(&self) -> U256;
	/// Get execution origin.
	fn origin(&self) -> H160;
	/// Get environmental block hash.
	fn block_hash(&self, number: U256) -> H256;
	/// Get environmental block number.
	fn block_number(&self) -> U256;
	/// Get environmental coinbase.
	fn block_coinbase(&self) -> H160;
	/// Get environmental block timestamp.
	fn block_timestamp(&self) -> U256;
	/// Get environmental block difficulty.
	fn block_difficulty(&self) -> U256;
	/// Get environmental gas limit.
	fn block_gas_limit(&self) -> U256;
	/// Get environmental chain ID.
	fn chain_id(&self) -> U256;

	/// Check whether an address exists.
	fn exists(&self, address: H160) -> bool;
	/// Check whether an address has already been deleted.
	fn deleted(&self, address: H160) -> bool;
	/// Checks if the address or (address, index) pair has been previously accessed
	/// (or set in `accessed_addresses` / `accessed_storage_keys` via an access list
	/// transaction).
	/// References:
	/// * https://eips.ethereum.org/EIPS/eip-2929
	/// * https://eips.ethereum.org/EIPS/eip-2930
	fn is_cold(&self, address: H160, index: Option<H256>) -> bool;

	/// Set storage value of address at index.
	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError>;
	/// Create a log owned by address with given topics and data.
	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError>;
	/// Mark an address to be deleted, with funds transferred to target.
	fn mark_delete<const CALL_TRACE: bool>(
		&mut self,
		address: H160,
		target: H160,
	) -> Result<(), ExitError>;
	/// Invoke a create operation.
	fn create<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> (ExitReason, Option<H160>, Vec<u8>);

	/// Invoke a call operation.
	fn call<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> (ExitReason, Vec<u8>);

	/// Pre-validation step for the gasometer. Used to calculate gas cost
	fn pre_validate<const GAS_PRICE: bool>(
		&mut self,
		context: &Context,
		opcode: OpCode,
		stack: &Stack,
	) -> Result<(), ExitError>;
}
