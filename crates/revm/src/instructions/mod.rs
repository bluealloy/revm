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
    // let time = std::time::Instant::now();
    let out = match opcode {
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
        opcode::PUSH1 => misc::push::<1>(machine, position),
        opcode::PUSH2 => misc::push::<2>(machine, position),
        opcode::PUSH3 => misc::push::<3>(machine, position),
        opcode::PUSH4 => misc::push::<4>(machine, position),
        opcode::PUSH5 => misc::push::<5>(machine, position),
        opcode::PUSH6 => misc::push::<6>(machine, position),
        opcode::PUSH7 => misc::push::<7>(machine, position),
        opcode::PUSH8 => misc::push::<8>(machine, position),
        opcode::PUSH9 => misc::push::<9>(machine, position),
        opcode::PUSH10 => misc::push::<10>(machine, position),
        opcode::PUSH11 => misc::push::<11>(machine, position),
        opcode::PUSH12 => misc::push::<12>(machine, position),
        opcode::PUSH13 => misc::push::<13>(machine, position),
        opcode::PUSH14 => misc::push::<14>(machine, position),
        opcode::PUSH15 => misc::push::<15>(machine, position),
        opcode::PUSH16 => misc::push::<16>(machine, position),
        opcode::PUSH17 => misc::push::<17>(machine, position),
        opcode::PUSH18 => misc::push::<18>(machine, position),
        opcode::PUSH19 => misc::push::<19>(machine, position),
        opcode::PUSH20 => misc::push::<20>(machine, position),
        opcode::PUSH21 => misc::push::<21>(machine, position),
        opcode::PUSH22 => misc::push::<22>(machine, position),
        opcode::PUSH23 => misc::push::<23>(machine, position),
        opcode::PUSH24 => misc::push::<24>(machine, position),
        opcode::PUSH25 => misc::push::<25>(machine, position),
        opcode::PUSH26 => misc::push::<26>(machine, position),
        opcode::PUSH27 => misc::push::<27>(machine, position),
        opcode::PUSH28 => misc::push::<28>(machine, position),
        opcode::PUSH29 => misc::push::<29>(machine, position),
        opcode::PUSH30 => misc::push::<30>(machine, position),
        opcode::PUSH31 => misc::push::<31>(machine, position),
        opcode::PUSH32 => misc::push::<32>(machine, position),
        opcode::DUP1 => misc::dup::<1>(machine),
        opcode::DUP2 => misc::dup::<2>(machine),
        opcode::DUP3 => misc::dup::<3>(machine),
        opcode::DUP4 => misc::dup::<4>(machine),
        opcode::DUP5 => misc::dup::<5>(machine),
        opcode::DUP6 => misc::dup::<6>(machine),
        opcode::DUP7 => misc::dup::<7>(machine),
        opcode::DUP8 => misc::dup::<8>(machine),
        opcode::DUP9 => misc::dup::<9>(machine),
        opcode::DUP10 => misc::dup::<10>(machine),
        opcode::DUP11 => misc::dup::<11>(machine),
        opcode::DUP12 => misc::dup::<12>(machine),
        opcode::DUP13 => misc::dup::<13>(machine),
        opcode::DUP14 => misc::dup::<14>(machine),
        opcode::DUP15 => misc::dup::<15>(machine),
        opcode::DUP16 => misc::dup::<16>(machine),

        opcode::SWAP1 => misc::swap::<1>(machine),
        opcode::SWAP2 => misc::swap::<2>(machine),
        opcode::SWAP3 => misc::swap::<3>(machine),
        opcode::SWAP4 => misc::swap::<4>(machine),
        opcode::SWAP5 => misc::swap::<5>(machine),
        opcode::SWAP6 => misc::swap::<6>(machine),
        opcode::SWAP7 => misc::swap::<7>(machine),
        opcode::SWAP8 => misc::swap::<8>(machine),
        opcode::SWAP9 => misc::swap::<9>(machine),
        opcode::SWAP10 => misc::swap::<10>(machine),
        opcode::SWAP11 => misc::swap::<11>(machine),
        opcode::SWAP12 => misc::swap::<12>(machine),
        opcode::SWAP13 => misc::swap::<13>(machine),
        opcode::SWAP14 => misc::swap::<14>(machine),
        opcode::SWAP15 => misc::swap::<15>(machine),
        opcode::SWAP16 => misc::swap::<16>(machine),

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
    };
    // let times = &mut machine.times[opcode as usize];
    // times.0 += time.elapsed();
    // times.1 += 1;

    out
}
