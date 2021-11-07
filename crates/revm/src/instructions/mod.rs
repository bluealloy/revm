#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
pub(crate) mod gas;
mod i256;
mod misc;
pub mod opcode;
mod system;

pub use opcode::OpCode;

use crate::{
    error::{ExitError, ExitReason, ExitSucceed},
    machine::Machine,
    spec::{Spec, SpecId::*},
    CallScheme, Handler,
};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control {
    Continue,
    ContinueN(usize),
    Exit(ExitReason),
    Jump(usize),
}

#[inline(always)]
pub fn eval<H: Handler, S: Spec>(
    machine: &mut Machine,
    opcode: OpCode,
    position: usize,
    handler: &mut H,
) -> Control {
    let opcode = opcode.as_u8();
    match opcode {
        opcode::STOP => Control::Exit(ExitSucceed::Stopped.into()),
        opcode::ADD => op2_u256_tuple!(machine, overflowing_add, gas::VERYLOW),
        opcode::MUL => op2_u256_tuple!(machine, overflowing_mul, gas::LOW),
        opcode::SUB => op2_u256_tuple!(machine, overflowing_sub, gas::VERYLOW),
        opcode::DIV => op2_u256_fn!(machine, arithmetic::div, gas::LOW),
        opcode::SDIV => op2_u256_fn!(machine, arithmetic::sdiv, gas::LOW),
        opcode::MOD => op2_u256_fn!(machine, arithmetic::rem, gas::LOW),
        opcode::SMOD => op2_u256_fn!(machine, arithmetic::srem, gas::LOW),
        opcode::ADDMOD => op3_u256_fn!(machine, arithmetic::addmod, gas::MID),
        opcode::MULMOD => op3_u256_fn!(machine, arithmetic::mulmod, gas::MID),
        opcode::EXP => arithmetic::eval_exp::<S>(machine),
        opcode::SIGNEXTEND => op2_u256_fn!(machine, arithmetic::signextend, gas::LOW),
        opcode::LT => op2_u256_bool_ref!(machine, lt, gas::VERYLOW),
        opcode::GT => op2_u256_bool_ref!(machine, gt, gas::VERYLOW),
        opcode::SLT => op2_u256_fn!(machine, bitwise::slt, gas::VERYLOW),
        opcode::SGT => op2_u256_fn!(machine, bitwise::sgt, gas::VERYLOW),
        opcode::EQ => op2_u256_bool_ref!(machine, eq, gas::VERYLOW),
        opcode::ISZERO => op1_u256_fn!(machine, bitwise::iszero, gas::VERYLOW),
        opcode::AND => op2_u256!(machine, bitand, gas::VERYLOW),
        opcode::OR => op2_u256!(machine, bitor, gas::VERYLOW),
        opcode::XOR => op2_u256!(machine, bitxor, gas::VERYLOW),
        opcode::NOT => op1_u256_fn!(machine, bitwise::not, gas::VERYLOW),
        opcode::BYTE => op2_u256_fn!(machine, bitwise::byte, gas::VERYLOW),
        opcode::SHL => op2_u256_fn!(
            machine,
            bitwise::shl,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SHR => op2_u256_fn!(
            machine,
            bitwise::shr,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SAR => op2_u256_fn!(
            machine,
            bitwise::sar,
            gas::VERYLOW,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::CODESIZE => misc::codesize(machine),
        opcode::CODECOPY => misc::codecopy(machine),
        opcode::CALLDATALOAD => misc::calldataload(machine),
        opcode::CALLDATASIZE => misc::calldatasize(machine),
        opcode::CALLDATACOPY => misc::calldatacopy(machine),
        opcode::POP => misc::pop(machine),
        opcode::MLOAD => misc::mload(machine),
        opcode::MSTORE => misc::mstore(machine),
        opcode::MSTORE8 => misc::mstore8(machine),
        opcode::JUMP => misc::jump(machine),
        opcode::JUMPI => misc::jumpi(machine),
        opcode::PC => misc::pc(machine, position),
        opcode::MSIZE => misc::msize(machine),
        opcode::JUMPDEST => misc::jumpdest(machine),
        opcode::PUSH1..=opcode::PUSH32 => {
            misc::push(machine, (1 + opcode - opcode::PUSH1) as usize, position)
        }
        opcode::DUP1..=opcode::DUP16 => misc::dup(machine, (1 + opcode - opcode::DUP1) as usize),
        opcode::SWAP1..=opcode::SWAP16 => {
            misc::swap(machine, (1 + opcode - opcode::SWAP1) as usize)
        }
        opcode::RETURN => misc::ret(machine),
        opcode::REVERT => misc::revert::<S>(machine),
        opcode::INVALID => Control::Exit(ExitError::DesignatedInvalid.into()),
        opcode::SHA3 => system::sha3(machine),
        opcode::ADDRESS => system::address(machine),
        opcode::BALANCE => system::balance::<H, S>(machine, handler),
        opcode::SELFBALANCE => system::selfbalance::<H, S>(machine, handler),
        opcode::BASEFEE => system::basefee::<H, S>(machine, handler),
        opcode::ORIGIN => system::origin(machine, handler),
        opcode::CALLER => system::caller(machine),
        opcode::CALLVALUE => system::callvalue(machine),
        opcode::GASPRICE => system::gasprice(machine, handler),
        opcode::EXTCODESIZE => system::extcodesize::<H, S>(machine, handler),
        opcode::EXTCODEHASH => system::extcodehash::<H, S>(machine, handler),
        opcode::EXTCODECOPY => system::extcodecopy::<H, S>(machine, handler),
        opcode::RETURNDATASIZE => system::returndatasize::<S>(machine),
        opcode::RETURNDATACOPY => system::returndatacopy::<S>(machine),
        opcode::BLOCKHASH => system::blockhash(machine, handler),
        opcode::COINBASE => system::coinbase(machine, handler),
        opcode::TIMESTAMP => system::timestamp(machine, handler),
        opcode::NUMBER => system::number(machine, handler),
        opcode::DIFFICULTY => system::difficulty(machine, handler),
        opcode::GASLIMIT => system::gaslimit(machine, handler),
        opcode::SLOAD => system::sload::<H, S>(machine, handler),
        opcode::SSTORE => system::sstore::<H, S>(machine, handler),
        opcode::GAS => system::gas(machine),
        opcode::LOG0 => system::log::<H, S>(machine, 0, handler),
        opcode::LOG1 => system::log::<H, S>(machine, 1, handler),
        opcode::LOG2 => system::log::<H, S>(machine, 2, handler),
        opcode::LOG3 => system::log::<H, S>(machine, 3, handler),
        opcode::LOG4 => system::log::<H, S>(machine, 4, handler),
        opcode::SELFDESTRUCT => system::selfdestruct::<H, S>(machine, handler),
        opcode::CREATE => system::create::<H, S>(machine, false, handler), //check
        opcode::CREATE2 => system::create::<H, S>(machine, true, handler), //check
        opcode::CALL => system::call::<H, S>(machine, CallScheme::Call, handler), //check
        opcode::CALLCODE => system::call::<H, S>(machine, CallScheme::CallCode, handler), //check
        opcode::DELEGATECALL => system::call::<H, S>(machine, CallScheme::DelegateCall, handler), //check
        opcode::STATICCALL => system::call::<H, S>(machine, CallScheme::StaticCall, handler), //check
        opcode::CHAINID => system::chainid::<H, S>(machine, handler),
        _ => Control::Exit(ExitReason::Error(ExitError::OpcodeNotFound)),
    }
}
