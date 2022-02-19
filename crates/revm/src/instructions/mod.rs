#[macro_use]
mod macros;
mod arithmetic;
mod bitwise;
pub(crate) mod gas;
mod i256;
mod misc;
pub mod opcode;
mod system;

pub use opcode::{OpCode, OPCODE_JUMPMAP};

use crate::{
    interpreter::Machine,
    spec::{Spec, SpecId::*},
    CallScheme, Host,
};
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
pub fn eval<H: Host, S: Spec>(opcode: u8, machine: &mut Machine, host: &mut H) -> Return {
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
        opcode::BALANCE => system::balance::<H, S>(machine, host),
        opcode::SELFBALANCE => system::selfbalance::<H, S>(machine, host),
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
        opcode::PC => misc::pc(machine),
        opcode::MSIZE => misc::msize(machine),
        opcode::JUMPDEST => misc::jumpdest(machine),
        opcode::PUSH1 => misc::push::<1>(machine),
        opcode::PUSH2 => misc::push::<2>(machine),
        opcode::PUSH3 => misc::push::<3>(machine),
        opcode::PUSH4 => misc::push::<4>(machine),
        opcode::PUSH5 => misc::push::<5>(machine),
        opcode::PUSH6 => misc::push::<6>(machine),
        opcode::PUSH7 => misc::push::<7>(machine),
        opcode::PUSH8 => misc::push::<8>(machine),
        opcode::PUSH9 => misc::push::<9>(machine),
        opcode::PUSH10 => misc::push::<10>(machine),
        opcode::PUSH11 => misc::push::<11>(machine),
        opcode::PUSH12 => misc::push::<12>(machine),
        opcode::PUSH13 => misc::push::<13>(machine),
        opcode::PUSH14 => misc::push::<14>(machine),
        opcode::PUSH15 => misc::push::<15>(machine),
        opcode::PUSH16 => misc::push::<16>(machine),
        opcode::PUSH17 => misc::push::<17>(machine),
        opcode::PUSH18 => misc::push::<18>(machine),
        opcode::PUSH19 => misc::push::<19>(machine),
        opcode::PUSH20 => misc::push::<20>(machine),
        opcode::PUSH21 => misc::push::<21>(machine),
        opcode::PUSH22 => misc::push::<22>(machine),
        opcode::PUSH23 => misc::push::<23>(machine),
        opcode::PUSH24 => misc::push::<24>(machine),
        opcode::PUSH25 => misc::push::<25>(machine),
        opcode::PUSH26 => misc::push::<26>(machine),
        opcode::PUSH27 => misc::push::<27>(machine),
        opcode::PUSH28 => misc::push::<28>(machine),
        opcode::PUSH29 => misc::push::<29>(machine),
        opcode::PUSH30 => misc::push::<30>(machine),
        opcode::PUSH31 => misc::push::<31>(machine),
        opcode::PUSH32 => misc::push::<32>(machine),
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
        opcode::INVALID => Return::InvalidOpcode,
        opcode::BASEFEE => system::basefee::<H, S>(machine, host),
        opcode::ORIGIN => system::origin(machine, host),
        opcode::CALLER => system::caller(machine),
        opcode::CALLVALUE => system::callvalue(machine),
        opcode::GASPRICE => system::gasprice(machine, host),
        opcode::EXTCODESIZE => system::extcodesize::<H, S>(machine, host),
        opcode::EXTCODEHASH => system::extcodehash::<H, S>(machine, host),
        opcode::EXTCODECOPY => system::extcodecopy::<H, S>(machine, host),
        opcode::RETURNDATASIZE => system::returndatasize::<S>(machine),
        opcode::RETURNDATACOPY => system::returndatacopy::<S>(machine),
        opcode::BLOCKHASH => system::blockhash(machine, host),
        opcode::COINBASE => system::coinbase(machine, host),
        opcode::TIMESTAMP => system::timestamp(machine, host),
        opcode::NUMBER => system::number(machine, host),
        opcode::DIFFICULTY => system::difficulty(machine, host),
        opcode::GASLIMIT => system::gaslimit(machine, host),
        opcode::SLOAD => system::sload::<H, S>(machine, host),
        opcode::SSTORE => system::sstore::<H, S>(machine, host),
        opcode::GAS => system::gas(machine),
        opcode::LOG0 => system::log::<H, S>(machine, 0, host),
        opcode::LOG1 => system::log::<H, S>(machine, 1, host),
        opcode::LOG2 => system::log::<H, S>(machine, 2, host),
        opcode::LOG3 => system::log::<H, S>(machine, 3, host),
        opcode::LOG4 => system::log::<H, S>(machine, 4, host),
        opcode::SELFDESTRUCT => system::selfdestruct::<H, S>(machine, host),
        opcode::CREATE => system::create::<H, S>(machine, false, host), //check
        opcode::CREATE2 => system::create::<H, S>(machine, true, host), //check
        opcode::CALL => system::call::<H, S>(machine, CallScheme::Call, host), //check
        opcode::CALLCODE => system::call::<H, S>(machine, CallScheme::CallCode, host), //check
        opcode::DELEGATECALL => system::call::<H, S>(machine, CallScheme::DelegateCall, host), //check
        opcode::STATICCALL => system::call::<H, S>(machine, CallScheme::StaticCall, host), //check
        opcode::CHAINID => system::chainid::<H, S>(machine, host),
        _ => Return::OpcodeNotFound,
    }
}
