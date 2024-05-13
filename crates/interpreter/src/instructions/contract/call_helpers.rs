use crate::{
    gas,
    interpreter::Interpreter,
    primitives::{Bytes, Spec, SpecId::*, U256},
};
use core::{cmp::min, ops::Range};

#[inline]
pub fn get_memory_input_and_out_ranges(
    interpreter: &mut Interpreter,
) -> Option<(Bytes, Range<usize>)> {
    pop_ret!(interpreter, in_offset, in_len, out_offset, out_len, None);

    let in_range = resize_memory_and_return_range(interpreter, in_offset, in_len)?;

    let mut input = Bytes::new();
    if !in_range.is_empty() {
        input = Bytes::copy_from_slice(interpreter.shared_memory.slice_range(in_range));
    }

    let ret_range = resize_memory_and_return_range(interpreter, out_offset, out_len)?;
    Some((input, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory_and_return_range(
    interpreter: &mut Interpreter,
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

#[inline]
pub fn calc_call_gas<SPEC: Spec>(
    interpreter: &mut Interpreter,
    is_cold: bool,
    has_transfer: bool,
    new_account_accounting: bool,
    local_gas_limit: u64,
) -> Option<u64> {
    let call_cost = gas::call_cost(SPEC::SPEC_ID, has_transfer, is_cold, new_account_accounting);

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
