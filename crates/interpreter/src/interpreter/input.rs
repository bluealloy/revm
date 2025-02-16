use crate::interpreter_types::InputsTr;
use primitives::{Address, Bytes, U256};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputsImpl {
    pub target_address: Address,
    pub caller_address: Address,
    pub input: Bytes,
    pub call_value: U256,
}

impl InputsTr for InputsImpl {
    fn target_address(&self) -> Address {
        self.target_address
    }

    fn caller_address(&self) -> Address {
        self.caller_address
    }

    fn input(&self) -> &[u8] {
        &self.input
    }

    fn call_value(&self) -> U256 {
        self.call_value
    }
}
