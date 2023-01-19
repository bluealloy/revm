#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
mod control;
mod host;
mod host_env;
mod i256;
mod memory;
pub mod opcode;
mod stack;
mod system;

use crate::{interpreter::Interpreter, primitives::Spec, Host};
pub use opcode::{OpCode, OPCODE_JUMPMAP};

pub use crate::{return_ok, return_revert, InstructionResult};
pub fn return_stop(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::Stop;
}
pub fn return_invalid(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::InvalidFEOpcode;
}

pub fn return_not_found(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::OpcodeNotFound;
}

#[inline(always)]
pub fn eval<H: Host, S: Spec>(opcode: u8, interp: &mut Interpreter, host: &mut H) {
    match opcode {
        opcode::STOP => return_stop(interp, host),
        opcode::ADD => arithmetic::wrapped_add(interp, host),
        opcode::MUL => arithmetic::wrapping_mul(interp, host),
        opcode::SUB => arithmetic::wrapping_sub(interp, host),
        opcode::DIV => arithmetic::div(interp, host),
        opcode::SDIV => arithmetic::sdiv(interp, host),
        opcode::MOD => arithmetic::rem(interp, host),
        opcode::SMOD => arithmetic::smod(interp, host),
        opcode::ADDMOD => arithmetic::addmod(interp, host),
        opcode::MULMOD => arithmetic::mulmod(interp, host),
        opcode::EXP => arithmetic::eval_exp::<S>(interp, host),
        opcode::SIGNEXTEND => arithmetic::signextend(interp, host),
        opcode::LT => bitwise::lt(interp, host),
        opcode::GT => bitwise::gt(interp, host),
        opcode::SLT => bitwise::slt(interp, host),
        opcode::SGT => bitwise::sgt(interp, host),
        opcode::EQ => bitwise::eq(interp, host),
        opcode::ISZERO => bitwise::iszero(interp, host),
        opcode::AND => bitwise::bitand(interp, host),
        opcode::OR => bitwise::bitor(interp, host),
        opcode::XOR => bitwise::bitxor(interp, host),
        opcode::NOT => bitwise::not(interp, host),
        opcode::BYTE => bitwise::byte(interp, host),
        opcode::SHL => bitwise::shl::<S>(interp, host),
        opcode::SHR => bitwise::shr::<S>(interp, host),
        opcode::SAR => bitwise::sar::<S>(interp, host),
        opcode::SHA3 => system::sha3(interp, host),
        opcode::ADDRESS => system::address(interp, host),
        opcode::BALANCE => host::balance::<S>(interp, host),
        opcode::SELFBALANCE => host::selfbalance::<S>(interp, host),
        opcode::CODESIZE => system::codesize(interp, host),
        opcode::CODECOPY => system::codecopy(interp, host),
        opcode::CALLDATALOAD => system::calldataload(interp, host),
        opcode::CALLDATASIZE => system::calldatasize(interp, host),
        opcode::CALLDATACOPY => system::calldatacopy(interp, host),
        opcode::POP => stack::pop(interp, host),
        opcode::MLOAD => memory::mload(interp, host),
        opcode::MSTORE => memory::mstore(interp, host),
        opcode::MSTORE8 => memory::mstore8(interp, host),
        opcode::JUMP => control::jump(interp, host),
        opcode::JUMPI => control::jumpi(interp, host),
        opcode::PC => control::pc(interp, host),
        opcode::MSIZE => memory::msize(interp, host),
        opcode::JUMPDEST => control::jumpdest(interp, host),
        opcode::PUSH1 => stack::push::<1>(interp, host),
        opcode::PUSH2 => stack::push::<2>(interp, host),
        opcode::PUSH3 => stack::push::<3>(interp, host),
        opcode::PUSH4 => stack::push::<4>(interp, host),
        opcode::PUSH5 => stack::push::<5>(interp, host),
        opcode::PUSH6 => stack::push::<6>(interp, host),
        opcode::PUSH7 => stack::push::<7>(interp, host),
        opcode::PUSH8 => stack::push::<8>(interp, host),
        opcode::PUSH9 => stack::push::<9>(interp, host),
        opcode::PUSH10 => stack::push::<10>(interp, host),
        opcode::PUSH11 => stack::push::<11>(interp, host),
        opcode::PUSH12 => stack::push::<12>(interp, host),
        opcode::PUSH13 => stack::push::<13>(interp, host),
        opcode::PUSH14 => stack::push::<14>(interp, host),
        opcode::PUSH15 => stack::push::<15>(interp, host),
        opcode::PUSH16 => stack::push::<16>(interp, host),
        opcode::PUSH17 => stack::push::<17>(interp, host),
        opcode::PUSH18 => stack::push::<18>(interp, host),
        opcode::PUSH19 => stack::push::<19>(interp, host),
        opcode::PUSH20 => stack::push::<20>(interp, host),
        opcode::PUSH21 => stack::push::<21>(interp, host),
        opcode::PUSH22 => stack::push::<22>(interp, host),
        opcode::PUSH23 => stack::push::<23>(interp, host),
        opcode::PUSH24 => stack::push::<24>(interp, host),
        opcode::PUSH25 => stack::push::<25>(interp, host),
        opcode::PUSH26 => stack::push::<26>(interp, host),
        opcode::PUSH27 => stack::push::<27>(interp, host),
        opcode::PUSH28 => stack::push::<28>(interp, host),
        opcode::PUSH29 => stack::push::<29>(interp, host),
        opcode::PUSH30 => stack::push::<30>(interp, host),
        opcode::PUSH31 => stack::push::<31>(interp, host),
        opcode::PUSH32 => stack::push::<32>(interp, host),
        opcode::DUP1 => stack::dup::<1>(interp, host),
        opcode::DUP2 => stack::dup::<2>(interp, host),
        opcode::DUP3 => stack::dup::<3>(interp, host),
        opcode::DUP4 => stack::dup::<4>(interp, host),
        opcode::DUP5 => stack::dup::<5>(interp, host),
        opcode::DUP6 => stack::dup::<6>(interp, host),
        opcode::DUP7 => stack::dup::<7>(interp, host),
        opcode::DUP8 => stack::dup::<8>(interp, host),
        opcode::DUP9 => stack::dup::<9>(interp, host),
        opcode::DUP10 => stack::dup::<10>(interp, host),
        opcode::DUP11 => stack::dup::<11>(interp, host),
        opcode::DUP12 => stack::dup::<12>(interp, host),
        opcode::DUP13 => stack::dup::<13>(interp, host),
        opcode::DUP14 => stack::dup::<14>(interp, host),
        opcode::DUP15 => stack::dup::<15>(interp, host),
        opcode::DUP16 => stack::dup::<16>(interp, host),

        opcode::SWAP1 => stack::swap::<1>(interp, host),
        opcode::SWAP2 => stack::swap::<2>(interp, host),
        opcode::SWAP3 => stack::swap::<3>(interp, host),
        opcode::SWAP4 => stack::swap::<4>(interp, host),
        opcode::SWAP5 => stack::swap::<5>(interp, host),
        opcode::SWAP6 => stack::swap::<6>(interp, host),
        opcode::SWAP7 => stack::swap::<7>(interp, host),
        opcode::SWAP8 => stack::swap::<8>(interp, host),
        opcode::SWAP9 => stack::swap::<9>(interp, host),
        opcode::SWAP10 => stack::swap::<10>(interp, host),
        opcode::SWAP11 => stack::swap::<11>(interp, host),
        opcode::SWAP12 => stack::swap::<12>(interp, host),
        opcode::SWAP13 => stack::swap::<13>(interp, host),
        opcode::SWAP14 => stack::swap::<14>(interp, host),
        opcode::SWAP15 => stack::swap::<15>(interp, host),
        opcode::SWAP16 => stack::swap::<16>(interp, host),

        opcode::RETURN => control::ret(interp, host),
        opcode::REVERT => control::revert::<S>(interp, host),
        opcode::INVALID => return_invalid(interp, host),
        opcode::BASEFEE => host_env::basefee::<S>(interp, host),
        opcode::ORIGIN => host_env::origin(interp, host),
        opcode::CALLER => system::caller(interp, host),
        opcode::CALLVALUE => system::callvalue(interp, host),
        opcode::GASPRICE => host_env::gasprice(interp, host),
        opcode::EXTCODESIZE => host::extcodesize::<S>(interp, host),
        opcode::EXTCODEHASH => host::extcodehash::<S>(interp, host),
        opcode::EXTCODECOPY => host::extcodecopy::<S>(interp, host),
        opcode::RETURNDATASIZE => system::returndatasize::<S>(interp, host),
        opcode::RETURNDATACOPY => system::returndatacopy::<S>(interp, host),
        opcode::BLOCKHASH => host::blockhash(interp, host),
        opcode::COINBASE => host_env::coinbase(interp, host),
        opcode::TIMESTAMP => host_env::timestamp(interp, host),
        opcode::NUMBER => host_env::number(interp, host),
        opcode::DIFFICULTY => host_env::difficulty::<H, S>(interp, host),
        opcode::GASLIMIT => host_env::gaslimit(interp, host),
        opcode::SLOAD => host::sload::<S>(interp, host),
        opcode::SSTORE => host::sstore::<S>(interp, host),
        opcode::GAS => system::gas(interp, host),
        opcode::LOG0 => host::log::<0, S>(interp, host),
        opcode::LOG1 => host::log::<1, S>(interp, host),
        opcode::LOG2 => host::log::<2, S>(interp, host),
        opcode::LOG3 => host::log::<3, S>(interp, host),
        opcode::LOG4 => host::log::<4, S>(interp, host),
        opcode::SELFDESTRUCT => host::selfdestruct::<S>(interp, host),
        opcode::CREATE => host::create::<false, S>(interp, host), //check
        opcode::CREATE2 => host::create::<true, S>(interp, host), //check
        opcode::CALL => host::call::<S>(interp, host),            //check
        opcode::CALLCODE => host::call_code::<S>(interp, host),   //check
        opcode::DELEGATECALL => host::delegate_call::<S>(interp, host), //check
        opcode::STATICCALL => host::static_call::<S>(interp, host), //check
        opcode::CHAINID => host_env::chainid::<S>(interp, host),
        _ => return_not_found(interp, host),
    }
}
