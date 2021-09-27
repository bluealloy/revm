#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod codes;
pub(crate) mod gas;
mod i256;
mod misc;
mod system;

pub use codes::OpCode;

use crate::{
    error::{ExitError, ExitReason, ExitSucceed},
    machine::Machine,
    spec::Spec,
    CallScheme, ExtHandler,
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

#[inline]
pub fn eval<
    H: ExtHandler,
    S: Spec,
>(
    state: &mut Machine,
    opcode: OpCode,
    position: usize,
    handler: &mut H,
) -> Control {
    match opcode {
        OpCode::STOP => Control::Exit(ExitSucceed::Stopped.into()),
        OpCode::ADD => op2_u256_tuple!(state, overflowing_add, gas::VERYLOW),
        OpCode::MUL => op2_u256_tuple!(state, overflowing_mul, gas::LOW),
        OpCode::SUB => op2_u256_tuple!(state, overflowing_sub, gas::VERYLOW),
        OpCode::DIV => op2_u256_fn!(state, arithmetic::div, gas::LOW),
        OpCode::SDIV => op2_u256_fn!(state, arithmetic::sdiv, gas::LOW),
        OpCode::MOD => op2_u256_fn!(state, arithmetic::rem, gas::LOW),
        OpCode::SMOD => op2_u256_fn!(state, arithmetic::srem, gas::LOW),
        OpCode::ADDMOD => op3_u256_fn!(state, arithmetic::addmod, gas::MID),
        OpCode::MULMOD => op3_u256_fn!(state, arithmetic::mulmod, gas::MID),
        OpCode::EXP => arithmetic::eval_exp::<S>(state),
        OpCode::SIGNEXTEND => op2_u256_fn!(state, arithmetic::signextend, gas::LOW),
        OpCode::LT => op2_u256_bool_ref!(state, lt, gas::VERYLOW),
        OpCode::GT => op2_u256_bool_ref!(state, gt, gas::VERYLOW),
        OpCode::SLT => op2_u256_fn!(state, bitwise::slt, gas::VERYLOW),
        OpCode::SGT => op2_u256_fn!(state, bitwise::sgt, gas::VERYLOW),
        OpCode::EQ => op2_u256_bool_ref!(state, eq, gas::VERYLOW),
        OpCode::ISZERO => op1_u256_fn!(state, bitwise::iszero, gas::VERYLOW),
        OpCode::AND => op2_u256!(state, bitand, gas::VERYLOW),
        OpCode::OR => op2_u256!(state, bitor, gas::VERYLOW),
        OpCode::XOR => op2_u256!(state, bitxor, gas::VERYLOW),
        OpCode::NOT => op1_u256_fn!(state, bitwise::not, gas::VERYLOW),
        OpCode::BYTE => op2_u256_fn!(state, bitwise::byte, gas::VERYLOW),
        OpCode::SHL => op2_u256_fn!(state, bitwise::shl, gas::VERYLOW, S::has_bitwise_shifting),
        OpCode::SHR => op2_u256_fn!(state, bitwise::shr, gas::VERYLOW, S::has_bitwise_shifting),
        OpCode::SAR => op2_u256_fn!(state, bitwise::sar, gas::VERYLOW, S::has_bitwise_shifting),
        OpCode::CODESIZE => misc::codesize(state),
        OpCode::CODECOPY => misc::codecopy(state),
        OpCode::CALLDATALOAD => misc::calldataload(state),
        OpCode::CALLDATASIZE => misc::calldatasize(state),
        OpCode::CALLDATACOPY => misc::calldatacopy(state),
        OpCode::POP => misc::pop(state),
        OpCode::MLOAD => misc::mload(state),
        OpCode::MSTORE => misc::mstore(state),
        OpCode::MSTORE8 => misc::mstore(state),
        OpCode::JUMP => misc::jump(state),
        OpCode::JUMPI => misc::jumpi(state),
        OpCode::PC => misc::pc(state, position),
        OpCode::MSIZE => misc::msize(state),
        OpCode::JUMPDEST => state.spend_gas(gas::JUMPDEST),

        OpCode::PUSH1 => misc::push(state, 1, position),
        OpCode::PUSH2 => misc::push(state, 2, position),
        OpCode::PUSH3 => misc::push(state, 3, position),
        OpCode::PUSH4 => misc::push(state, 4, position),
        OpCode::PUSH5 => misc::push(state, 5, position),
        OpCode::PUSH6 => misc::push(state, 6, position),
        OpCode::PUSH7 => misc::push(state, 7, position),
        OpCode::PUSH8 => misc::push(state, 8, position),
        OpCode::PUSH9 => misc::push(state, 9, position),
        OpCode::PUSH10 => misc::push(state, 10, position),
        OpCode::PUSH11 => misc::push(state, 11, position),
        OpCode::PUSH12 => misc::push(state, 12, position),
        OpCode::PUSH13 => misc::push(state, 13, position),
        OpCode::PUSH14 => misc::push(state, 14, position),
        OpCode::PUSH15 => misc::push(state, 15, position),
        OpCode::PUSH16 => misc::push(state, 16, position),
        OpCode::PUSH17 => misc::push(state, 17, position),
        OpCode::PUSH18 => misc::push(state, 18, position),
        OpCode::PUSH19 => misc::push(state, 19, position),
        OpCode::PUSH20 => misc::push(state, 20, position),
        OpCode::PUSH21 => misc::push(state, 21, position),
        OpCode::PUSH22 => misc::push(state, 22, position),
        OpCode::PUSH23 => misc::push(state, 23, position),
        OpCode::PUSH24 => misc::push(state, 24, position),
        OpCode::PUSH25 => misc::push(state, 25, position),
        OpCode::PUSH26 => misc::push(state, 26, position),
        OpCode::PUSH27 => misc::push(state, 27, position),
        OpCode::PUSH28 => misc::push(state, 28, position),
        OpCode::PUSH29 => misc::push(state, 29, position),
        OpCode::PUSH30 => misc::push(state, 30, position),
        OpCode::PUSH31 => misc::push(state, 31, position),
        OpCode::PUSH32 => misc::push(state, 32, position),

        OpCode::DUP1 => misc::dup(state, 1),
        OpCode::DUP2 => misc::dup(state, 2),
        OpCode::DUP3 => misc::dup(state, 3),
        OpCode::DUP4 => misc::dup(state, 4),
        OpCode::DUP5 => misc::dup(state, 5),
        OpCode::DUP6 => misc::dup(state, 6),
        OpCode::DUP7 => misc::dup(state, 7),
        OpCode::DUP8 => misc::dup(state, 8),
        OpCode::DUP9 => misc::dup(state, 9),
        OpCode::DUP10 => misc::dup(state, 10),
        OpCode::DUP11 => misc::dup(state, 11),
        OpCode::DUP12 => misc::dup(state, 12),
        OpCode::DUP13 => misc::dup(state, 13),
        OpCode::DUP14 => misc::dup(state, 14),
        OpCode::DUP15 => misc::dup(state, 15),
        OpCode::DUP16 => misc::dup(state, 16),

        OpCode::SWAP1 => misc::swap(state, 1),
        OpCode::SWAP2 => misc::swap(state, 2),
        OpCode::SWAP3 => misc::swap(state, 3),
        OpCode::SWAP4 => misc::swap(state, 4),
        OpCode::SWAP5 => misc::swap(state, 5),
        OpCode::SWAP6 => misc::swap(state, 6),
        OpCode::SWAP7 => misc::swap(state, 7),
        OpCode::SWAP8 => misc::swap(state, 8),
        OpCode::SWAP9 => misc::swap(state, 9),
        OpCode::SWAP10 => misc::swap(state, 10),
        OpCode::SWAP11 => misc::swap(state, 11),
        OpCode::SWAP12 => misc::swap(state, 12),
        OpCode::SWAP13 => misc::swap(state, 13),
        OpCode::SWAP14 => misc::swap(state, 14),
        OpCode::SWAP15 => misc::swap(state, 15),
        OpCode::SWAP16 => misc::swap(state, 16),

        OpCode::RETURN => misc::ret(state),
        OpCode::REVERT => misc::revert::<S>(state),
        OpCode::INVALID => Control::Exit(ExitError::DesignatedInvalid.into()),
        OpCode::SHA3 => system::sha3(state),
        OpCode::ADDRESS => system::address(state),
        OpCode::BALANCE => system::balance::<H,S>(state, handler),
        OpCode::SELFBALANCE => system::selfbalance::<H,S>(state, handler),
        OpCode::ORIGIN => system::origin(state, handler),
        OpCode::CALLER => system::caller(state),
        OpCode::CALLVALUE => system::callvalue(state),
        OpCode::GASPRICE => system::gasprice(state, handler),
        OpCode::EXTCODESIZE => system::extcodesize::<H,S>(state, handler),
        OpCode::EXTCODEHASH => system::extcodehash::<H,S>(state, handler),
        OpCode::EXTCODECOPY => system::extcodecopy::<H,S>(state, handler),
        OpCode::RETURNDATASIZE => system::returndatasize::<S>(state),
        OpCode::RETURNDATACOPY => system::returndatacopy::<S>(state),
        OpCode::BLOCKHASH => system::blockhash(state, handler),
        OpCode::COINBASE => system::coinbase(state, handler),
        OpCode::TIMESTAMP => system::timestamp(state, handler),
        OpCode::NUMBER => system::number(state, handler),
        OpCode::DIFFICULTY => system::difficulty(state, handler),
        OpCode::GASLIMIT => system::gaslimit(state, handler),
        OpCode::SLOAD => system::sload::<H,false>(state, handler), //check
        OpCode::SSTORE => system::sstore::<H, S>(state, handler), //check
        OpCode::GAS => system::gas(state, handler),
        OpCode::LOG0 => system::log::<H,S>(state, 0, handler),
        OpCode::LOG1 => system::log::<H,S>(state, 1, handler),
        OpCode::LOG2 => system::log::<H,S>(state, 2, handler),
        OpCode::LOG3 => system::log::<H,S>(state, 3, handler),
        OpCode::LOG4 => system::log::<H,S>(state, 4, handler),
        OpCode::SUICIDE => system::suicide::<H,S>(state, handler),
        OpCode::CREATE => system::create::<H,S>(state, false, handler), //check
        OpCode::CREATE2 => system::create::<H,S>(state, true, handler), //check
        OpCode::CALL => system::call::<H,S>(state, CallScheme::Call, handler), //check
        OpCode::CALLCODE => system::call::<H,S>(state, CallScheme::CallCode, handler), //check
        OpCode::DELEGATECALL => system::call::<H,S>(state, CallScheme::DelegateCall, handler), //check
        OpCode::STATICCALL => system::call::<H,S>(state, CallScheme::StaticCall, handler), //check
        OpCode::CHAINID => system::chainid::<H, S>(state, handler),
    }
}
