use crate::{
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
    InstructionContext,
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
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    gas_params: &GasParams,
) -> Option<(Range<usize>, Range<usize>)> {
    popn!([in_offset, in_len, out_offset, out_len], interpreter, None);

    let mut in_range = resize_memory(interpreter, gas_params, in_offset, in_len)?;

    if !in_range.is_empty() {
        let offset = interpreter.memory.local_memory_offset();
        in_range = in_range.start.saturating_add(offset)..in_range.end.saturating_add(offset);
    }

    let ret_range = resize_memory(interpreter, gas_params, out_offset, out_len)?;
    Some((in_range, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    gas_params: &GasParams,
    offset: U256,
    len: U256,
) -> Option<Range<usize>> {
    let len = as_usize_or_fail_ret!(interpreter, len, None);
    let offset = if len != 0 {
        let offset = as_usize_or_fail_ret!(interpreter, offset, None);
        resize_memory!(interpreter, gas_params, offset, len, None);
        offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };
    Some(offset..offset + len)
}

/// Calculates gas cost and limit for call instructions.
#[inline(never)]
pub fn load_acc_and_calc_gas<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
    stack_gas_limit: u64,
) -> Option<(u64, Bytecode, B256)> {
    // Transfer value cost
    if transfers_value {
        gas!(
            context.interpreter,
            context.host.gas_params().transfer_value_cost(),
            None
        );
    }

    // load account delegated and deduct dynamic gas.
    let (gas, state_gas_cost, bytecode, code_hash) =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let interpreter = &mut context.interpreter;

    // deduct dynamic gas.
    gas!(interpreter, gas, None);

    // deduct state gas (TIP-1016) if any.
    state_gas!(interpreter, state_gas_cost, None);

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
    // Deduct gas forwarded to child from remaining only (not regular gas).
    // Child inherits parent's regular_gas_remaining directly.
    if !interpreter.gas.record_remaining_cost(gas_limit) {
        interpreter.halt_oog();
        return None;
    }

    // Add call stipend if there is value to be transferred.
    if transfers_value {
        gas_limit = gas_limit.saturating_add(host.gas_params().call_stipend());
    }

    Some((gas_limit, bytecode, code_hash))
}

/// Loads accounts and its delegate account.
///
/// Returns `(regular_gas_cost, state_gas_cost, bytecode, code_hash)`.
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Option<(u64, u64, Bytecode, B256)> {
    // move this to static gas.
    let remaining_gas = context.interpreter.gas.remaining();
    match load_account_delegated(
        context.host,
        context.interpreter.runtime_flag.spec_id(),
        remaining_gas,
        to,
        transfers_value,
        create_empty_account,
    ) {
        Ok(out) => return Some(out),
        Err(LoadError::ColdLoadSkipped) => {
            context.interpreter.halt_oog();
        }
        Err(LoadError::DBError) => {
            context.interpreter.halt_fatal();
        }
    }
    None
}

/// Loads accounts and its delegate account.
///
/// Assumption is that warm gas is already deducted.
///
/// Returns `(regular_gas_cost, state_gas_cost, bytecode, code_hash)`.
/// `state_gas_cost` is non-zero only when creating a new empty account (TIP-1016).
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, u64, Bytecode, B256), LoadError> {
    let mut cost = 0;
    let mut state_gas_cost = 0;
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
        if host.is_state_gas_enabled() {
            state_gas_cost += host.gas_params().new_account_state_gas();
        }
        return Ok((cost, state_gas_cost, bytecode, code_hash));
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

    Ok((cost, state_gas_cost, bytecode, code_hash))
}
