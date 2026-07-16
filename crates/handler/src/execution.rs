use context::{ContextTr, Database, JournalTr};
use context_interface::{
    journaled_state::{account::JournaledAccountTr, JournalCheckpoint, JournalLoadError},
    Cfg, Transaction,
};
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
    GasTracker,
};
use primitives::TxKind;
use state::Bytecode;
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction and the
/// transaction-level `gas`, forwarding all remaining regular gas and the
/// reservoir to the frame.
///
/// Under EIP-2780 this completes the runtime gas phase started at
/// pre-execution (the authorization charges): the target is loaded once and
/// its state-dependent charges are recorded on `gas`:
///
/// - A value transfer to a non-existent recipient grows the state by one
///   account leaf and pays the new-account state gas. The charge stays
///   deducted and only the decision travels on the frame inputs
///   (`charged_new_account_state_gas`), so the handler refunds it at frame
///   settle when no account leaf ends up created. Checked before the
///   delegation resolution, matching the spec's `prepare_dispatch` order.
/// - An EIP-7702 delegated recipient pays the delegation-target access
///   following the standard EIP-2929 warm/cold model. The charge must be
///   covered before the delegate is read, so an out-of-gas here leaves a cold
///   delegate out of the EIP-7928 block access list.
/// - A create transaction whose deployment target does not already exist pays
///   the account-creation state gas (`charged_create_state_gas`), decided by
///   the single read that also warms the target.
///
/// The recipient is evaluated after the authorizations were applied, so a
/// recipient materialized or delegated by an authorization in this
/// transaction is seen in its post-authorization state. A self-transfer
/// recipient is the (non-empty) sender, so the new-account charge never fires
/// for it, while a delegated sender still pays for resolving its delegation.
///
/// Returns `None` when the charges run out of gas: the transaction stays
/// valid but must be included as an out-of-gas halt without entering
/// execution ([`runtime_oog_unwind`]).
#[inline]
pub fn create_init_frame<CTX: ContextTr>(
    ctx: &mut CTX,
    gas: &mut GasTracker,
) -> Result<Option<FrameInput>, <<CTX::Journal as JournalTr>::Database as Database>::Error> {
    let is_eip2780 = ctx.cfg().is_amsterdam_eip2780_enabled();
    let params = ctx.cfg().gas_params();
    let new_account_state_gas = params.new_account_state_gas();
    let create_state_gas = params.create_state_gas();
    let warm_access_cost = params.warm_storage_read_cost();
    let cold_account_additional_cost = params.cold_account_additional_cost();
    let (tx, journal) = ctx.tx_journal_mut();
    let input = tx.input().clone();

    match tx.kind() {
        TxKind::Call(target_address) => {
            // Load the recipient once (its access was already charged at the
            // cold rate at the intrinsic phase).
            let account = &journal.load_account_with_code(target_address)?.info;
            let recipient_is_empty = account.is_empty();
            let delegated_address = account.code.as_ref().and_then(Bytecode::eip7702_address);
            let mut known_bytecode = (
                account.code_hash(),
                account.code.clone().unwrap_or_default(),
            );

            let mut charged_new_account_state_gas = false;
            if is_eip2780 && !tx.value().is_zero() && recipient_is_empty {
                if !gas.record_state_cost(new_account_state_gas) {
                    return Ok(None);
                }
                charged_new_account_state_gas = true;
            }

            if let Some(delegated_address) = delegated_address {
                if is_eip2780 {
                    // Charge the warm access upfront and the cold difference
                    // after the load. Charges made before an out-of-gas bail
                    // need no undo: the runtime out-of-gas path rebuilds the
                    // transaction-level gas.
                    if !gas.record_regular_cost(warm_access_cost) {
                        return Ok(None);
                    }
                    let skip_cold_load = gas.remaining() < cold_account_additional_cost;
                    let acc = match journal.load_account_info_skip_cold_load(
                        delegated_address,
                        true,
                        skip_cold_load,
                    ) {
                        Ok(acc) => acc,
                        Err(JournalLoadError::ColdLoadSkipped) => return Ok(None),
                        Err(JournalLoadError::DBError(e)) => return Err(e),
                    };

                    if acc.is_cold && !gas.record_regular_cost(cold_account_additional_cost) {
                        return Ok(None);
                    }
                    known_bytecode = (acc.code_hash(), acc.code.clone().unwrap_or_default());
                } else {
                    let account = &journal.load_account_with_code(delegated_address)?.info;
                    known_bytecode = (
                        account.code_hash(),
                        account.code.clone().unwrap_or_default(),
                    );
                }
            }

            Ok(Some(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::Bytes(input),
                gas_limit: gas.remaining(),
                target_address,
                bytecode_address: target_address,
                known_bytecode,
                caller: tx.caller(),
                value: CallValue::Transfer(tx.value()),
                scheme: CallScheme::Call,
                is_static: false,
                return_memory_offset: 0..0,
                reservoir: gas.reservoir(),
                charged_new_account_state_gas,
            }))))
        }
        TxKind::Create => {
            let mut charged_create_state_gas = false;
            if is_eip2780 {
                // The tx nonce was validated against the caller's nonce, which
                // a create transaction bumps only at frame creation — after
                // this point.
                let created_address = tx.caller().create(tx.nonce());
                let target_is_empty = journal.load_account(created_address)?.info.is_empty();
                if target_is_empty {
                    if !gas.record_state_cost(create_state_gas) {
                        return Ok(None);
                    }
                    charged_create_state_gas = true;
                }
            }
            let mut inputs = CreateInputs::new(
                tx.caller(),
                CreateScheme::Create,
                tx.value(),
                input,
                gas.remaining(),
                gas.reservoir(),
            );
            inputs.set_charged_create_state_gas(charged_create_state_gas);
            Ok(Some(FrameInput::Create(Box::new(inputs))))
        }
    }
}

/// Unwinds the EIP-2780 runtime gas phase after first-frame creation ran out
/// of gas ([`create_init_frame`] returned `None`): reverts the phase's
/// journal checkpoint — dropping all runtime state changes, including applied
/// EIP-7702 delegations — and bumps the sender nonce of a create transaction,
/// whose increment precedes message processing and survives the halt. (Calls
/// bump it in `validate_against_state_and_deduct_caller`; creates at frame
/// creation, which the runtime halt never reaches.)
#[inline]
pub fn runtime_oog_unwind<CTX: ContextTr>(
    ctx: &mut CTX,
    checkpoint: JournalCheckpoint,
) -> Result<(), <<CTX::Journal as JournalTr>::Database as Database>::Error> {
    let (tx, journal) = ctx.tx_journal_mut();
    journal.checkpoint_revert(checkpoint);
    if tx.kind().is_create() {
        journal.load_account_mut(tx.caller())?.data.bump_nonce();
    }
    Ok(())
}
