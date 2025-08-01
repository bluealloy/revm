use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{RuntimeFlag, StackTr},
    Host, InstructionContextTr,
};
use primitives::{hardfork::SpecId::*, U256};

/// EIP-1344: ChainID opcode
#[inline]
pub fn chainid<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, ISTANBUL);
    gas!(context, gas::BASE);
    push!(context, context.host().chain_id());
    InstructionReturn::cont()
}

/// Implements the COINBASE instruction.
///
/// Pushes the current block's beneficiary address onto the stack.
#[inline]
pub fn coinbase<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.host().beneficiary().into_word().into());
    InstructionReturn::cont()
}

/// Implements the TIMESTAMP instruction.
///
/// Pushes the current block's timestamp onto the stack.
#[inline]
pub fn timestamp<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.host().timestamp());
    InstructionReturn::cont()
}

/// Implements the NUMBER instruction.
///
/// Pushes the current block number onto the stack.
#[inline]
pub fn block_number<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.host().block_number()));
    InstructionReturn::cont()
}

/// Implements the DIFFICULTY/PREVRANDAO instruction.
///
/// Pushes the block difficulty (pre-merge) or prevrandao (post-merge) onto the stack.
#[inline]
pub fn difficulty<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    let value = if context.runtime_flag().spec_id().is_enabled_in(MERGE) {
        // Unwrap is safe as this fields is checked in validation handler.
        context.host().prevrandao().unwrap()
    } else {
        context.host().difficulty()
    };
    push!(context, value);
    InstructionReturn::cont()
}

/// Implements the GASLIMIT instruction.
///
/// Pushes the current block's gas limit onto the stack.
#[inline]
pub fn gaslimit<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.host().gas_limit());
    InstructionReturn::cont()
}

/// EIP-3198: BASEFEE opcode
#[inline]
pub fn basefee<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, LONDON);
    gas!(context, gas::BASE);
    push!(context, context.host().basefee());
    InstructionReturn::cont()
}

/// EIP-7516: BLOBBASEFEE opcode
#[inline]
pub fn blob_basefee<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CANCUN);
    gas!(context, gas::BASE);
    push!(context, context.host().blob_gasprice());
    InstructionReturn::cont()
}
