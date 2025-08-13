use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{AddressAndId, Bytes, U256};
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
pub fn create_init_frame(
    is_call: bool,
    caller: AddressAndId,
    target: AddressAndId,
    delegated_address: Option<AddressAndId>,
    input: Bytes,
    value: U256,
    gas_limit: u64,
) -> FrameInput {
    if is_call {
        let bytecode_address = if let Some(delegated_address) = delegated_address {
            delegated_address
        } else {
            target
        };
        FrameInput::Call(Box::new(CallInputs {
            input: CallInput::Bytes(input),
            gas_limit,
            target_address: target,
            bytecode_address,
            caller: caller,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: false,
            return_memory_offset: 0..0,
        }))
    } else {
        FrameInput::Create(Box::new(CreateInputs {
            caller,
            scheme: CreateScheme::Create,
            value,
            init_code: input,
            gas_limit,
        }))
    }
}
