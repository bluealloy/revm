use crate::primitives::{Address, Eof, U256};
use core::ops::Range;

/// Inputs for EOF create call.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EOFCreateInput {
    /// Caller of Eof Craate
    pub caller: Address,
    /// New contract address.
    pub created_address: Address,
    /// Values of ether transfered
    pub value: U256,
    /// Init eof code that is going to be executed.
    pub eof_init_code: Eof,
    /// Gas limit for the create call.
    pub gas_limit: u64,
    /// Return memory range. If EOF creation Reverts it can return the
    /// the memory range.
    pub return_memory_range: Range<usize>,
}

impl EOFCreateInput {
    /// Returns a new instance of EOFCreateInput.
    pub fn new(
        caller: Address,
        created_address: Address,
        value: U256,
        eof_init_code: Eof,
        gas_limit: u64,
        return_memory_range: Range<usize>,
    ) -> EOFCreateInput {
        EOFCreateInput {
            caller,
            created_address,
            value,
            eof_init_code,
            gas_limit,
            return_memory_range,
        }
    }
}
