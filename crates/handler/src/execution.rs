use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{AddressAndId, Bytes, U256};
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
#[inline]
pub fn create_init_frame(
    is_call: bool,
    caller: AddressAndId,
    target: AddressAndId,
    input: Bytes,
    value: U256,
    gas_limit: u64,
) -> FrameInput {
    if is_call {
        FrameInput::Call(CallInputs {
            input: CallInput::Bytes(input),
            gas_limit,
            target_address: target,
            bytecode_address: target,
            caller: caller,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: false,
            return_memory_offset: 0..0,
        })
    } else {
        FrameInput::Create(CreateInputs {
            caller,
            scheme: CreateScheme::Create,
            value,
            init_code: input,
            gas_limit,
        })
    }
}
