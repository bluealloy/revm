use crate::{
    gas::{
        self, calc_call_static_gas, COLD_ACCOUNT_ACCESS_COST_ADDITIONAL, NEWACCOUNT,
        WARM_STORAGE_READ_COST,
    },
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
    InstructionContext,
};
use context_interface::{host::LoadError, Host};
use core::{cmp::min, ops::Range};
use primitives::{
    hardfork::SpecId::{self, *},
    Address, U256,
};
use state::Bytecode;

/// Gets memory input and output ranges for call instructions.
#[inline]
pub fn get_memory_input_and_out_ranges(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
) -> Option<(Range<usize>, Range<usize>)> {
    popn!([in_offset, in_len, out_offset, out_len], interpreter, None);

    let mut in_range = resize_memory(interpreter, in_offset, in_len)?;

    if !in_range.is_empty() {
        let offset = interpreter.memory.local_memory_offset();
        in_range = in_range.start.saturating_add(offset)..in_range.end.saturating_add(offset);
    }

    let ret_range = resize_memory(interpreter, out_offset, out_len)?;
    Some((in_range, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    offset: U256,
    len: U256,
) -> Option<Range<usize>> {
    let len = as_usize_or_fail_ret!(interpreter, len, None);
    let offset = if len != 0 {
        let offset = as_usize_or_fail_ret!(interpreter, offset, None);
        resize_memory!(interpreter, offset, len, None);
        offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };
    Some(offset..offset + len)
}

/// Calculates gas cost and limit for call instructions.
#[inline]
pub fn load_acc_and_calc_gas<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
    stack_gas_limit: u64,
) -> Option<u64> {
    let spec = context.interpreter.runtime_flag.spec_id();
    // calculate static gas first. For berlin hardfork it will take warm gas.
    let static_gas = calc_call_static_gas(spec, transfers_value);
    gas!(context.interpreter, static_gas, None);

    // load account delegated and deduct dynamic gas.
    let gas =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let interpreter = &mut context.interpreter;

    // deduct dynamic gas.
    gas!(interpreter, gas, None);

    // EIP-150: Gas cost changes for IO-heavy operations
    let mut gas_limit = if interpreter.runtime_flag.spec_id().is_enabled_in(TANGERINE) {
        // Take l64 part of gas_limit
        min(interpreter.gas.remaining_63_of_64_parts(), stack_gas_limit)
    } else {
        stack_gas_limit
    };

    gas!(interpreter, gas_limit, None);

    // Add call stipend if there is value to be transferred.
    if transfers_value {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    Some(gas_limit)
}

/// Loads accounts and its delegate account.
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Option<u64> {
    let remaining_gas = context.interpreter.gas.remaining();
    match load_account_delegated(
        context.host,
        context.interpreter.runtime_flag.spec_id(),
        remaining_gas,
        to,
        transfers_value,
        create_empty_account,
    ) {
        Ok(remaining_gas) => return Some(remaining_gas),
        Err(LoadError::ColdLoadSkipped) => {
            context.interpreter.halt_oog();
        }
        Err(LoadError::DBError) => context.interpreter.halt_fatal(),
    }
    None
}

/// Loads accounts and its delegate account.
///
/// Assumption is that warm gas is already deducted.
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<u64, LoadError> {
    let mut cost = 0;
    let is_berlin = spec.is_enabled_in(SpecId::BERLIN);
    let is_spurioud_dragon = spec.is_enabled_in(SpecId::SPURIOUS_DRAGON);

    let skip_cold_load = is_berlin && remaining_gas < COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
    let account = host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;

    if is_berlin && account.is_cold {
        cost += COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
    }

    // New account cost, as account is empty there is no delegated account and we can return early.
    if create_empty_account && account.is_empty {
        cost += new_account_cost(is_spurioud_dragon, transfers_value);
        return Ok(cost);
    }

    // load delegate code if account is EIP-7702
    if let Some(Bytecode::Eip7702(code)) = &account.code {
        // EIP-7702 is enabled after berlin hardfork.
        cost += WARM_STORAGE_READ_COST;
        if cost > remaining_gas {
            return Err(LoadError::ColdLoadSkipped);
        }
        let address = code.address();

        // skip cold load if there is enough gas to cover the cost.
        let skip_cold_load = remaining_gas < cost + COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
        let delegate_account =
            host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;

        if delegate_account.is_cold {
            cost += COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
        }
        if create_empty_account && delegate_account.is_empty {
            cost += new_account_cost(is_spurioud_dragon, transfers_value);
        }
    }

    Ok(cost)
}

/// Returns new account cost.
#[inline]
pub fn new_account_cost(is_spurioud_dragon: bool, transfers_value: bool) -> u64 {
    // EIP-161: State trie clearing (invariant-preserving alternative)
    // Account only if there is value transferred.
    if !is_spurioud_dragon || transfers_value {
        // before spurious dragon [`NEWACCOUNT`] will be always accounted
        // After EIP-161 it is only accounted if there is value transferred.
        return NEWACCOUNT;
    }
    0
}
