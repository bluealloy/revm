use crate::FrameResult;
use context::{ContextTr, Database, JournalTr};
use context_interface::{Cfg, Transaction};
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput, Gas,
};
use primitives::TxKind;
use state::Bytecode;
use std::boxed::Box;

/// Refunds the EIP-2780 refundable first-frame state gas from the
/// transaction-level gas when the first frame did not create the account leaf
/// the charge paid for, mirroring how `EthFrame::return_result` refunds the
/// upfront CALL/CREATE state charges of inner frames from the outcome's
/// `charged_*` flags.
///
/// The charge was recorded on the transaction-level gas by the EIP-2780
/// runtime gas phase as its last state charge. On success without deployment
/// (a create that early-failed with `address == None`) or on revert it is
/// fully refunded in LIFO order ([`Gas::refill_reservoir`]): the spilled
/// portion returns to `remaining` and the rest to the reservoir. On an
/// exceptional halt the regular gas is consumed, so the spilled portion is
/// consumed with it and only the portion drawn from the reservoir is
/// restored.
pub fn refund_refundable_state_gas<CTX: ContextTr>(
    ctx: &CTX,
    frame_result: &FrameResult,
    gas: &mut Gas,
) {
    let instruction_result = frame_result.instruction_result();
    let charge = match frame_result {
        FrameResult::Call(outcome) => {
            if instruction_result.is_ok() || !outcome.charged_new_account_state_gas {
                return;
            }
            ctx.cfg().gas_params().new_account_state_gas()
        }
        FrameResult::Create(outcome) => {
            let create_failed = outcome.address.is_none() || !instruction_result.is_ok();
            if !create_failed || !outcome.charged_create_state_gas {
                return;
            }
            ctx.cfg().gas_params().create_state_gas()
        }
    };

    if instruction_result.is_ok_or_revert() {
        gas.refill_reservoir(charge);
    } else {
        // Exceptional halt: the spilled portion of the charge is consumed
        // along with the regular gas; restore only the reservoir-drawn part.
        let spill = charge.min(gas.state_gas_spilled());
        gas.set_state_gas_spilled(gas.state_gas_spilled() - spill);
        gas.set_reservoir(gas.reservoir().saturating_add(charge - spill));
        gas.set_state_gas_spent(gas.state_gas_spent() - charge as i64);
    }
}

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
///
/// `charged_refundable_state_gas` marks that the EIP-2780 runtime gas phase
/// charged the refundable first-frame state gas; it is carried on the frame
/// inputs like the `charged_*` flags of the CALL/CREATE opcodes.
#[inline]
pub fn create_init_frame<CTX: ContextTr>(
    ctx: &mut CTX,
    gas_limit: u64,
    reservoir: u64,
    charged_refundable_state_gas: bool,
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
                charged_new_account_state_gas: charged_refundable_state_gas,
            })))
        }
        TxKind::Create => {
            let mut inputs = CreateInputs::new(
                tx.caller(),
                CreateScheme::Create,
                tx.value(),
                input,
                gas_limit,
                reservoir,
            );
            inputs.set_charged_create_state_gas(charged_refundable_state_gas);
            Ok(FrameInput::Create(Box::new(inputs)))
        }
    }
}
