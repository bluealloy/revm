use crate::{
    gas::params::GasParams,
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
    InstructionContext,
};
use context_interface::{host::LoadError, Host};
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
            context.interpreter.gas_table.transfer_value_cost(),
            None
        );
    }

    println!(
        "GAS BEFORE LOAD OF ACCOUNT: {}",
        context.interpreter.gas.remaining()
    );
    // load account delegated and deduct dynamic gas.
    let (gas, bytecode, code_hash) =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let interpreter = &mut context.interpreter;

    println!("GAS calc LOAD OF ACCOUNT: {}", gas);

    // deduct dynamic gas.
    gas!(interpreter, gas, None);
    println!(
        "GAS AFTer LOAD OF ACCOUNT: {}",
        context.interpreter.gas.remaining()
    );
    let interpreter = &mut context.interpreter;

    // EIP-150: Gas cost changes for IO-heavy operations
    let mut gas_limit = if interpreter.runtime_flag.spec_id().is_enabled_in(TANGERINE) {
        // On mainnet this will take return 63/64 of gas_limit.
        let reduced_gas_limit = interpreter
            .gas_table
            .call_stipend_reduction(interpreter.gas.remaining());
        println!("REDUCED GAS LIMIT: {}", reduced_gas_limit);
        println!("STACK GAS LIMIT: {}", stack_gas_limit);
        min(reduced_gas_limit, stack_gas_limit)
    } else {
        println!("STACK GAS LIMIT: {}", stack_gas_limit);
        stack_gas_limit
    };

    println!("CALL stipend before: {}", interpreter.gas.remaining());

    gas!(interpreter, gas_limit, None);

    println!("CALL stipend after: {}", interpreter.gas.remaining());
    // Add call stipend if there is value to be transferred.
    if transfers_value {
        gas_limit = gas_limit.saturating_add(interpreter.gas_table.call_stipend());
    }

    Some((gas_limit, bytecode, code_hash))
}

/// Loads accounts and its delegate account.
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Option<(u64, Bytecode, B256)> {
    // move this to static gas.
    let remaining_gas = context.interpreter.gas.remaining();
    let gas_table = &context.interpreter.gas_table;
    match load_account_delegated(
        context.host,
        gas_table,
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
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    gas_table: &GasParams,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, Bytecode, B256), LoadError> {
    let mut cost = 0;
    let is_berlin = spec.is_enabled_in(SpecId::BERLIN);
    let is_spurious_dragon = spec.is_enabled_in(SpecId::SPURIOUS_DRAGON);

    let additional_cold_cost = gas_table.cold_account_additional_cost();

    let skip_cold_load = is_berlin && remaining_gas < additional_cold_cost;
    let account = host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;
    if is_berlin && account.is_cold {
        cost += additional_cold_cost;
    }
    let mut bytecode = account.code.clone().unwrap_or_default();
    let mut code_hash = account.code_hash();
    // New account cost, as account is empty there is no delegated account and we can return early.
    if create_empty_account && account.is_empty {
        cost += gas_table.new_account_cost(is_spurious_dragon, transfers_value);
        return Ok((cost, bytecode, code_hash));
    }

    // load delegate code if account is EIP-7702
    if let Some(Bytecode::Eip7702(code)) = &account.code {
        // EIP-7702 is enabled after berlin hardfork.
        cost += gas_table.warm_storage_read_cost();
        if cost > remaining_gas {
            return Err(LoadError::ColdLoadSkipped);
        }
        let address = code.address();

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

    Ok((cost, bytecode, code_hash))
}
