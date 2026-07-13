use crate::FrameResult;
use context::{ContextTr, Database, JournalTr};
use context_interface::Transaction;
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
    InitialAndFloorGas,
};
use primitives::TxKind;
use state::Bytecode;
use std::boxed::Box;

/// Deducts the EIP-2780 refundable first-frame state gas
/// ([`InitialAndFloorGas::refundable_state_gas`]) from the first frame's
/// budget: the reservoir is drawn first and the deficit spills into the
/// regular gas budget, mirroring `record_state_cost`. The charge was the last
/// state charge of the runtime gas phase, so deducting it on top of the
/// aggregate initial state gas reproduces the split that phase verified;
/// affordability failures set `runtime_oog` there instead of reaching this
/// point.
///
/// Returns the spilled portion, which [`settle_refundable_state_gas`] needs to
/// restore the pools correctly when the charge is refunded.
pub fn deduct_refundable_state_gas(
    init_and_floor_gas: &InitialAndFloorGas,
    gas_limit: &mut u64,
    reservoir: &mut u64,
) -> u64 {
    let charge = init_and_floor_gas.refundable_state_gas;
    let drawn = core::cmp::min(*reservoir, charge);
    let spill = charge - drawn;
    *reservoir -= drawn;
    *gas_limit = gas_limit.saturating_sub(spill);
    spill
}

/// Seeds the first frame's state-gas counters with the EIP-2780 refundable
/// charge that [`deduct_refundable_state_gas`] took from the frame's budget,
/// so `last_frame_result` settles it like a charge the frame recorded itself:
/// on success the seeded `state_gas_spent` is reported as spent state gas; on
/// revert or halt `rollback_state_gas` restores it in LIFO order (the spilled
/// portion back to `remaining` — consumed by a halt — and the rest to the
/// reservoir).
pub const fn settle_refundable_state_gas(charge: u64, spill: u64, frame_result: &mut FrameResult) {
    if charge == 0 {
        return;
    }
    // A create that returns success without deploying (e.g. sender nonce
    // overflow at frame creation) never created the account leaf, but its
    // success result skips `last_frame_result`'s rollback — roll the charge
    // back directly. Its early-fail gas carries no other state charges, so
    // the rollback undoes exactly the seeded charge.
    let refund_now = matches!(
        frame_result,
        FrameResult::Create(outcome)
            if outcome.instruction_result().is_ok() && outcome.address.is_none()
    );
    let gas = frame_result.gas_mut();
    gas.set_state_gas_spent(gas.state_gas_spent() + charge as i64);
    gas.add_state_gas_spilled(spill);
    if refund_now {
        gas.rollback_state_gas();
    }
}

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
                charged_new_account_state_gas: false,
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
