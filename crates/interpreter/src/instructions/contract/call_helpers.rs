use crate::{
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes as ITy, MemoryTr, RuntimeFlag, StackTr},
    InstructionContext as Ictx, InstructionResult,
};
use context_interface::{cfg::GasParams, host::LoadError, Host};
use core::{cmp::min, ops::Range};
use primitives::{
    hardfork::SpecId::{self, *},
    Address, B256, U256,
};
use state::Bytecode;

/// Gets memory input and output ranges for call instructions.
#[inline]
pub fn get_memory_input_and_out_ranges(
    interpreter: &mut Interpreter<impl ITy>,
    gas_params: &GasParams,
) -> Result<(Range<usize>, Range<usize>), InstructionResult> {
    popn!([in_offset, in_len, out_offset, out_len], interpreter);

    let mut in_range = resize_memory(interpreter, gas_params, in_offset, in_len)?;

    if !in_range.is_empty() {
        let offset = interpreter.memory.local_memory_offset();
        in_range = in_range.start.saturating_add(offset)..in_range.end.saturating_add(offset);
    }

    let ret_range = resize_memory(interpreter, gas_params, out_offset, out_len)?;
    Ok((in_range, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory(
    interpreter: &mut Interpreter<impl ITy>,
    gas_params: &GasParams,
    offset: U256,
    len: U256,
) -> Result<Range<usize>, InstructionResult> {
    let len = as_usize_or_fail!(interpreter, len);
    let offset = if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        interpreter.resize_memory(gas_params, offset, len)?;
        offset
    } else {
        usize::MAX // unrealistic value so we are sure it is not used
    };
    Ok(offset..offset + len)
}

/// Calculates gas cost and limit for call instructions.
#[inline(never)]
pub fn load_acc_and_calc_gas<H: Host + ?Sized>(
    context: &mut Ictx<'_, H, impl ITy>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
    stack_gas_limit: u64,
) -> Result<(u64, Bytecode, B256), InstructionResult> {
    // Transfer value cost
    if transfers_value {
        gas!(
            context.interpreter,
            context.host.gas_params().transfer_value_cost()
        );
    }

    // load account delegated and deduct dynamic gas.
    let (gas, creates_new_account, bytecode, code_hash) =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let interpreter = &mut context.interpreter;

    // deduct dynamic gas.
    gas!(interpreter, gas);

    // EIP-8037 new-account counter for CALL-with-value-to-empty.
    if creates_new_account {
        interpreter.new_state.add_call_account();
    }

    let interpreter = &mut context.interpreter;
    let host = &mut context.host;

    // EIP-150: Gas cost changes for IO-heavy operations
    let mut gas_limit = if interpreter.runtime_flag.spec_id().is_enabled_in(TANGERINE) {
        // On mainnet this will take return 63/64 of gas_limit.
        let reduced_gas_limit = host
            .gas_params()
            .call_stipend_reduction(interpreter.gas.remaining());
        min(reduced_gas_limit, stack_gas_limit)
    } else {
        stack_gas_limit
    };
    gas!(interpreter, gas_limit);

    // Add call stipend if there is value to be transferred.
    if transfers_value {
        gas_limit = gas_limit.saturating_add(host.gas_params().call_stipend());
    }

    Ok((gas_limit, bytecode, code_hash))
}

/// Loads accounts and its delegate account.
///
/// Returns `(regular_gas_cost, creates_new_account, bytecode, code_hash)`.
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut Ictx<'_, H, impl ITy>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, bool, Bytecode, B256), InstructionResult> {
    // move this to static gas.
    let remaining_gas = context.interpreter.gas.remaining();
    Ok(load_account_delegated(
        context.host,
        context.interpreter.runtime_flag.spec_id(),
        remaining_gas,
        to,
        transfers_value,
        create_empty_account,
    )?)
}

/// Loads accounts and its delegate account.
///
/// Assumption is that warm gas is already deducted.
///
/// Returns `(regular_gas_cost, creates_new_account, bytecode, code_hash)`.
/// `creates_new_account` is `true` only when CALL with value materializes a
/// new empty account (EIP-8037 new-account state-counter trigger).
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, bool, Bytecode, B256), LoadError> {
    let mut cost = 0;
    let mut creates_new_account = false;
    let is_berlin = spec.is_enabled_in(SpecId::BERLIN);
    let is_spurious_dragon = spec.is_enabled_in(SpecId::SPURIOUS_DRAGON);

    let additional_cold_cost = host.gas_params().cold_account_additional_cost();
    let warm_storage_read_cost = host.gas_params().warm_storage_read_cost();

    let skip_cold_load = is_berlin && remaining_gas < additional_cold_cost;
    let account = host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;
    if is_berlin && account.is_cold {
        cost += additional_cold_cost;
    }
    let mut bytecode = account.code.clone().unwrap_or_default();
    let mut code_hash = account.code_hash();
    // New account cost, as account is empty there is no delegated account and we can return early.
    if create_empty_account && account.is_empty {
        cost += host
            .gas_params()
            .new_account_cost(is_spurious_dragon, transfers_value);
        if host.is_amsterdam_eip8037_enabled() && transfers_value {
            creates_new_account = true;
        }
        return Ok((cost, creates_new_account, bytecode, code_hash));
    }

    // load delegate code if account is EIP-7702
    if let Some(address) = account.code.as_ref().and_then(Bytecode::eip7702_address) {
        // EIP-7702 is enabled after berlin hardfork.
        cost += warm_storage_read_cost;
        if cost > remaining_gas {
            return Err(LoadError::ColdLoadSkipped);
        }

        // skip cold load if there is enough gas to cover the cost.
        let skip_cold_load = remaining_gas < cost + additional_cold_cost;
        let delegate_account =
            host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;

        if delegate_account.is_cold {
            cost += additional_cold_cost;
        }
        bytecode = delegate_account.code.clone().unwrap_or_default();
        code_hash = delegate_account.code_hash();
    }

    Ok((cost, creates_new_account, bytecode, code_hash))
}
