use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{AddressAndId, Bytes, U256};
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
pub fn create_init_frame(
    caller: AddressAndId,
    target: Option<(AddressAndId, Option<AddressAndId>)>,
    input: Bytes,
    value: U256,
    gas_limit: u64,
) -> FrameInput {
    if let Some((target, delegated_address)) = target {
        let (bytecode_address, is_bytecode_delegated) =
            if let Some(delegated_address) = delegated_address {
                (delegated_address, true)
            } else {
                (target, false)
            };
        FrameInput::Call(Box::new(CallInputs {
            input: CallInput::Bytes(input),
            gas_limit,
            target_address: target,
            bytecode_address,
            is_bytecode_delegated,
            caller,
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
