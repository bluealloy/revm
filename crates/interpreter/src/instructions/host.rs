use crate::InstructionContextTr;
use crate::{
    gas::{self, warm_cold_cost, CALL_STIPEND},
    instructions::{
        utility::{IntoAddress, IntoU256},
        InstructionReturn,
    },
    interpreter_types::{InputsTr, MemoryTr, RuntimeFlag, StackTr},
    Host, InstructionResult,
};
use core::cmp::min;
use primitives::{hardfork::SpecId::*, Bytes, Log, LogData, B256, BLOCK_HASH_HISTORY, U256};

/// Implements the BALANCE instruction.
///
/// Gets the balance of the given account.
#[inline]
pub fn balance<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn_top!([], top, context);
    let address = top.into_address();
    let Some(balance) = context.host().balance(address) else {
        return context.halt(InstructionResult::FatalExternalError);
    };
    let spec_id = context.runtime_flag().spec_id();
    gas!(
        context,
        if spec_id.is_enabled_in(BERLIN) {
            warm_cold_cost(balance.is_cold)
        } else if spec_id.is_enabled_in(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            700
        } else if spec_id.is_enabled_in(TANGERINE) {
            400
        } else {
            20
        }
    );
    *top = balance.data;
    InstructionReturn::cont()
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
#[inline]
pub fn selfbalance<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, ISTANBUL);
    gas!(context, gas::LOW);

    let address = context.input().target_address();
    let Some(balance) = context.host().balance(address) else {
        return context.halt(InstructionResult::FatalExternalError);
    };
    push!(context, balance.data);
    InstructionReturn::cont()
}

/// Implements the EXTCODESIZE instruction.
///
/// Gets the size of an account's code.
#[inline]
pub fn extcodesize<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn_top!([], top, context);
    let address = top.into_address();
    let Some(code) = context.host().load_account_code(address) else {
        return context.halt(InstructionResult::FatalExternalError);
    };
    let spec_id = context.runtime_flag().spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(context, warm_cold_cost(code.is_cold));
    } else if spec_id.is_enabled_in(TANGERINE) {
        gas!(context, 700);
    } else {
        gas!(context, 20);
    }

    *top = U256::from(code.len());
    InstructionReturn::cont()
}

/// EIP-1052: EXTCODEHASH opcode
#[inline]
pub fn extcodehash<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CONSTANTINOPLE);
    popn_top!([], top, context);
    let address = top.into_address();
    let Some(code_hash) = context.host().load_account_code_hash(address) else {
        return context.halt(InstructionResult::FatalExternalError);
    };
    let spec_id = context.runtime_flag().spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(context, warm_cold_cost(code_hash.is_cold));
    } else if spec_id.is_enabled_in(ISTANBUL) {
        gas!(context, 700);
    } else {
        gas!(context, 400);
    }
    *top = code_hash.into_u256();
    InstructionReturn::cont()
}

/// Implements the EXTCODECOPY instruction.
///
/// Copies a portion of an account's code to memory.
#[inline]
pub fn extcodecopy<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn!([address, memory_offset, code_offset, len_u256], context);
    let address = address.into_address();
    let Some(code) = context.host().load_account_code(address) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    let len = as_usize_or_fail!(context, len_u256);
    gas_or_fail!(
        context,
        gas::extcodecopy_cost(context.runtime_flag().spec_id(), len, code.is_cold)
    );
    if len == 0 {
        return InstructionReturn::cont();
    }
    let memory_offset = as_usize_or_fail!(context, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(context, memory_offset, len);

    // Note: This can't panic because we resized memory to fit.
    context
        .memory()
        .set_data(memory_offset, code_offset, len, &code);
    InstructionReturn::cont()
}

/// Implements the BLOCKHASH instruction.
///
/// Gets the hash of one of the 256 most recent complete blocks.
#[inline]
pub fn blockhash<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BLOCKHASH);
    popn_top!([], number, context);

    let requested_number = *number;
    let block_number = context.host().block_number();

    let Some(diff) = block_number.checked_sub(requested_number) else {
        *number = U256::ZERO;
        return InstructionReturn::cont();
    };

    let diff = as_u64_saturated!(diff);

    // blockhash should push zero if number is same as current block number.
    if diff == 0 {
        *number = U256::ZERO;
        return InstructionReturn::cont();
    }

    *number = if diff <= BLOCK_HASH_HISTORY {
        let Some(hash) = context
            .host()
            .block_hash(as_u64_saturated!(requested_number))
        else {
            return context.halt(InstructionResult::FatalExternalError);
        };
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    };
    InstructionReturn::cont()
}

/// Implements the SLOAD instruction.
///
/// Loads a word from storage.
#[inline]
pub fn sload<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn_top!([], index, context);

    let address = context.input().target_address();
    let Some(value) = context.host().sload(address, *index) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    gas!(
        context,
        gas::sload_cost(context.runtime_flag().spec_id(), value.is_cold)
    );
    *index = value.data;
    InstructionReturn::cont()
}

/// Implements the SSTORE instruction.
///
/// Stores a word to storage.
#[inline]
pub fn sstore<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    require_non_staticcall!(context);

    popn!([index, value], context);

    let address = context.input().target_address();
    let Some(state_load) = context.host().sstore(address, index, value) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    let spec_id = context.runtime_flag().spec_id();
    if spec_id.is_enabled_in(ISTANBUL) && context.gas().remaining() <= CALL_STIPEND {
        return context.halt(InstructionResult::ReentrancySentryOOG);
    }
    gas!(
        context,
        gas::sstore_cost(spec_id, &state_load.data, state_load.is_cold)
    );

    context.record_refund(gas::sstore_refund(spec_id, &state_load.data));
    InstructionReturn::cont()
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
#[inline]
pub fn tstore<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CANCUN);
    require_non_staticcall!(context);
    gas!(context, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], context);

    let address = context.input().target_address();
    context.host().tstore(address, index, value);
    InstructionReturn::cont()
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
#[inline]
pub fn tload<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CANCUN);
    gas!(context, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, context);

    let address = context.input().target_address();
    *index = context.host().tload(address, *index);
    InstructionReturn::cont()
}

/// Implements the LOG0-LOG4 instructions.
///
/// Appends log record with N topics.
#[inline]
pub fn log<const N: usize, C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    require_non_staticcall!(context);

    popn!([offset, len], context);
    let len = as_usize_or_fail!(context, len);
    gas_or_fail!(context, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(context, offset);
        resize_memory!(context, offset, len);
        Bytes::copy_from_slice(context.memory().slice_len(offset, len).as_ref())
    };
    let Some(topics) = context.stack().popn::<N>() else {
        return context.halt(InstructionResult::StackUnderflow);
    };

    let log = Log {
        address: context.input().target_address(),
        data: LogData::new(topics.map(B256::from).to_vec(), data)
            .expect("LogData should have <=4 topics"),
    };

    context.host().log(log);
    InstructionReturn::cont()
}

/// Implements the SELFDESTRUCT instruction.
///
/// Halt execution and register account for later deletion.
#[inline]
pub fn selfdestruct<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    require_non_staticcall!(context);
    popn!([target], context);
    let target = target.into_address();

    let address = context.input().target_address();
    let Some(res) = context.host().selfdestruct(address, target) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    // EIP-3529: Reduction in refunds
    if !context.runtime_flag().spec_id().is_enabled_in(LONDON) && !res.previously_destroyed {
        context.record_refund(gas::SELFDESTRUCT)
    }

    gas!(
        context,
        gas::selfdestruct_cost(context.runtime_flag().spec_id(), res)
    );

    context.halt(InstructionResult::SelfDestruct)
}
