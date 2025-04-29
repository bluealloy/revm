use crate::{interpreter_types::InputsTr, CallInput};
use primitives::{Address, U256};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputsImpl {
    pub target_address: Address,
    pub caller_address: Address,
    pub input: CallInput,
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
        match &self.input {
            CallInput::SharedBuffer { range, buffer } => {
                // Get slice from parent memory using range
                todo!("Implement memory range access")
            }
            CallInput::Bytes(bytes) => bytes.as_ref(),
        }
    }

    fn call_value(&self) -> U256 {
        self.call_value
    }
}
