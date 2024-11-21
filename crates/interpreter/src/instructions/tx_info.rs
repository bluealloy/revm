use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_wiring::{InterpreterTypes, LoopControl, RuntimeFlag, StackTrait},
    Host,
};
use context_interface::{transaction::Eip4844Tx, Block, Transaction, TransactionType};
use primitives::U256;

pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    let basefee = *host.block().basefee();
    push!(interpreter, host.tx().effective_gas_price(basefee));
    push!(interpreter, U256::ZERO)
}

pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        host.tx().common_fields().caller().into_word().into()
    );
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
    let tx = &host.tx();
    *index = if tx.tx_type().into() == TransactionType::Eip4844 {
        tx.eip4844()
            .blob_versioned_hashes()
            .get(i)
            .cloned()
            .map(|b| U256::from_be_bytes(*b))
            .unwrap_or(U256::ZERO)
    } else {
        U256::ZERO
    };
}
