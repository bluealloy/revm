use context::{ContextTr, Database, JournalTr};
use context_interface::Transaction;
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::TxKind;
use state::Bytecode;
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
#[inline]
pub fn create_init_frame<CTX: ContextTr>(
    ctx: &mut CTX,
    gas_limit: u64,
    reservoir: u64,
) -> Result<FrameInput, <<CTX::Journal as JournalTr>::Database as Database>::Error> {
    let (tx, journal) = ctx.tx_journal_mut();
    let input = tx.input().clone();

    match tx.kind() {
        TxKind::Call(target_address) => {
            let account = &journal.load_account_with_code(target_address)?.info;

            let known_bytecode = if let Some(delegated_address) =
                account.code.as_ref().and_then(Bytecode::eip7702_address)
            {
                let account = &journal.load_account_with_code(delegated_address)?.info;
                (
                    account.code_hash(),
                    account.code.clone().unwrap_or_default(),
                )
            } else {
                (
                    account.code_hash(),
                    account.code.clone().unwrap_or_default(),
                )
            };
            Ok(FrameInput::Call(Box::new(CallInputs {
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
                reservoir,
            })))
        }
        TxKind::Create => Ok(FrameInput::Create(Box::new(CreateInputs::new(
            tx.caller(),
            CreateScheme::Create,
            tx.value(),
            input,
            gas_limit,
            reservoir,
        )))),
    }
}
