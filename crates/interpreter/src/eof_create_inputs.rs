use crate::primitives::{Address, Bytes, Eof, TransactTo, TxEnv, U256};
use core::ops::Range;
use std::boxed::Box;

/// Inputs for create call.
#[derive(Debug, Default, Clone)]
pub struct EofCreateInput {
    /// Caller of Eof Craate
    pub caller: Address,
    /// Values of ether transfered
    pub value: U256,
    /// Init eof code that is going to be executed.
    pub eof_init_code: Eof,
    /// Gas limit for the create call.
    pub gas_limit: u64,
    /// Created address,
    pub created_address: Address,
}

impl EofCreateInput {
    /// Returns a new instance of EofCreateInput.
    pub fn new(
        caller: Address,
        created_address: Address,
        value: U256,
        eof_init_code: Eof,
        gas_limit: u64,
    ) -> EofCreateInput {
        EofCreateInput {
            caller,
            value,
            eof_init_code,
            gas_limit,
            created_address,
        }
    }
}
