use crate::{
    gas::{self},
    interpreter::Interpreter,
    primitives::{Address, Bytes, Spec, SpecId::*},
    Host, InstructionResult,
};
use core::{cmp::min, ops::Range};

#[inline]
pub fn get_memory_input_and_out_ranges(
    interpreter: &mut Interpreter,
) -> Option<(Bytes, Range<usize>)> {
    pop_ret!(interpreter, in_offset, in_len, out_offset, out_len, None);

    let in_len = as_usize_or_fail_ret!(interpreter, in_len, None);
    let input = if in_len != 0 {
        let in_offset = as_usize_or_fail_ret!(interpreter, in_offset, None);
        shared_memory_resize!(interpreter, in_offset, in_len, None);
        Bytes::copy_from_slice(interpreter.shared_memory.slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    let out_len = as_usize_or_fail_ret!(interpreter, out_len, None);
    let out_offset = if out_len != 0 {
        let out_offset = as_usize_or_fail_ret!(interpreter, out_offset, None);
        shared_memory_resize!(interpreter, out_offset, out_len, None);
        out_offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };

    Some((input, out_offset..out_offset + out_len))
}

#[inline]
pub fn calc_call_gas<H: Host, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut H,
    to: Address,
    has_transfer: bool,
    local_gas_limit: u64,
    is_call_or_callcode: bool,
    is_call_or_staticcall: bool,
) -> Option<u64> {
    let Some((is_cold, exist)) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return None;
    };
    let is_new = !exist;

    let call_cost = gas::call_cost::<SPEC>(
        has_transfer,
        is_new,
        is_cold,
        is_call_or_callcode,
        is_call_or_staticcall,
    );

    gas!(interpreter, call_cost, None);

    // EIP-150: Gas cost changes for IO-heavy operations
    let gas_limit = if SPEC::enabled(TANGERINE) {
        let gas = interpreter.gas().remaining();
        // take l64 part of gas_limit
        min(gas - gas / 64, local_gas_limit)
    } else {
        local_gas_limit
    };

    Some(gas_limit)
}
