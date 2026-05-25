use crate::{
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes as ITy, MemoryTr, ReturnData, RuntimeFlag, StackTr},
    InstructionContext as Ictx, InstructionResult,
};
use context_interface::{cfg::GasParams, host::LoadError, Host};
use core::{cmp::min, ops::Range};
use primitives::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, *},
    Address, B256, U256,
};
use state::{Bytecode, BytecodeLoad};

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
///
/// The trailing bool in the returned tuple is `charged_new_account_state_gas`:
/// `true` iff this call upfront-charged EIP-8037 `new_account_state_gas`
/// (transfers value into a previously-empty account). Callers should propagate
/// it onto `CallInputs` so the parent can refund the charge if the resulting
/// frame reverts/halts.
#[inline(never)]
pub fn load_acc_and_calc_gas<H: Host + ?Sized>(
    context: &mut Ictx<'_, H, impl ITy>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
    stack_gas_limit: u64,
) -> Result<(u64, BytecodeLoad, B256, bool), InstructionResult> {
    // Transfer value cost
    if transfers_value {
        gas!(
            context.interpreter,
            context.host.gas_params().transfer_value_cost()
        );
    }

    // load account delegated and deduct dynamic gas.
    let (gas, state_gas_cost, bytecode, code_hash) =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let charged_new_account_state_gas = state_gas_cost > 0;
    let interpreter = &mut context.interpreter;

    // deduct dynamic gas.
    gas!(interpreter, gas);

    // deduct state gas (EIP-8037) if any.
    state_gas!(interpreter, state_gas_cost);

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

    Ok((
        gas_limit,
        bytecode,
        code_hash,
        charged_new_account_state_gas,
    ))
}

/// Loads accounts and its delegate account.
///
/// Returns `(regular_gas_cost, state_gas_cost, bytecode_load, code_hash)`.
///
/// `bytecode_load` is `BytecodeLoad::Bytecode(_)` when the bytecode for the
/// frame is already known (non-delegated path), and
/// `BytecodeLoad::LoadFrom(delegate_address)` when the call target is an
/// EIP-7702 delegation — in that case the delegate account is not loaded here,
/// only its warm/cold status is consulted to charge gas, and `code_hash` is
/// returned as [`B256::ZERO`] (the real hash is fetched when the deferred
/// load is resolved at frame creation).
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut Ictx<'_, H, impl ITy>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, u64, BytecodeLoad, B256), InstructionResult> {
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
/// Returns `(regular_gas_cost, state_gas_cost, bytecode_load, code_hash)`.
/// `state_gas_cost` is non-zero only when creating a new empty account (EIP-8037).
///
/// For EIP-7702 delegations the delegate account is **not** loaded here —
/// only its warm/cold status is checked via [`Host::is_account_cold`] in
/// order to charge the correct gas. The actual bytecode load is deferred to
/// frame creation, signalled by returning `BytecodeLoad::LoadFrom(address)`.
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, u64, BytecodeLoad, B256), LoadError> {
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
    let bytecode = account.code.clone().unwrap_or_default();
    let code_hash = account.code_hash();
    // New account cost, as account is empty there is no delegated account and we can return early.
    if create_empty_account && account.is_empty {
        cost += host
            .gas_params()
            .new_account_cost(is_spurious_dragon, transfers_value);
        if host.is_amsterdam_eip8037_enabled() && transfers_value {
            state_gas_cost += host.gas_params().new_account_state_gas(host.cpsb());
        }
        return Ok((
            cost,
            state_gas_cost,
            BytecodeLoad::Bytecode(bytecode),
            code_hash,
        ));
    }

    // EIP-7702 delegation: the delegate is warmed eagerly so the cold/warm gas
    // charge can be computed correctly, but its bytecode is only loaded here
    // when the account is already present in the journal state map. Otherwise
    // the bytecode load is deferred to frame creation via
    // `BytecodeLoad::LoadFrom`.
    if let Some(delegate_address) = account.code.as_ref().and_then(Bytecode::eip7702_address) {
        // EIP-7702 is enabled after berlin hardfork.
        cost += warm_storage_read_cost;
        if cost > remaining_gas {
            return Err(LoadError::ColdLoadSkipped);
        }

        // Skip the cold load if the caller cannot afford the cold surcharge.
        let skip_cold_load = remaining_gas < cost + additional_cold_cost;
        let load = host.optional_account_warming(delegate_address, skip_cold_load)?;
        if load.is_cold {
            cost += additional_cold_cost;
            if cost > remaining_gas {
                return Err(LoadError::ColdLoadSkipped);
            }
        }
        let bytecode_load = match load.data {
            Some(code) => BytecodeLoad::Bytecode(code),
            None => BytecodeLoad::LoadFrom(delegate_address),
        };
        return Ok((cost, state_gas_cost, bytecode_load, B256::ZERO));
    }

    Ok((
        cost,
        state_gas_cost,
        BytecodeLoad::Bytecode(bytecode),
        code_hash,
    ))
}

/// EIP-150 call-stack depth check for the CALL family.
///
/// Returns `true` and short-circuits the opcode when creating a child frame
/// would exceed [`CALL_STACK_LIMIT`]. The full `gas_limit` (already including
/// the call stipend for value-transferring calls) is reimbursed to the
/// parent's tracker — matching the EVM spec where the stipend on a failed
/// immediate call is effectively returned to the caller — `U256::ZERO` is
/// pushed and the caller should return `Ok(())` to continue execution.
///
/// Returns `false` when the depth check passes; the caller should proceed to
/// dispatch the child frame.
#[inline]
pub fn check_call_depth<IT: ITy, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IT>,
    host: &H,
    gas_limit: u64,
) -> bool {
    if host.depth() <= CALL_STACK_LIMIT as usize {
        return false;
    }
    interpreter.gas.erase_cost(gas_limit);
    // EIP-211: a failed call exposes an empty return data buffer to the
    // caller's subsequent RETURNDATASIZE/RETURNDATACOPY.
    interpreter.return_data.clear();
    // Safe to push without stack-overflow check: each CALL family opcode pops
    // at least two stack items before reaching this point.
    let _ = interpreter.stack.push(U256::ZERO);
    true
}

/// EIP-150 call-stack depth check for CREATE/CREATE2.
///
/// Returns `true` when the depth check fails: the allocated child gas is
/// reimbursed to the parent, the EIP-8037 `create_state_gas` (if any) is
/// returned to the reservoir, `U256::ZERO` is pushed, and the caller should
/// return `Ok(())` to continue execution.
#[inline]
pub fn check_create_depth<IT: ITy, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IT>,
    host: &H,
    gas_limit: u64,
) -> bool {
    if host.depth() <= CALL_STACK_LIMIT as usize {
        return false;
    }
    interpreter.gas.erase_cost(gas_limit);
    if host.is_amsterdam_eip8037_enabled() {
        let state_gas = host.gas_params().create_state_gas(host.cpsb());
        interpreter.gas.refill_reservoir(state_gas);
    }
    // A non-reverted CREATE failure clears the caller's return data buffer
    // (the spec only preserves it for the CREATE revert path).
    interpreter.return_data.clear();
    let _ = interpreter.stack.push(U256::ZERO);
    true
}

/// Caller-balance check for value-transferring CALL/CALLCODE.
///
/// Returns `true` when the caller cannot afford the value transfer: the
/// allocated child gas (already including the call stipend) is reimbursed to
/// the parent, `U256::ZERO` is pushed, and the caller should return `Ok(())`
/// to continue execution — the same short-circuit shape used for the depth
/// check.
///
/// Returns `false` when the caller has enough balance; the caller should
/// proceed to dispatch the child frame.
#[inline]
pub fn check_caller_balance<IT: ITy, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IT>,
    host: &mut H,
    caller: Address,
    value: U256,
    gas_limit: u64,
) -> Result<bool, InstructionResult> {
    let balance = host
        .balance(caller)
        .ok_or(InstructionResult::FatalExternalError)?;
    if balance.data >= value {
        return Ok(false);
    }
    interpreter.gas.erase_cost(gas_limit);
    // EIP-211: a failed call exposes an empty return data buffer to the
    // caller's subsequent RETURNDATASIZE/RETURNDATACOPY.
    interpreter.return_data.clear();
    // Safe to push without stack-overflow check: each CALL family opcode pops
    // at least two stack items before reaching this point.
    let _ = interpreter.stack.push(U256::ZERO);
    Ok(true)
}

/// Resolve a [`BytecodeLoad`] produced by [`load_acc_and_calc_gas`] into the
/// final `(code_hash, bytecode)` pair used to build [`crate::CallInputs`].
///
/// `BytecodeLoad::Bytecode` is returned as-is (the main account's code was
/// already loaded). `BytecodeLoad::LoadFrom(delegate)` triggers a fetch of
/// the delegate account's code via [`Host::load_account_info_skip_cold_load`]
/// (with `skip_cold_load = false`) — by the time this helper is invoked the
/// CALL opcode has charged the access cost in [`load_account_delegated`] and
/// passed the depth/balance gates, so this is the right moment to actually
/// warm and read the delegate.
#[inline]
pub fn resolve_bytecode_load<H: Host + ?Sized>(
    host: &mut H,
    bytecode_hash: B256,
    bytecode_load: BytecodeLoad,
) -> Result<(B256, Bytecode), InstructionResult> {
    match bytecode_load {
        BytecodeLoad::Bytecode(code) => Ok((bytecode_hash, code)),
        BytecodeLoad::LoadFrom(addr) => {
            let info = host.load_account_info_skip_cold_load(addr, true, false)?;
            Ok((info.code_hash(), info.code.clone().unwrap_or_default()))
        }
    }
}
