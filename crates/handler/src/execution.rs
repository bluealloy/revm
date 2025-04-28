use bytecode::EOF_MAGIC_BYTES;
use context_interface::Transaction;
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, EOFCreateInputs,
    EOFCreateKind, FrameInput,
};
use primitives::{hardfork::SpecId, TxKind};
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
pub fn create_init_frame(tx: &impl Transaction, spec: SpecId, gas_limit: u64) -> FrameInput {
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
            // If first byte of data is magic 0xEF00, then it is EOFCreate.
            if spec.is_enabled_in(SpecId::OSAKA) && input.starts_with(&EOF_MAGIC_BYTES) {
                FrameInput::EOFCreate(Box::new(EOFCreateInputs::new(
                    tx.caller(),
                    tx.value(),
                    gas_limit,
                    EOFCreateKind::Tx { initdata: input },
                )))
            } else {
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
}
