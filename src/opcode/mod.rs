#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod i256;
mod misc;
mod opcode_type;
mod system;

pub use opcode_type::OpCode;

use crate::{CallScheme, ExtHandler, Handler, Machine, error::{ExitError, ExitFatal, ExitReason, ExitSucceed}, spec::Spec};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::{H256, U256};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Control {
    Continue,
    ContinueN(usize),
    Exit(ExitReason),
    Jump(usize),
}

fn eval_stop() -> Control {
    Control::Exit(ExitSucceed::Stopped.into())
}

fn eval_add(state: &mut Machine) -> Control {
    op2_u256_tuple!(state, overflowing_add)
}

fn eval_mul(state: &mut Machine) -> Control {
    op2_u256_tuple!(state, overflowing_mul)
}

fn eval_sub(state: &mut Machine) -> Control {
    op2_u256_tuple!(state, overflowing_sub)
}

fn eval_div(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::div)
}

fn eval_sdiv(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::sdiv)
}

fn eval_mod(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::rem)
}

fn eval_smod(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::srem)
}

fn eval_addmod(state: &mut Machine) -> Control {
    op3_u256_fn!(state, self::arithmetic::addmod)
}

fn eval_mulmod(state: &mut Machine) -> Control {
    op3_u256_fn!(state, self::arithmetic::mulmod)
}

fn eval_exp(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::exp)
}

fn eval_signextend(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::arithmetic::signextend)
}

fn eval_lt(state: &mut Machine) -> Control {
    op2_u256_bool_ref!(state, lt)
}

fn eval_gt(state: &mut Machine) -> Control {
    op2_u256_bool_ref!(state, gt)
}

fn eval_slt(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::slt)
}

fn eval_sgt(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::sgt)
}

fn eval_eq(state: &mut Machine) -> Control {
    op2_u256_bool_ref!(state, eq)
}

fn eval_iszero(state: &mut Machine) -> Control {
    op1_u256_fn!(state, self::bitwise::iszero)
}

fn eval_and(state: &mut Machine) -> Control {
    op2_u256!(state, bitand)
}

fn eval_or(state: &mut Machine) -> Control {
    op2_u256!(state, bitor)
}

fn eval_xor(state: &mut Machine) -> Control {
    op2_u256!(state, bitxor)
}

fn eval_not(state: &mut Machine) -> Control {
    op1_u256_fn!(state, self::bitwise::not)
}

fn eval_byte(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::byte)
}

fn eval_shl(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::shl)
}

fn eval_shr(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::shr)
}

fn eval_sar(state: &mut Machine) -> Control {
    op2_u256_fn!(state, self::bitwise::sar)
}

fn eval_codesize(state: &mut Machine) -> Control {
    self::misc::codesize(state)
}

fn eval_codecopy(state: &mut Machine) -> Control {
    self::misc::codecopy(state)
}

fn eval_calldataload(state: &mut Machine) -> Control {
    self::misc::calldataload(state)
}

fn eval_calldatasize(state: &mut Machine) -> Control {
    self::misc::calldatasize(state)
}

fn eval_calldatacopy(state: &mut Machine) -> Control {
    self::misc::calldatacopy(state)
}

fn eval_pop(state: &mut Machine) -> Control {
    self::misc::pop(state)
}

fn eval_mload(state: &mut Machine) -> Control {
    self::misc::mload(state)
}

fn eval_mstore(state: &mut Machine) -> Control {
    self::misc::mstore(state)
}

fn eval_mstore8(state: &mut Machine) -> Control {
    self::misc::mstore8(state)
}

fn eval_jump(state: &mut Machine) -> Control {
    self::misc::jump(state)
}

fn eval_jumpi(state: &mut Machine) -> Control {
    self::misc::jumpi(state)
}

fn eval_pc(state: &mut Machine, position: usize) -> Control {
    self::misc::pc(state, position)
}

fn eval_msize(state: &mut Machine) -> Control {
    self::misc::msize(state)
}

fn eval_jumpdest() -> Control {
    Control::Continue
}

fn eval_push1(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 1, position)
}

fn eval_push2(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 2, position)
}

fn eval_push3(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 3, position)
}

fn eval_push4(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 4, position)
}

fn eval_push5(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 5, position)
}

fn eval_push6(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 6, position)
}

fn eval_push7(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 7, position)
}

fn eval_push8(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 8, position)
}

fn eval_push9(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 9, position)
}

fn eval_push10(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 10, position)
}

fn eval_push11(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 11, position)
}

fn eval_push12(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 12, position)
}

fn eval_push13(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 13, position)
}

fn eval_push14(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 14, position)
}

fn eval_push15(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 15, position)
}

fn eval_push16(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 16, position)
}

fn eval_push17(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 17, position)
}

fn eval_push18(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 18, position)
}

fn eval_push19(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 19, position)
}

fn eval_push20(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 20, position)
}

fn eval_push21(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 21, position)
}

fn eval_push22(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 22, position)
}

fn eval_push23(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 23, position)
}

fn eval_push24(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 24, position)
}

fn eval_push25(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 25, position)
}

fn eval_push26(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 26, position)
}

fn eval_push27(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 27, position)
}

fn eval_push28(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 28, position)
}

fn eval_push29(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 29, position)
}

fn eval_push30(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 30, position)
}

fn eval_push31(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 31, position)
}

fn eval_push32(state: &mut Machine, position: usize) -> Control {
    self::misc::push(state, 32, position)
}

fn eval_dup1(state: &mut Machine) -> Control {
    self::misc::dup(state, 1)
}

fn eval_dup2(state: &mut Machine) -> Control {
    self::misc::dup(state, 2)
}

fn eval_dup3(state: &mut Machine) -> Control {
    self::misc::dup(state, 3)
}

fn eval_dup4(state: &mut Machine) -> Control {
    self::misc::dup(state, 4)
}

fn eval_dup5(state: &mut Machine) -> Control {
    self::misc::dup(state, 5)
}

fn eval_dup6(state: &mut Machine) -> Control {
    self::misc::dup(state, 6)
}

fn eval_dup7(state: &mut Machine) -> Control {
    self::misc::dup(state, 7)
}

fn eval_dup8(state: &mut Machine) -> Control {
    self::misc::dup(state, 8)
}

fn eval_dup9(state: &mut Machine) -> Control {
    self::misc::dup(state, 9)
}

fn eval_dup10(state: &mut Machine) -> Control {
    self::misc::dup(state, 10)
}

fn eval_dup11(state: &mut Machine) -> Control {
    self::misc::dup(state, 11)
}

fn eval_dup12(state: &mut Machine) -> Control {
    self::misc::dup(state, 12)
}

fn eval_dup13(state: &mut Machine) -> Control {
    self::misc::dup(state, 13)
}

fn eval_dup14(state: &mut Machine) -> Control {
    self::misc::dup(state, 14)
}

fn eval_dup15(state: &mut Machine) -> Control {
    self::misc::dup(state, 15)
}

fn eval_dup16(state: &mut Machine) -> Control {
    self::misc::dup(state, 16)
}

fn eval_swap1(state: &mut Machine) -> Control {
    self::misc::swap(state, 1)
}

fn eval_swap2(state: &mut Machine) -> Control {
    self::misc::swap(state, 2)
}

fn eval_swap3(state: &mut Machine) -> Control {
    self::misc::swap(state, 3)
}

fn eval_swap4(state: &mut Machine) -> Control {
    self::misc::swap(state, 4)
}

fn eval_swap5(state: &mut Machine) -> Control {
    self::misc::swap(state, 5)
}

fn eval_swap6(state: &mut Machine) -> Control {
    self::misc::swap(state, 6)
}

fn eval_swap7(state: &mut Machine) -> Control {
    self::misc::swap(state, 7)
}

fn eval_swap8(state: &mut Machine) -> Control {
    self::misc::swap(state, 8)
}

fn eval_swap9(state: &mut Machine) -> Control {
    self::misc::swap(state, 9)
}

fn eval_swap10(state: &mut Machine) -> Control {
    self::misc::swap(state, 10)
}

fn eval_swap11(state: &mut Machine) -> Control {
    self::misc::swap(state, 11)
}

fn eval_swap12(state: &mut Machine) -> Control {
    self::misc::swap(state, 12)
}

fn eval_swap13(state: &mut Machine) -> Control {
    self::misc::swap(state, 13)
}

fn eval_swap14(state: &mut Machine) -> Control {
    self::misc::swap(state, 14)
}

fn eval_swap15(state: &mut Machine) -> Control {
    self::misc::swap(state, 15)
}

fn eval_swap16(state: &mut Machine) -> Control {
    self::misc::swap(state, 16)
}

fn eval_return(state: &mut Machine) -> Control {
    self::misc::ret(state)
}

fn eval_revert(state: &mut Machine) -> Control {
    self::misc::revert(state)
}

fn eval_invalid() -> Control {
    Control::Exit(ExitError::DesignatedInvalid.into())
}

#[inline]
pub fn eval<H: ExtHandler, SPEC: Spec, const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
    state: &mut Machine,
    opcode: OpCode,
    position: usize,
    handler: &mut H,
) -> Control {
    
    match opcode {
        OpCode::STOP => Control::Exit(ExitSucceed::Stopped.into()),
        OpCode::ADD => eval_add(state),
        OpCode::MUL => eval_mul(state),
        OpCode::SUB => eval_sub(state),
        OpCode::DIV => eval_div(state),
        OpCode::SDIV => eval_sdiv(state),
        OpCode::MOD => eval_mod(state),
        OpCode::SMOD => eval_smod(state),
        OpCode::ADDMOD => eval_addmod(state),
        OpCode::MULMOD => eval_mulmod(state),
        OpCode::EXP => eval_exp(state),
        OpCode::SIGNEXTEND => eval_signextend(state),
        OpCode::LT => eval_lt(state),
        OpCode::GT => eval_gt(state),
        OpCode::SLT => eval_slt(state),
        OpCode::SGT => eval_sgt(state),
        OpCode::EQ => eval_eq(state),
        OpCode::ISZERO => eval_iszero(state),
        OpCode::AND => eval_and(state),
        OpCode::OR => eval_or(state),
        OpCode::XOR => eval_xor(state),
        OpCode::NOT => eval_not(state),
        OpCode::BYTE => eval_byte(state),
        OpCode::SHL => eval_shl(state),
        OpCode::SHR => eval_shr(state),
        OpCode::SAR => eval_sar(state),
        OpCode::CODESIZE => eval_codesize(state),
        OpCode::CODECOPY => eval_codecopy(state),
        OpCode::CALLDATALOAD => eval_calldataload(state),
        OpCode::CALLDATASIZE => eval_calldatasize(state),
        OpCode::CALLDATACOPY => eval_calldatacopy(state),
        OpCode::POP => eval_pop(state),
        OpCode::MLOAD => eval_mload(state),
        OpCode::MSTORE => eval_mstore(state),
        OpCode::MSTORE8 => eval_mstore8(state),
        OpCode::JUMP => eval_jump(state),
        OpCode::JUMPI => eval_jumpi(state),
        OpCode::PC => eval_pc(state, position),
        OpCode::MSIZE => eval_msize(state),
        OpCode::JUMPDEST => eval_jumpdest(),

        OpCode::PUSH1 => eval_push1(state, position),
        OpCode::PUSH2 => eval_push2(state, position),
        OpCode::PUSH3 => eval_push3(state, position),
        OpCode::PUSH4 => eval_push4(state, position),
        OpCode::PUSH5 => eval_push5(state, position),
        OpCode::PUSH6 => eval_push6(state, position),
        OpCode::PUSH7 => eval_push7(state, position),
        OpCode::PUSH8 => eval_push8(state, position),
        OpCode::PUSH9 => eval_push9(state, position),
        OpCode::PUSH10 => eval_push10(state, position),
        OpCode::PUSH11 => eval_push11(state, position),
        OpCode::PUSH12 => eval_push12(state, position),
        OpCode::PUSH13 => eval_push13(state, position),
        OpCode::PUSH14 => eval_push14(state, position),
        OpCode::PUSH15 => eval_push15(state, position),
        OpCode::PUSH16 => eval_push16(state, position),
        OpCode::PUSH17 => eval_push17(state, position),
        OpCode::PUSH18 => eval_push18(state, position),
        OpCode::PUSH19 => eval_push19(state, position),
        OpCode::PUSH20 => eval_push20(state, position),
        OpCode::PUSH21 => eval_push21(state, position),
        OpCode::PUSH22 => eval_push22(state, position),
        OpCode::PUSH23 => eval_push23(state, position),
        OpCode::PUSH24 => eval_push24(state, position),
        OpCode::PUSH25 => eval_push25(state, position),
        OpCode::PUSH26 => eval_push26(state, position),
        OpCode::PUSH27 => eval_push27(state, position),
        OpCode::PUSH28 => eval_push28(state, position),
        OpCode::PUSH29 => eval_push29(state, position),
        OpCode::PUSH30 => eval_push30(state, position),
        OpCode::PUSH31 => eval_push31(state, position),
        OpCode::PUSH32 => eval_push32(state, position),

        OpCode::DUP1 => eval_dup1(state),
        OpCode::DUP2 => eval_dup2(state),
        OpCode::DUP3 => eval_dup3(state),
        OpCode::DUP4 => eval_dup4(state),
        OpCode::DUP5 => eval_dup5(state),
        OpCode::DUP6 => eval_dup6(state),
        OpCode::DUP7 => eval_dup7(state),
        OpCode::DUP8 => eval_dup8(state),
        OpCode::DUP9 => eval_dup9(state),
        OpCode::DUP10 => eval_dup10(state),
        OpCode::DUP11 => eval_dup11(state),
        OpCode::DUP12 => eval_dup12(state),
        OpCode::DUP13 => eval_dup13(state),
        OpCode::DUP14 => eval_dup14(state),
        OpCode::DUP15 => eval_dup15(state),
        OpCode::DUP16 => eval_dup16(state),

        OpCode::SWAP1 => eval_swap1(state),
        OpCode::SWAP2 => eval_swap2(state),
        OpCode::SWAP3 => eval_swap3(state),
        OpCode::SWAP4 => eval_swap4(state),
        OpCode::SWAP5 => eval_swap5(state),
        OpCode::SWAP6 => eval_swap6(state),
        OpCode::SWAP7 => eval_swap7(state),
        OpCode::SWAP8 => eval_swap8(state),
        OpCode::SWAP9 => eval_swap9(state),
        OpCode::SWAP10 => eval_swap10(state),
        OpCode::SWAP11 => eval_swap11(state),
        OpCode::SWAP12 => eval_swap12(state),
        OpCode::SWAP13 => eval_swap13(state),
        OpCode::SWAP14 => eval_swap14(state),
        OpCode::SWAP15 => eval_swap15(state),
        OpCode::SWAP16 => eval_swap16(state),

        OpCode::RETURN => eval_return(state),
        OpCode::REVERT => eval_revert(state),
        OpCode::INVALID => eval_invalid(),
        OpCode::SHA3 => system::sha3(state),
        OpCode::ADDRESS => system::address(state),
        OpCode::BALANCE => system::balance(state, handler),
        OpCode::SELFBALANCE => system::selfbalance(state, handler),
        OpCode::ORIGIN => system::origin(state, handler),
        OpCode::CALLER => system::caller(state),
        OpCode::CALLVALUE => system::callvalue(state),
        OpCode::GASPRICE => system::gasprice(state, handler),
        OpCode::EXTCODESIZE => system::extcodesize(state, handler),
        OpCode::EXTCODEHASH => system::extcodehash(state, handler),
        OpCode::EXTCODECOPY => system::extcodecopy(state, handler),
        OpCode::RETURNDATASIZE => system::returndatasize(state),
        OpCode::RETURNDATACOPY => system::returndatacopy(state),
        OpCode::BLOCKHASH => system::blockhash(state, handler),
        OpCode::COINBASE => system::coinbase(state, handler),
        OpCode::TIMESTAMP => system::timestamp(state, handler),
        OpCode::NUMBER => system::number(state, handler),
        OpCode::DIFFICULTY => system::difficulty(state, handler),
        OpCode::GASLIMIT => system::gaslimit(state, handler),
        OpCode::SLOAD => system::sload::<H, OPCODE_TRACE>(state, handler),
        OpCode::SSTORE => system::sstore::<H, OPCODE_TRACE>(state, handler),
        OpCode::GAS => system::gas(state, handler),
        OpCode::LOG0 => system::log(state, 0, handler),
        OpCode::LOG1 => system::log(state, 1, handler),
        OpCode::LOG2 => system::log(state, 2, handler),
        OpCode::LOG3 => system::log(state, 3, handler),
        OpCode::LOG4 => system::log(state, 4, handler),
        OpCode::SUICIDE => system::suicide::<H, CALL_TRACE>(state, handler),
        OpCode::CREATE => {
            system::create::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(state, false, handler)
        }
        OpCode::CREATE2 => {
            system::create::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(state, true, handler)
        }
        OpCode::CALL => {
            system::call::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(state, CallScheme::Call, handler)
        }
        OpCode::CALLCODE => system::call::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(
            state,
            CallScheme::CallCode,
            handler,
        ),
        OpCode::DELEGATECALL => system::call::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(
            state,
            CallScheme::DelegateCall,
            handler,
        ),
        OpCode::STATICCALL => system::call::<H, CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(
            state,
            CallScheme::StaticCall,
            handler,
        ),
        OpCode::CHAINID => system::chainid(state, handler),
        //_ => Control::Exit(ExitReason::Fatal(ExitFatal::NotSupported)),
    }
}
