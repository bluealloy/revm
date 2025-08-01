use crate::{
    gas,
    interpreter_types::{MemoryTr, RuntimeFlag, StackTr},
    InstructionContextTr,
};
use context_interface::{context::StateLoad, journaled_state::AccountLoad};
use core::{cmp::min, ops::Range};
use primitives::{hardfork::SpecId::*, U256};

/// Gets memory input and output ranges for call instructions.
#[inline]
pub fn get_memory_input_and_out_ranges<C: InstructionContextTr>(
    context: &mut C,
) -> Option<(Range<usize>, Range<usize>)> {
    popn!([in_offset, in_len, out_offset, out_len], context, None);

    let mut in_range = resize_memory(context, in_offset, in_len)?;

    if !in_range.is_empty() {
        let offset = context.memory().local_memory_offset();
        in_range = in_range.start.saturating_add(offset)..in_range.end.saturating_add(offset);
    }

    let ret_range = resize_memory(context, out_offset, out_len)?;
    Some((in_range, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory<C: InstructionContextTr>(
    context: &mut C,
    offset: U256,
    len: U256,
) -> Option<Range<usize>> {
    let len = as_usize_or_fail_ret!(context, len, None);
    let offset = if len != 0 {
        let offset = as_usize_or_fail_ret!(context, offset, None);
        resize_memory!(context, offset, len, None);
        offset
    } else {
        usize::MAX // Unrealistic value so we are sure it is not used.
    };
    Some(offset..offset + len)
}

/// Calculates gas cost and limit for call instructions.
#[inline]
pub fn calc_call_gas<C: InstructionContextTr>(
    context: &mut C,
    account_load: StateLoad<AccountLoad>,
    has_transfer: bool,
    local_gas_limit: u64,
) -> Option<u64> {
    let call_cost = gas::call_cost(context.runtime_flag().spec_id(), has_transfer, account_load);
    gas!(context, call_cost, None);

    // EIP-150: Gas cost changes for IO-heavy operations
    let gas_limit = if context.runtime_flag().spec_id().is_enabled_in(TANGERINE) {
        // Take l64 part of gas_limit
        min(context.gas().remaining_63_of_64_parts(), local_gas_limit)
    } else {
        local_gas_limit
    };

    Some(gas_limit)
}
