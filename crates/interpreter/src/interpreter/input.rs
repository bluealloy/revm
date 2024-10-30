pub use crate::interpreter_wiring::InputsTrait;
use primitives::{Address, Bytes, U256};

pub struct InputsImpl {
    target_address: Address,
    caller_address: Address,
    input: Bytes,
    call_value: U256,
}

impl InputsTrait for InputsImpl {
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
