use crate::{
    instructions::utility::{IntoAddress, IntoU256},
    interpreter_types::{InputsTr, InterpreterTypes as ITy, MemoryTr, RuntimeFlag, StackTr},
    Gas, Host, InstructionExecResult as Result, InstructionResult,
};
use context_interface::{host::LoadError, journaled_state::AccountInfoLoad};
use core::cmp::min;
use primitives::{
    hardfork::SpecId::{self, *},
    Bytes, Log, LogData, B256, BLOCK_HASH_HISTORY, U256,
};

use crate::InstructionContext as Ictx;

/// Loads an account, handling cold load gas accounting.
///
/// Pre-Berlin, `cold_account_additional_cost` is 0, so the cold load logic is a no-op.
fn load_account<'a, H: Host + ?Sized>(
    gas: &mut Gas,
    host: &'a mut H,
    address: primitives::Address,
    load_code: bool,
) -> core::result::Result<AccountInfoLoad<'a>, LoadError> {
    let cold_load_gas = host.gas_params().cold_account_additional_cost();
    let skip_cold_load = gas.remaining() < cold_load_gas;
    let account = host.load_account_info_skip_cold_load(address, load_code, skip_cold_load)?;
    if account.is_cold && !gas.record_regular_cost(cold_load_gas) {
        return Err(LoadError::ColdLoadSkipped);
    }
    Ok(account)
}

/// Implements the BALANCE instruction.
///
/// Gets the balance of the given account.
pub fn balance<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let account = load_account(&mut context.interpreter.gas, context.host, address, false)?;
    *top = account.balance;
    Ok(())
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub fn selfbalance<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, ISTANBUL);

    let balance = context
        .host
        .balance(context.interpreter.input.target_address())
        .ok_or(InstructionResult::FatalExternalError)?;
    push!(context.interpreter, balance.data);
    Ok(())
}

/// Implements the EXTCODESIZE instruction.
///
/// Gets the size of an account's code.
pub fn extcodesize<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let account = load_account(&mut context.interpreter.gas, context.host, address, true)?;
    // safe to unwrap because we are loading code
    *top = U256::from(account.code.as_ref().unwrap().len());
    Ok(())
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, CONSTANTINOPLE);
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let account = load_account(&mut context.interpreter.gas, context.host, address, false)?;
    // if account is empty, code hash is zero
    let code_hash = if account.is_empty() {
        B256::ZERO
    } else {
        account.code_hash
    };
    *top = code_hash.into_u256();
    Ok(())
}

/// Implements the EXTCODECOPY instruction.
///
/// Copies a portion of an account's code to memory.
pub fn extcodecopy<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn!(
        [address, memory_offset, code_offset, len_u256],
        context.interpreter
    );
    let address = address.into_address();

    let len = as_usize_or_fail!(context.interpreter, len_u256);
    gas!(
        context.interpreter,
        context.host.gas_params().extcodecopy(len)
    );

    let mut memory_offset_usize = 0;
    // resize memory only if len is not zero
    if len != 0 {
        // fail on casting of memory_offset only if len is not zero.
        memory_offset_usize = as_usize_or_fail!(context.interpreter, memory_offset);
        // Resize memory to fit the code
        context
            .interpreter
            .resize_memory(context.host.gas_params(), memory_offset_usize, len)?;
    }

    let account = load_account(&mut context.interpreter.gas, context.host, address, true)?;
    let code = account.code.as_ref().unwrap().original_bytes();

    let code_offset_usize = min(as_usize_saturated!(code_offset), code.len());

    // Note: This can't panic because we resized memory to fit.
    // len zero is handled in set_data
    context
        .interpreter
        .memory
        .set_data(memory_offset_usize, code_offset_usize, len, &code);
    Ok(())
}

/// Implements the BLOCKHASH instruction.
///
/// Gets the hash of one of the 256 most recent complete blocks.
pub fn blockhash<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([], number, context.interpreter);

    let requested_number = *number;
    let block_number = context.host.block_number();

    let Some(diff) = block_number.checked_sub(requested_number) else {
        *number = U256::ZERO;
        return Ok(());
    };

    let diff = as_u64_saturated!(diff);

    // blockhash should push zero if number is same as current block number.
    if diff == 0 {
        *number = U256::ZERO;
        return Ok(());
    }

    *number = if diff <= BLOCK_HASH_HISTORY {
        let hash = context
            .host
            .block_hash(as_u64_saturated!(requested_number))
            .ok_or(InstructionResult::FatalExternalError)?;
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    };
    Ok(())
}

/// Implements the SLOAD instruction.
///
/// Loads a word from storage.
pub fn sload<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([], index, context.interpreter);
    let spec_id = context.interpreter.runtime_flag.spec_id();
    let target = context.interpreter.input.target_address();

    if spec_id.is_enabled_in(BERLIN) {
        let additional_cold_cost = context.host.gas_params().cold_storage_additional_cost();
        let skip_cold = context.interpreter.gas.remaining() < additional_cold_cost;
        let storage = context
            .host
            .sload_skip_cold_load(target, *index, skip_cold)?;
        if storage.is_cold {
            gas!(context.interpreter, additional_cold_cost);
        }
        *index = storage.data;
    } else {
        let storage = context
            .host
            .sload(target, *index)
            .ok_or(InstructionResult::FatalExternalError)?;
        *index = storage.data;
    };
    Ok(())
}

/// Implements the SSTORE instruction.
///
/// Stores a word to storage.
pub fn sstore<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    require_non_staticcall!(context.interpreter);
    popn!([index, value], context.interpreter);

    let target = context.interpreter.input.target_address();
    let spec_id = context.interpreter.runtime_flag.spec_id();

    // EIP-2200: Structured Definitions for Net Gas Metering
    // If gasleft is less than or equal to gas stipend, fail the current call frame with 'out of gas' exception.
    if spec_id.is_enabled_in(ISTANBUL)
        && context.interpreter.gas.remaining() <= context.host.gas_params().call_stipend()
    {
        return Err(InstructionResult::ReentrancySentryOOG);
    }

    gas!(
        context.interpreter,
        context.host.gas_params().sstore_static_gas()
    );

    let state_load = if spec_id.is_enabled_in(BERLIN) {
        let additional_cold_cost = context.host.gas_params().cold_storage_additional_cost();
        let skip_cold = context.interpreter.gas.remaining() < additional_cold_cost;
        context
            .host
            .sstore_skip_cold_load(target, index, value, skip_cold)?
    } else {
        context
            .host
            .sstore(target, index, value)
            .ok_or(InstructionResult::FatalExternalError)?
    };

    let is_istanbul = spec_id.is_enabled_in(ISTANBUL);

    // dynamic gas
    gas!(
        context.interpreter,
        context.host.gas_params().sstore_dynamic_gas(
            is_istanbul,
            &state_load.data,
            state_load.is_cold
        )
    );

    // state gas for new slot creation (EIP-8037)
    if context.host.is_amsterdam_eip8037_enabled() {
        state_gas!(
            context.interpreter,
            context.host.gas_params().sstore_state_gas(&state_load.data)
        );
    }

    // refund
    context.interpreter.gas.record_refund(
        context
            .host
            .gas_params()
            .sstore_refund(is_istanbul, &state_load.data),
    );
    Ok(())
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, CANCUN);
    require_non_staticcall!(context.interpreter);
    popn!([index, value], context.interpreter);

    context
        .host
        .tstore(context.interpreter.input.target_address(), index, value);
    Ok(())
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, CANCUN);
    popn_top!([], index, context.interpreter);

    *index = context
        .host
        .tload(context.interpreter.input.target_address(), *index);
    Ok(())
}

/// Implements the LOG0-LOG4 instructions.
///
/// Appends log record with N topics.
pub fn log<const N: usize, H: Host + ?Sized>(context: Ictx<'_, H, impl ITy>) -> Result {
    require_non_staticcall!(context.interpreter);

    popn!([offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    gas!(
        context.interpreter,
        context.host.gas_params().log_cost(N as u8, len as u64)
    );
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(context.interpreter, offset);
        // Resize memory to fit the data
        context
            .interpreter
            .resize_memory(context.host.gas_params(), offset, len)?;
        Bytes::copy_from_slice(context.interpreter.memory.slice_len(offset, len).as_ref())
    };
    let Some(topics) = context.interpreter.stack.popn::<N>() else {
        return Err(InstructionResult::StackUnderflow);
    };

    let log = Log {
        address: context.interpreter.input.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    context.host.log(log);
    Ok(())
}

/// Implements the SELFDESTRUCT instruction.
///
/// Halt execution and register account for later deletion.
pub fn selfdestruct<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    require_non_staticcall!(context.interpreter);
    popn!([target], context.interpreter);
    let target = target.into_address();
    let spec = context.interpreter.runtime_flag.spec_id();

    let cold_load_gas = context.host.gas_params().selfdestruct_cold_cost();

    let skip_cold_load = context.interpreter.gas.remaining() < cold_load_gas;
    let res = context.host.selfdestruct(
        context.interpreter.input.target_address(),
        target,
        skip_cold_load,
    )?;

    // EIP-161: State trie clearing (invariant-preserving alternative)
    let should_charge_topup = if spec.is_enabled_in(SpecId::SPURIOUS_DRAGON) {
        res.had_value && !res.target_exists
    } else {
        !res.target_exists
    };

    gas!(
        context.interpreter,
        context
            .host
            .gas_params()
            .selfdestruct_cost(should_charge_topup, res.is_cold)
    );

    // State gas for new account creation (EIP-8037)
    if context.host.is_amsterdam_eip8037_enabled() && should_charge_topup {
        state_gas!(
            context.interpreter,
            context.host.gas_params().new_account_state_gas()
        );
    }

    if !res.previously_destroyed {
        context
            .interpreter
            .gas
            .record_refund(context.host.gas_params().selfdestruct_refund());
    }

    Err(InstructionResult::SelfDestruct)
}
