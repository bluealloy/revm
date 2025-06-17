use context_interface::Transaction;
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{hardfork::SpecId, TxKind};
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
pub fn create_init_frame(tx: &impl Transaction, _spec: SpecId, gas_limit: u64) -> FrameInput {
    let input = tx.input().clone();

    match tx.kind() {
        TxKind::Call(target_address) => FrameInput::Call(Box::new(CallInputs {
            input: CallInput::Bytes(input),
            gas_limit,
            target_address,
            bytecode_address: target_address,
            caller: tx.caller(),
            value: CallValue::Transfer(tx.value()),
            scheme: CallScheme::Call,
            is_static: false,
            is_eof: false,
            return_memory_offset: 0..0,
        })),
        TxKind::Create => {
            // Since EOF support has been removed, all creates are legacy creates
            FrameInput::Create(Box::new(CreateInputs {
                caller: tx.caller(),
                scheme: CreateScheme::Create,
                value: tx.value(),
                init_code: input,
                gas_limit,
            }))
        }
    }
}
