use context_interface::Transaction;
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{TxKind, B256};
use state::Bytecode;
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
#[inline]
pub fn create_init_frame(
    tx: &impl Transaction,
    bytecode: Option<(Bytecode, B256)>,
    gas_limit: u64,
) -> FrameInput {
    let input = tx.input().clone();

    match tx.kind() {
        TxKind::Call(target_address) => {
            let known_bytecode = bytecode.map(|(code, hash)| (hash, code));
            FrameInput::Call(Box::new(CallInputs {
                input: CallInput::Bytes(input),
                gas_limit,
                target_address,
                bytecode_address: target_address,
                known_bytecode,
                caller: tx.caller(),
                value: CallValue::Transfer(tx.value()),
                scheme: CallScheme::Call,
                is_static: false,
                return_memory_offset: 0..0,
            }))
        }
        TxKind::Create => FrameInput::Create(Box::new(CreateInputs::new(
            tx.caller(),
            CreateScheme::Create,
            tx.value(),
            input,
            gas_limit,
        ))),
    }
}
