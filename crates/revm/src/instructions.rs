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

pub use opcode::{OpCode, OPCODE_JUMPMAP};

use crate::{interpreter::Interpreter, CallScheme, Host, Spec, SpecId::*};
use core::ops::{BitAnd, BitOr, BitXor};
use primitive_types::U256;

#[macro_export]
macro_rules! return_ok {
    () => {
        Return::Continue | Return::Stop | Return::Return | Return::SelfDestruct
    };
}

#[macro_export]
macro_rules! return_revert {
    () => {
        Return::Revert | Return::CallTooDeep | Return::OutOfFund
    };
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Return {
    //success codes
    Continue = 0x00,
    Stop = 0x01,
    Return = 0x02,
    SelfDestruct = 0x03,

    // revert code
    Revert = 0x20, // revert opcode
    CallTooDeep = 0x21,
    OutOfFund = 0x22,

    // error codes
    OutOfGas = 0x50,
    OpcodeNotFound,
    CallNotAllowedInsideStatic,
    InvalidOpcode,
    InvalidJump,
    InvalidMemoryRange,
    NotActivated,
    StackUnderflow,
    StackOverflow,
    OutOfOffset,
    FatalNotSupported,
    GasMaxFeeGreaterThanPriorityFee,
    GasPriceLessThenBasefee,
    CallerGasLimitMoreThenBlock,
    RejectCallerWithCode, //new eip included in london
    LackOfFundForGasLimit,
    CreateCollision,
    OverflowPayment,
    Precompile,

    /// Create init code exceeds limit (runtime).
    CreateContractLimit,
    /// Create contract that begins with EF
    CreateContractWithEF,
}

#[inline(always)]
pub fn eval<H: Host, S: Spec>(opcode: u8, machine: &mut Interpreter, host: &mut H) -> Return {
    match opcode {
        /*12_u8..=15_u8 => Return::OpcodeNotFound,
        30_u8..=31_u8 => Return::OpcodeNotFound,
        33_u8..=47_u8 => Return::OpcodeNotFound,
        73_u8..=79_u8 => Return::OpcodeNotFound,
        92_u8..=95_u8 => Return::OpcodeNotFound,
        165_u8..=239_u8 => Return::OpcodeNotFound,
        246_u8..=249_u8 => Return::OpcodeNotFound,
        251_u8..=252_u8 => Return::OpcodeNotFound,*/
        opcode::STOP => Return::Stop,
        opcode::ADD => op2_u256_tuple!(machine, overflowing_add),
        opcode::MUL => op2_u256_tuple!(machine, overflowing_mul),
        opcode::SUB => op2_u256_tuple!(machine, overflowing_sub),
        opcode::DIV => op2_u256_fn!(machine, arithmetic::div),
        opcode::SDIV => op2_u256_fn!(machine, arithmetic::sdiv),
        opcode::MOD => op2_u256_fn!(machine, arithmetic::rem),
        opcode::SMOD => op2_u256_fn!(machine, arithmetic::smod),
        opcode::ADDMOD => op3_u256_fn!(machine, arithmetic::addmod),
        opcode::MULMOD => op3_u256_fn!(machine, arithmetic::mulmod),
        opcode::EXP => arithmetic::eval_exp::<S>(machine),
        opcode::SIGNEXTEND => op2_u256_fn!(machine, arithmetic::signextend),
        opcode::LT => op2_u256_bool_ref!(machine, lt),
        opcode::GT => op2_u256_bool_ref!(machine, gt),
        opcode::SLT => op2_u256_fn!(machine, bitwise::slt),
        opcode::SGT => op2_u256_fn!(machine, bitwise::sgt),
        opcode::EQ => op2_u256_bool_ref!(machine, eq),
        opcode::ISZERO => op1_u256_fn!(machine, bitwise::iszero),
        opcode::AND => op2_u256!(machine, bitand),
        opcode::OR => op2_u256!(machine, bitor),
        opcode::XOR => op2_u256!(machine, bitxor),
        opcode::NOT => op1_u256_fn!(machine, bitwise::not),
        opcode::BYTE => op2_u256_fn!(machine, bitwise::byte),
        opcode::SHL => op2_u256_fn!(
            machine,
            bitwise::shl,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SHR => op2_u256_fn!(
            machine,
            bitwise::shr,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SAR => op2_u256_fn!(
            machine,
            bitwise::sar,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SHA3 => system::sha3(machine),

        opcode::ADDRESS => system::address(machine),
        opcode::BALANCE => host::balance::<H, S>(machine, host),
        opcode::SELFBALANCE => host::selfbalance::<H, S>(machine, host),
        opcode::CODESIZE => system::codesize(machine),
        opcode::CODECOPY => system::codecopy(machine),
        opcode::CALLDATALOAD => system::calldataload(machine),
        opcode::CALLDATASIZE => system::calldatasize(machine),
        opcode::CALLDATACOPY => system::calldatacopy(machine),
        opcode::POP => stack::pop(machine),
        opcode::MLOAD => memory::mload(machine),
        opcode::MSTORE => memory::mstore(machine),
        opcode::MSTORE8 => memory::mstore8(machine),
        opcode::JUMP => control::jump(machine),
        opcode::JUMPI => control::jumpi(machine),
        opcode::PC => control::pc(machine),
        opcode::MSIZE => memory::msize(machine),
        opcode::JUMPDEST => control::jumpdest(machine),
        opcode::PUSH1 => stack::push::<1>(machine),
        opcode::PUSH2 => stack::push::<2>(machine),
        opcode::PUSH3 => stack::push::<3>(machine),
        opcode::PUSH4 => stack::push::<4>(machine),
        opcode::PUSH5 => stack::push::<5>(machine),
        opcode::PUSH6 => stack::push::<6>(machine),
        opcode::PUSH7 => stack::push::<7>(machine),
        opcode::PUSH8 => stack::push::<8>(machine),
        opcode::PUSH9 => stack::push::<9>(machine),
        opcode::PUSH10 => stack::push::<10>(machine),
        opcode::PUSH11 => stack::push::<11>(machine),
        opcode::PUSH12 => stack::push::<12>(machine),
        opcode::PUSH13 => stack::push::<13>(machine),
        opcode::PUSH14 => stack::push::<14>(machine),
        opcode::PUSH15 => stack::push::<15>(machine),
        opcode::PUSH16 => stack::push::<16>(machine),
        opcode::PUSH17 => stack::push::<17>(machine),
        opcode::PUSH18 => stack::push::<18>(machine),
        opcode::PUSH19 => stack::push::<19>(machine),
        opcode::PUSH20 => stack::push::<20>(machine),
        opcode::PUSH21 => stack::push::<21>(machine),
        opcode::PUSH22 => stack::push::<22>(machine),
        opcode::PUSH23 => stack::push::<23>(machine),
        opcode::PUSH24 => stack::push::<24>(machine),
        opcode::PUSH25 => stack::push::<25>(machine),
        opcode::PUSH26 => stack::push::<26>(machine),
        opcode::PUSH27 => stack::push::<27>(machine),
        opcode::PUSH28 => stack::push::<28>(machine),
        opcode::PUSH29 => stack::push::<29>(machine),
        opcode::PUSH30 => stack::push::<30>(machine),
        opcode::PUSH31 => stack::push::<31>(machine),
        opcode::PUSH32 => stack::push::<32>(machine),
        opcode::DUP1 => stack::dup::<1>(machine),
        opcode::DUP2 => stack::dup::<2>(machine),
        opcode::DUP3 => stack::dup::<3>(machine),
        opcode::DUP4 => stack::dup::<4>(machine),
        opcode::DUP5 => stack::dup::<5>(machine),
        opcode::DUP6 => stack::dup::<6>(machine),
        opcode::DUP7 => stack::dup::<7>(machine),
        opcode::DUP8 => stack::dup::<8>(machine),
        opcode::DUP9 => stack::dup::<9>(machine),
        opcode::DUP10 => stack::dup::<10>(machine),
        opcode::DUP11 => stack::dup::<11>(machine),
        opcode::DUP12 => stack::dup::<12>(machine),
        opcode::DUP13 => stack::dup::<13>(machine),
        opcode::DUP14 => stack::dup::<14>(machine),
        opcode::DUP15 => stack::dup::<15>(machine),
        opcode::DUP16 => stack::dup::<16>(machine),

        opcode::SWAP1 => stack::swap::<1>(machine),
        opcode::SWAP2 => stack::swap::<2>(machine),
        opcode::SWAP3 => stack::swap::<3>(machine),
        opcode::SWAP4 => stack::swap::<4>(machine),
        opcode::SWAP5 => stack::swap::<5>(machine),
        opcode::SWAP6 => stack::swap::<6>(machine),
        opcode::SWAP7 => stack::swap::<7>(machine),
        opcode::SWAP8 => stack::swap::<8>(machine),
        opcode::SWAP9 => stack::swap::<9>(machine),
        opcode::SWAP10 => stack::swap::<10>(machine),
        opcode::SWAP11 => stack::swap::<11>(machine),
        opcode::SWAP12 => stack::swap::<12>(machine),
        opcode::SWAP13 => stack::swap::<13>(machine),
        opcode::SWAP14 => stack::swap::<14>(machine),
        opcode::SWAP15 => stack::swap::<15>(machine),
        opcode::SWAP16 => stack::swap::<16>(machine),

        opcode::RETURN => control::ret(machine),
        opcode::REVERT => control::revert::<S>(machine),
        opcode::INVALID => Return::InvalidOpcode,
        opcode::BASEFEE => host_env::basefee::<H, S>(machine, host),
        opcode::ORIGIN => host_env::origin(machine, host),
        opcode::CALLER => system::caller(machine),
        opcode::CALLVALUE => system::callvalue(machine),
        opcode::GASPRICE => host_env::gasprice(machine, host),
        opcode::EXTCODESIZE => host::extcodesize::<H, S>(machine, host),
        opcode::EXTCODEHASH => host::extcodehash::<H, S>(machine, host),
        opcode::EXTCODECOPY => host::extcodecopy::<H, S>(machine, host),
        opcode::RETURNDATASIZE => system::returndatasize::<S>(machine),
        opcode::RETURNDATACOPY => system::returndatacopy::<S>(machine),
        opcode::BLOCKHASH => host::blockhash(machine, host),
        opcode::COINBASE => host_env::coinbase(machine, host),
        opcode::TIMESTAMP => host_env::timestamp(machine, host),
        opcode::NUMBER => host_env::number(machine, host),
        opcode::DIFFICULTY => host_env::difficulty(machine, host),
        opcode::GASLIMIT => host_env::gaslimit(machine, host),
        opcode::SLOAD => host::sload::<H, S>(machine, host),
        opcode::SSTORE => host::sstore::<H, S>(machine, host),
        opcode::GAS => system::gas(machine),
        opcode::LOG0 => host::log::<H, S>(machine, 0, host),
        opcode::LOG1 => host::log::<H, S>(machine, 1, host),
        opcode::LOG2 => host::log::<H, S>(machine, 2, host),
        opcode::LOG3 => host::log::<H, S>(machine, 3, host),
        opcode::LOG4 => host::log::<H, S>(machine, 4, host),
        opcode::SELFDESTRUCT => host::selfdestruct::<H, S>(machine, host),
        opcode::CREATE => host::create::<H, S>(machine, false, host), //check
        opcode::CREATE2 => host::create::<H, S>(machine, true, host), //check
        opcode::CALL => host::call::<H, S>(machine, CallScheme::Call, host), //check
        opcode::CALLCODE => host::call::<H, S>(machine, CallScheme::CallCode, host), //check
        opcode::DELEGATECALL => host::call::<H, S>(machine, CallScheme::DelegateCall, host), //check
        opcode::STATICCALL => host::call::<H, S>(machine, CallScheme::StaticCall, host), //check
        opcode::CHAINID => host_env::chainid::<H, S>(machine, host),
        _ => Return::OpcodeNotFound,
    }
}
