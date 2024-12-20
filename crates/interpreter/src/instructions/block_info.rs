use crate::{
    gas,
    instructions::utility::IntoU256,
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, LoopControl, RuntimeFlag, StackTrait},
    Host,
};
use context_interface::{Block, Cfg};
use primitives::U256;
use specification::hardfork::SpecId::*;

/// EIP-1344: ChainID opcode
pub fn chainid<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(host.cfg().chain_id()));
}

pub fn coinbase<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.block().beneficiary().into_word().into());
}

pub fn timestamp<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.block().timestamp());
}

pub fn block_number<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.block().number());
}

pub fn difficulty<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    if interpreter.runtime_flag.spec_id().is_enabled_in(MERGE) {
        // Unwrap is safe as this fields is checked in validation handler.
        push!(
            interpreter,
            (*host.block().prevrandao().unwrap()).into_u256()
        );
    } else {
        push!(interpreter, *host.block().difficulty());
    }
}

pub fn gaslimit<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.block().gas_limit());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, LONDON);
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.block().basefee());
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(host.block().blob_gasprice().unwrap_or_default())
    );
}
