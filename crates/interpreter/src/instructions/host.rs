use crate::{
    gas::{
        self, CALL_STIPEND, COLD_ACCOUNT_ACCESS_COST_ADDITIONAL, COLD_SLOAD_COST_ADDITIONAL,
        ISTANBUL_SLOAD_GAS, WARM_STORAGE_READ_COST,
    },
    instructions::utility::{IntoAddress, IntoU256},
    interpreter_types::{InputsTr, InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
    Host, InstructionResult,
};
use context_interface::host::LoadError;
use core::cmp::min;
use primitives::{hardfork::SpecId::*, Bytes, Log, LogData, B256, BLOCK_HASH_HISTORY, U256};

use crate::InstructionContext;

/// Implements the BALANCE instruction.
///
/// Gets the balance of the given account.
pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let spec_id = context.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        let account = berlin_load_account!(context, address, false);
        *top = account.balance;
    } else {
        let gas = if spec_id.is_enabled_in(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            700
        } else if spec_id.is_enabled_in(TANGERINE) {
            400
        } else {
            20
        };
        gas!(context.interpreter, gas);
        let Ok(account) = context
            .host
            .load_account_info_skip_cold_load(address, false, false)
        else {
            return context.interpreter.halt_fatal();
        };
        *top = account.balance;
    };
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub fn selfbalance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, ISTANBUL);
    //gas!(context.interpreter, gas::LOW);

    let Some(balance) = context
        .host
        .balance(context.interpreter.input.target_address())
    else {
        return context.interpreter.halt_fatal();
    };
    push!(context.interpreter, balance.data);
}

/// Implements the EXTCODESIZE instruction.
///
/// Gets the size of an account's code.
pub fn extcodesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let spec_id = context.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        let account = berlin_load_account!(context, address, true);
        // safe to unwrap because we are loading code
        *top = U256::from(account.code.as_ref().unwrap().len());
    } else {
        let gas = if spec_id.is_enabled_in(TANGERINE) {
            700
        } else {
            20
        };
        gas!(context.interpreter, gas);
        let Ok(account) = context
            .host
            .load_account_info_skip_cold_load(address, true, false)
        else {
            return context.interpreter.halt_fatal();
        };
        // safe to unwrap because we are loading code
        *top = U256::from(account.code.as_ref().unwrap().len());
    }
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CONSTANTINOPLE);
    popn_top!([], top, context.interpreter);
    let address = top.into_address();

    let spec_id = context.interpreter.runtime_flag.spec_id();
    let account = if spec_id.is_enabled_in(BERLIN) {
        berlin_load_account!(context, address, true)
    } else {
        let gas = if spec_id.is_enabled_in(ISTANBUL) {
            700
        } else {
            400
        };
        gas!(context.interpreter, gas);
        let Ok(account) = context
            .host
            .load_account_info_skip_cold_load(address, true, false)
        else {
            return context.interpreter.halt_fatal();
        };
        account
    };
    // if account is empty, code hash is zero
    let code_hash = if account.is_empty() {
        B256::ZERO
    } else {
        account.code_hash
    };
    *top = code_hash.into_u256();
}

/// Implements the EXTCODECOPY instruction.
///
/// Copies a portion of an account's code to memory.
pub fn extcodecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    popn!(
        [address, memory_offset, code_offset, len_u256],
        context.interpreter
    );
    let address = address.into_address();

    let spec_id = context.interpreter.runtime_flag.spec_id();

    let len = as_usize_or_fail!(context.interpreter, len_u256);
    gas!(
        context.interpreter,
        gas::copy_cost(0, len).unwrap_or(u64::MAX)
    );

    let mut memory_offset_usize = 0;
    // resize memory only if len is not zero
    if len != 0 {
        // fail on casting of memory_offset only if len is not zero.
        memory_offset_usize = as_usize_or_fail!(context.interpreter, memory_offset);
        resize_memory!(context.interpreter, memory_offset_usize, len);
    }

    let code = if spec_id.is_enabled_in(BERLIN) {
        let account = berlin_load_account!(context, address, true);
        account.code.as_ref().unwrap().original_bytes()
    } else {
        let gas = if spec_id.is_enabled_in(TANGERINE) {
            700
        } else {
            20
        };
        gas!(context.interpreter, gas);

        let Some(code) = context.host.load_account_code(address) else {
            return context.interpreter.halt_fatal();
        };
        code.data
    };

    let code_offset_usize = min(as_usize_saturated!(code_offset), code.len());

    // Note: This can't panic because we resized memory to fit.
    // len zero is handled in set_data
    context
        .interpreter
        .memory
        .set_data(memory_offset_usize, code_offset_usize, len, &code);
}

/// Implements the BLOCKHASH instruction.
///
/// Gets the hash of one of the 256 most recent complete blocks.
pub fn blockhash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BLOCKHASH);
    popn_top!([], number, context.interpreter);

    let requested_number = *number;
    let block_number = context.host.block_number();

    let Some(diff) = block_number.checked_sub(requested_number) else {
        *number = U256::ZERO;
        return;
    };

    let diff = as_u64_saturated!(diff);

    // blockhash should push zero if number is same as current block number.
    if diff == 0 {
        *number = U256::ZERO;
        return;
    }

    *number = if diff <= BLOCK_HASH_HISTORY {
        let Some(hash) = context.host.block_hash(as_u64_saturated!(requested_number)) else {
            return context.interpreter.halt_fatal();
        };
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    }
}

/// Implements the SLOAD instruction.
///
/// Loads a word from storage.
pub fn sload<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    popn_top!([], index, context.interpreter);
    let spec_id = context.interpreter.runtime_flag.spec_id();
    let target = context.interpreter.input.target_address();

    // `SLOAD` opcode cost calculation.
    let gas = if spec_id.is_enabled_in(BERLIN) {
        WARM_STORAGE_READ_COST
    } else if spec_id.is_enabled_in(ISTANBUL) {
        // EIP-1884: Repricing for trie-size-dependent opcodes
        ISTANBUL_SLOAD_GAS
    } else if spec_id.is_enabled_in(TANGERINE) {
        // EIP-150: Gas cost changes for IO-heavy operations
        200
    } else {
        50
    };
    gas!(context.interpreter, gas);
    if spec_id.is_enabled_in(BERLIN) {
        let skip_cold = context.interpreter.gas.remaining() < COLD_SLOAD_COST_ADDITIONAL;
        let res = context.host.sload_skip_cold_load(target, *index, skip_cold);
        match res {
            Ok(storage) => {
                if storage.is_cold {
                    gas!(context.interpreter, COLD_SLOAD_COST_ADDITIONAL);
                }

                *index = storage.data;
            }
            Err(LoadError::ColdLoadSkipped) => context.interpreter.halt_oog(),
            Err(LoadError::DBError) => context.interpreter.halt_fatal(),
        }
    } else {
        let Some(storage) = context.host.sload(target, *index) else {
            return context.interpreter.halt_fatal();
        };
        *index = storage.data;
    };
}

/// Implements the SSTORE instruction.
///
/// Stores a word to storage.
pub fn sstore<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_non_staticcall!(context.interpreter);

    popn!([index, value], context.interpreter);

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(ISTANBUL)
        && context.interpreter.gas.remaining() <= CALL_STIPEND
    {
        context
            .interpreter
            .halt(InstructionResult::ReentrancySentryOOG);
        return;
    }

    // TODO spend warm gas before calling sstore.
    let Some(state_load) =
        context
            .host
            .sstore(context.interpreter.input.target_address(), index, value)
    else {
        return context.interpreter.halt_fatal();
    };

    gas!(
        context.interpreter,
        gas::sstore_cost(
            context.interpreter.runtime_flag.spec_id(),
            &state_load.data,
            state_load.is_cold
        )
    );

    context.interpreter.gas.record_refund(gas::sstore_refund(
        context.interpreter.runtime_flag.spec_id(),
        &state_load.data,
    ));
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, CANCUN);
    require_non_staticcall!(context.interpreter);
    //gas!(context.interpreter, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], context.interpreter);

    context
        .host
        .tstore(context.interpreter.input.target_address(), index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, CANCUN);
    //gas!(context.interpreter, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, context.interpreter);

    *index = context
        .host
        .tload(context.interpreter.input.target_address(), *index);
}

/// Implements the LOG0-LOG4 instructions.
///
/// Appends log record with N topics.
pub fn log<const N: usize, H: Host + ?Sized>(
    context: InstructionContext<'_, H, impl InterpreterTypes>,
) {
    require_non_staticcall!(context.interpreter);

    popn!([offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    gas_or_fail!(context.interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(context.interpreter, offset);
        resize_memory!(context.interpreter, offset, len);
        Bytes::copy_from_slice(context.interpreter.memory.slice_len(offset, len).as_ref())
    };
    if context.interpreter.stack.len() < N {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    }
    let Some(topics) = context.interpreter.stack.popn::<N>() else {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    };

    let log = Log {
        address: context.interpreter.input.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    context.host.log(log);
}

/// Implements the SELFDESTRUCT instruction.
///
/// Halt execution and register account for later deletion.
pub fn selfdestruct<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    require_non_staticcall!(context.interpreter);
    popn!([target], context.interpreter);
    let target = target.into_address();

    // TODO order of operations should be
    // * static gas should be spent, 5k gas
    // * warm load gas
    // * loading of account and checking cold load.
    // * if not existing additional gas for creation.

    let Some(res) = context
        .host
        .selfdestruct(context.interpreter.input.target_address(), target)
    else {
        context
            .interpreter
            .halt(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(LONDON)
        && !res.previously_destroyed
    {
        context.interpreter.gas.record_refund(gas::SELFDESTRUCT)
    }

    gas!(
        context.interpreter,
        gas::selfdestruct_cost(context.interpreter.runtime_flag.spec_id(), res)
    );

    context.interpreter.halt(InstructionResult::SelfDestruct);
}
