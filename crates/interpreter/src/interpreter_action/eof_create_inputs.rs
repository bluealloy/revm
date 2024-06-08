use crate::primitives::{eof::EofDecodeError, Address, Bytes, Eof, TxEnv, U256};
use std::boxed::Box;

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
    /// Call data the input of the EOFCREATE call.
    pub input: Bytes,
    /// Gas limit for the create call.
    pub gas_limit: u64,
}

impl EOFCreateInput {
    /// Returns boxed EOFCreateInput or error.
    /// Internally calls [`Self::new_tx`].
    pub fn new_tx_boxed(tx: &TxEnv, nonce: u64) -> Result<Box<Self>, EofDecodeError> {
        Ok(Box::new(Self::new_tx(tx, nonce)?))
    }

    /// Create new EOF crate input from transaction that has concatenated eof init code and calldata.
    ///
    /// Legacy transaction still have optional nonce so we need to obtain it.
    pub fn new_tx(tx: &TxEnv, nonce: u64) -> Result<Self, EofDecodeError> {
        let (eof_init_code, input) = Eof::decode_dangling(tx.data.clone())?;
        Ok(EOFCreateInput {
            caller: tx.caller,
            created_address: tx.caller.create(nonce),
            value: tx.value,
            eof_init_code,
            gas_limit: tx.gas_limit,
            input,
        })
    }

    /// Returns a new instance of EOFCreateInput.
    pub fn new(
        caller: Address,
        created_address: Address,
        value: U256,
        eof_init_code: Eof,
        gas_limit: u64,
        input: Bytes,
    ) -> EOFCreateInput {
        EOFCreateInput {
            caller,
            created_address,
            value,
            eof_init_code,
            gas_limit,
            input,
        }
    }
}
