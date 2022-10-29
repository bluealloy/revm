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
use ruint::aliases::U256;

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
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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
    FatalExternalError,
    GasMaxFeeGreaterThanPriorityFee,
    GasPriceLessThenBasefee,
    CallerGasLimitMoreThenBlock,
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    LackOfFundForGasLimit,
    CreateCollision,
    OverflowPayment,
    PrecompileError,
    NonceOverflow,
    /// Create init code exceeds limit (runtime).
    CreateContractLimit,
    /// Error on created contract that begins with EF
    CreateContractWithEF,
}

#[inline(always)]
pub fn eval<H: Host, S: Spec>(opcode: u8, interp: &mut Interpreter, host: &mut H) -> Return {
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
        opcode::ADD => op2_u256!(interp, wrapping_add),
        opcode::MUL => op2_u256!(interp, wrapping_mul),
        opcode::SUB => op2_u256!(interp, wrapping_sub),
        opcode::DIV => op2_u256_fn!(interp, arithmetic::div),
        opcode::SDIV => op2_u256_fn!(interp, arithmetic::sdiv),
        opcode::MOD => op2_u256_fn!(interp, arithmetic::rem),
        opcode::SMOD => op2_u256_fn!(interp, arithmetic::smod),
        opcode::ADDMOD => op3_u256_fn!(interp, arithmetic::addmod),
        opcode::MULMOD => op3_u256_fn!(interp, arithmetic::mulmod),
        opcode::EXP => arithmetic::eval_exp::<S>(interp),
        opcode::SIGNEXTEND => op2_u256_fn!(interp, arithmetic::signextend),
        opcode::LT => op2_u256_bool_ref!(interp, lt),
        opcode::GT => op2_u256_bool_ref!(interp, gt),
        opcode::SLT => op2_u256_fn!(interp, bitwise::slt),
        opcode::SGT => op2_u256_fn!(interp, bitwise::sgt),
        opcode::EQ => op2_u256_bool_ref!(interp, eq),
        opcode::ISZERO => op1_u256_fn!(interp, bitwise::iszero),
        opcode::AND => op2_u256!(interp, bitand),
        opcode::OR => op2_u256!(interp, bitor),
        opcode::XOR => op2_u256!(interp, bitxor),
        opcode::NOT => op1_u256_fn!(interp, bitwise::not),
        opcode::BYTE => op2_u256_fn!(interp, bitwise::byte),
        opcode::SHL => op2_u256_fn!(
            interp,
            bitwise::shl,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SHR => op2_u256_fn!(
            interp,
            bitwise::shr,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SAR => op2_u256_fn!(
            interp,
            bitwise::sar,
            S::enabled(CONSTANTINOPLE) // EIP-145: Bitwise shifting instructions in EVM
        ),
        opcode::SHA3 => system::sha3(interp),

        opcode::ADDRESS => system::address(interp),
        opcode::BALANCE => host::balance::<H, S>(interp, host),
        opcode::SELFBALANCE => host::selfbalance::<H, S>(interp, host),
        opcode::CODESIZE => system::codesize(interp),
        opcode::CODECOPY => system::codecopy(interp),
        opcode::CALLDATALOAD => system::calldataload(interp),
        opcode::CALLDATASIZE => system::calldatasize(interp),
        opcode::CALLDATACOPY => system::calldatacopy(interp),
        opcode::POP => stack::pop(interp),
        opcode::MLOAD => memory::mload(interp),
        opcode::MSTORE => memory::mstore(interp),
        opcode::MSTORE8 => memory::mstore8(interp),
        opcode::JUMP => control::jump(interp),
        opcode::JUMPI => control::jumpi(interp),
        opcode::PC => control::pc(interp),
        opcode::MSIZE => memory::msize(interp),
        opcode::JUMPDEST => control::jumpdest(interp),
        opcode::PUSH1 => stack::push::<1>(interp),
        opcode::PUSH2 => stack::push::<2>(interp),
        opcode::PUSH3 => stack::push::<3>(interp),
        opcode::PUSH4 => stack::push::<4>(interp),
        opcode::PUSH5 => stack::push::<5>(interp),
        opcode::PUSH6 => stack::push::<6>(interp),
        opcode::PUSH7 => stack::push::<7>(interp),
        opcode::PUSH8 => stack::push::<8>(interp),
        opcode::PUSH9 => stack::push::<9>(interp),
        opcode::PUSH10 => stack::push::<10>(interp),
        opcode::PUSH11 => stack::push::<11>(interp),
        opcode::PUSH12 => stack::push::<12>(interp),
        opcode::PUSH13 => stack::push::<13>(interp),
        opcode::PUSH14 => stack::push::<14>(interp),
        opcode::PUSH15 => stack::push::<15>(interp),
        opcode::PUSH16 => stack::push::<16>(interp),
        opcode::PUSH17 => stack::push::<17>(interp),
        opcode::PUSH18 => stack::push::<18>(interp),
        opcode::PUSH19 => stack::push::<19>(interp),
        opcode::PUSH20 => stack::push::<20>(interp),
        opcode::PUSH21 => stack::push::<21>(interp),
        opcode::PUSH22 => stack::push::<22>(interp),
        opcode::PUSH23 => stack::push::<23>(interp),
        opcode::PUSH24 => stack::push::<24>(interp),
        opcode::PUSH25 => stack::push::<25>(interp),
        opcode::PUSH26 => stack::push::<26>(interp),
        opcode::PUSH27 => stack::push::<27>(interp),
        opcode::PUSH28 => stack::push::<28>(interp),
        opcode::PUSH29 => stack::push::<29>(interp),
        opcode::PUSH30 => stack::push::<30>(interp),
        opcode::PUSH31 => stack::push::<31>(interp),
        opcode::PUSH32 => stack::push::<32>(interp),
        opcode::DUP1 => stack::dup::<1>(interp),
        opcode::DUP2 => stack::dup::<2>(interp),
        opcode::DUP3 => stack::dup::<3>(interp),
        opcode::DUP4 => stack::dup::<4>(interp),
        opcode::DUP5 => stack::dup::<5>(interp),
        opcode::DUP6 => stack::dup::<6>(interp),
        opcode::DUP7 => stack::dup::<7>(interp),
        opcode::DUP8 => stack::dup::<8>(interp),
        opcode::DUP9 => stack::dup::<9>(interp),
        opcode::DUP10 => stack::dup::<10>(interp),
        opcode::DUP11 => stack::dup::<11>(interp),
        opcode::DUP12 => stack::dup::<12>(interp),
        opcode::DUP13 => stack::dup::<13>(interp),
        opcode::DUP14 => stack::dup::<14>(interp),
        opcode::DUP15 => stack::dup::<15>(interp),
        opcode::DUP16 => stack::dup::<16>(interp),

        opcode::SWAP1 => stack::swap::<1>(interp),
        opcode::SWAP2 => stack::swap::<2>(interp),
        opcode::SWAP3 => stack::swap::<3>(interp),
        opcode::SWAP4 => stack::swap::<4>(interp),
        opcode::SWAP5 => stack::swap::<5>(interp),
        opcode::SWAP6 => stack::swap::<6>(interp),
        opcode::SWAP7 => stack::swap::<7>(interp),
        opcode::SWAP8 => stack::swap::<8>(interp),
        opcode::SWAP9 => stack::swap::<9>(interp),
        opcode::SWAP10 => stack::swap::<10>(interp),
        opcode::SWAP11 => stack::swap::<11>(interp),
        opcode::SWAP12 => stack::swap::<12>(interp),
        opcode::SWAP13 => stack::swap::<13>(interp),
        opcode::SWAP14 => stack::swap::<14>(interp),
        opcode::SWAP15 => stack::swap::<15>(interp),
        opcode::SWAP16 => stack::swap::<16>(interp),

        opcode::RETURN => control::ret(interp),
        opcode::REVERT => control::revert::<S>(interp),
        opcode::INVALID => Return::InvalidOpcode,
        opcode::BASEFEE => host_env::basefee::<H, S>(interp, host),
        opcode::ORIGIN => host_env::origin(interp, host),
        opcode::CALLER => system::caller(interp),
        opcode::CALLVALUE => system::callvalue(interp),
        opcode::GASPRICE => host_env::gasprice(interp, host),
        opcode::EXTCODESIZE => host::extcodesize::<H, S>(interp, host),
        opcode::EXTCODEHASH => host::extcodehash::<H, S>(interp, host),
        opcode::EXTCODECOPY => host::extcodecopy::<H, S>(interp, host),
        opcode::RETURNDATASIZE => system::returndatasize::<S>(interp),
        opcode::RETURNDATACOPY => system::returndatacopy::<S>(interp),
        opcode::BLOCKHASH => host::blockhash(interp, host),
        opcode::COINBASE => host_env::coinbase(interp, host),
        opcode::TIMESTAMP => host_env::timestamp(interp, host),
        opcode::NUMBER => host_env::number(interp, host),
        opcode::DIFFICULTY => host_env::difficulty(interp, host),
        opcode::GASLIMIT => host_env::gaslimit(interp, host),
        opcode::SLOAD => host::sload::<H, S>(interp, host),
        opcode::SSTORE => host::sstore::<H, S>(interp, host),
        opcode::GAS => system::gas(interp),
        opcode::LOG0 => host::log::<H, S>(interp, 0, host),
        opcode::LOG1 => host::log::<H, S>(interp, 1, host),
        opcode::LOG2 => host::log::<H, S>(interp, 2, host),
        opcode::LOG3 => host::log::<H, S>(interp, 3, host),
        opcode::LOG4 => host::log::<H, S>(interp, 4, host),
        opcode::SELFDESTRUCT => host::selfdestruct::<H, S>(interp, host),
        opcode::CREATE => host::create::<H, S>(interp, false, host), //check
        opcode::CREATE2 => host::create::<H, S>(interp, true, host), //check
        opcode::CALL => host::call::<H, S>(interp, CallScheme::Call, host), //check
        opcode::CALLCODE => host::call::<H, S>(interp, CallScheme::CallCode, host), //check
        opcode::DELEGATECALL => host::call::<H, S>(interp, CallScheme::DelegateCall, host), //check
        opcode::STATICCALL => host::call::<H, S>(interp, CallScheme::StaticCall, host), //check
        opcode::CHAINID => host_env::chainid::<H, S>(interp, host),
        _ => Return::OpcodeNotFound,
    }
}
