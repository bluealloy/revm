use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(host.effective_gas_price()));
}

pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.caller().into_word().into());
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::VERYLOW);
    popn_top!([], index, interpreter);
    let i = as_usize_saturated!(index);
    *index = host.blob_hash(i).unwrap_or_default();
}
