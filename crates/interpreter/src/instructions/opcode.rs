//! EVM opcode definitions and utilities.

use super::*;
use crate::{
    gas,
    primitives::{Spec, SpecId},
    Host, Interpreter,
};
use core::fmt;

pub type Instruction = fn(&mut Interpreter, &mut dyn Host);

macro_rules! opcodes {
    ($($val:literal => $name:ident => $f:expr),* $(,)?) => {
        // Constants for each opcode. This also takes care of duplicate names.
        $(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($name),"\") opcode.")]
            pub const $name: u8 = $val;
        )*

        /// Maps each opcode to its name.
        pub const OPCODE_JUMPMAP: [Option<&'static str>; 256] = {
            let mut map = [None; 256];
            let mut prev: u8 = 0;
            $(
                let val: u8 = $val;
                assert!(val == 0 || val > prev, "opcodes must be sorted in ascending order");
                prev = val;
                map[$val] = Some(stringify!($name));
            )*
            let _ = prev;
            map
        };

        // Requires `inline_const` and `const_mut_refs` unstable features,
        // but provides ~+2% extra performance.
        // See: https://github.com/bluealloy/revm/issues/310#issuecomment-1664381513
        /*
        type InstructionTable = [Instruction; 256];

        const fn make_instruction_table<SPEC: Spec>() -> InstructionTable {
            let mut table: InstructionTable = [control::not_found; 256];
            let mut i = 0usize;
            while i < 256 {
                table[i] = match i as u8 {
                    $($name => $f,)*
                    _ => control::not_found,
                };
                i += 1;
            }
            table
        }

        // in `eval`:
        (const { make_instruction_table::<SPEC>() })[opcode as usize](interpreter, host)
        */

        /// Evaluates the opcode in the given context.
        #[inline(always)]
        pub(crate) fn eval<SPEC: Spec>(opcode: u8, interpreter: &mut Interpreter, host: &mut dyn Host) {
            // See https://github.com/bluealloy/revm/issues/310#issuecomment-1664381513
            // for previous efforts on optimizing this function.
            let f: Instruction = match opcode {
                $($name => $f as Instruction,)*
                _ => control::not_found as Instruction,
            };
            f(interpreter, host);
        }
    };
}

// When adding new opcodes:
// 1. add the opcode to the list below; make sure it's sorted by opcode value
// 2. add its gas info in the `opcode_gas_info` function below
// 3. implement the opcode in the corresponding module;
//    the function signature must be the exact same as the others
opcodes! {
    0x00 => STOP => control::stop,

    0x01 => ADD        => arithmetic::wrapped_add,
    0x02 => MUL        => arithmetic::wrapping_mul,
    0x03 => SUB        => arithmetic::wrapping_sub,
    0x04 => DIV        => arithmetic::div,
    0x05 => SDIV       => arithmetic::sdiv,
    0x06 => MOD        => arithmetic::rem,
    0x07 => SMOD       => arithmetic::smod,
    0x08 => ADDMOD     => arithmetic::addmod,
    0x09 => MULMOD     => arithmetic::mulmod,
    0x0A => EXP        => arithmetic::exp::<SPEC>,
    0x0B => SIGNEXTEND => arithmetic::signextend,
    // 0x0C
    // 0x0D
    // 0x0E
    // 0x0F
    0x10 => LT     => bitwise::lt,
    0x11 => GT     => bitwise::gt,
    0x12 => SLT    => bitwise::slt,
    0x13 => SGT    => bitwise::sgt,
    0x14 => EQ     => bitwise::eq,
    0x15 => ISZERO => bitwise::iszero,
    0x16 => AND    => bitwise::bitand,
    0x17 => OR     => bitwise::bitor,
    0x18 => XOR    => bitwise::bitxor,
    0x19 => NOT    => bitwise::not,
    0x1A => BYTE   => bitwise::byte,
    0x1B => SHL    => bitwise::shl::<SPEC>,
    0x1C => SHR    => bitwise::shr::<SPEC>,
    0x1D => SAR    => bitwise::sar::<SPEC>,
    // 0x1E
    // 0x1F
    0x20 => KECCAK256 => system::keccak256,
    // 0x21
    // 0x22
    // 0x23
    // 0x24
    // 0x25
    // 0x26
    // 0x27
    // 0x28
    // 0x29
    // 0x2A
    // 0x2B
    // 0x2C
    // 0x2D
    // 0x2E
    // 0x2F
    0x30 => ADDRESS   => system::address,
    0x31 => BALANCE   => host::balance::<SPEC>,
    0x32 => ORIGIN    => host_env::origin,
    0x33 => CALLER    => system::caller,
    0x34 => CALLVALUE => system::callvalue,
    0x35 => CALLDATALOAD => system::calldataload,
    0x36 => CALLDATASIZE => system::calldatasize,
    0x37 => CALLDATACOPY => system::calldatacopy,
    0x38 => CODESIZE     => system::codesize,
    0x39 => CODECOPY     => system::codecopy,

    0x3A => GASPRICE       => host_env::gasprice,
    0x3B => EXTCODESIZE    => host::extcodesize::<SPEC>,
    0x3C => EXTCODECOPY    => host::extcodecopy::<SPEC>,
    0x3D => RETURNDATASIZE => system::returndatasize::<SPEC>,
    0x3E => RETURNDATACOPY => system::returndatacopy::<SPEC>,
    0x3F => EXTCODEHASH    => host::extcodehash::<SPEC>,
    0x40 => BLOCKHASH      => host::blockhash,
    0x41 => COINBASE       => host_env::coinbase,
    0x42 => TIMESTAMP      => host_env::timestamp,
    0x43 => NUMBER         => host_env::number,
    0x44 => DIFFICULTY     => host_env::difficulty::<SPEC>,
    0x45 => GASLIMIT       => host_env::gaslimit,
    0x46 => CHAINID        => host_env::chainid::<SPEC>,
    0x47 => SELFBALANCE    => host::selfbalance::<SPEC>,
    0x48 => BASEFEE        => host_env::basefee::<SPEC>,
    0x49 => BLOBHASH       => host_env::blob_hash::<SPEC>,
    // 0x4A
    // 0x4B
    // 0x4C
    // 0x4D
    // 0x4E
    // 0x4F
    0x50 => POP      => stack::pop,
    0x51 => MLOAD    => memory::mload,
    0x52 => MSTORE   => memory::mstore,
    0x53 => MSTORE8  => memory::mstore8,
    0x54 => SLOAD    => host::sload::<SPEC>,
    0x55 => SSTORE   => host::sstore::<SPEC>,
    0x56 => JUMP     => control::jump,
    0x57 => JUMPI    => control::jumpi,
    0x58 => PC       => control::pc,
    0x59 => MSIZE    => memory::msize,
    0x5A => GAS      => system::gas,
    0x5B => JUMPDEST => control::jumpdest,
    0x5C => TLOAD    => host::tload::<SPEC>,
    0x5D => TSTORE   => host::tstore::<SPEC>,
    0x5E => MCOPY    => memory::mcopy::<SPEC>,

    0x5F => PUSH0  => stack::push0::<SPEC>,
    0x60 => PUSH1  => stack::push::<1>,
    0x61 => PUSH2  => stack::push::<2>,
    0x62 => PUSH3  => stack::push::<3>,
    0x63 => PUSH4  => stack::push::<4>,
    0x64 => PUSH5  => stack::push::<5>,
    0x65 => PUSH6  => stack::push::<6>,
    0x66 => PUSH7  => stack::push::<7>,
    0x67 => PUSH8  => stack::push::<8>,
    0x68 => PUSH9  => stack::push::<9>,
    0x69 => PUSH10 => stack::push::<10>,
    0x6A => PUSH11 => stack::push::<11>,
    0x6B => PUSH12 => stack::push::<12>,
    0x6C => PUSH13 => stack::push::<13>,
    0x6D => PUSH14 => stack::push::<14>,
    0x6E => PUSH15 => stack::push::<15>,
    0x6F => PUSH16 => stack::push::<16>,
    0x70 => PUSH17 => stack::push::<17>,
    0x71 => PUSH18 => stack::push::<18>,
    0x72 => PUSH19 => stack::push::<19>,
    0x73 => PUSH20 => stack::push::<20>,
    0x74 => PUSH21 => stack::push::<21>,
    0x75 => PUSH22 => stack::push::<22>,
    0x76 => PUSH23 => stack::push::<23>,
    0x77 => PUSH24 => stack::push::<24>,
    0x78 => PUSH25 => stack::push::<25>,
    0x79 => PUSH26 => stack::push::<26>,
    0x7A => PUSH27 => stack::push::<27>,
    0x7B => PUSH28 => stack::push::<28>,
    0x7C => PUSH29 => stack::push::<29>,
    0x7D => PUSH30 => stack::push::<30>,
    0x7E => PUSH31 => stack::push::<31>,
    0x7F => PUSH32 => stack::push::<32>,

    0x80 => DUP1  => stack::dup::<1>,
    0x81 => DUP2  => stack::dup::<2>,
    0x82 => DUP3  => stack::dup::<3>,
    0x83 => DUP4  => stack::dup::<4>,
    0x84 => DUP5  => stack::dup::<5>,
    0x85 => DUP6  => stack::dup::<6>,
    0x86 => DUP7  => stack::dup::<7>,
    0x87 => DUP8  => stack::dup::<8>,
    0x88 => DUP9  => stack::dup::<9>,
    0x89 => DUP10 => stack::dup::<10>,
    0x8A => DUP11 => stack::dup::<11>,
    0x8B => DUP12 => stack::dup::<12>,
    0x8C => DUP13 => stack::dup::<13>,
    0x8D => DUP14 => stack::dup::<14>,
    0x8E => DUP15 => stack::dup::<15>,
    0x8F => DUP16 => stack::dup::<16>,

    0x90 => SWAP1  => stack::swap::<1>,
    0x91 => SWAP2  => stack::swap::<2>,
    0x92 => SWAP3  => stack::swap::<3>,
    0x93 => SWAP4  => stack::swap::<4>,
    0x94 => SWAP5  => stack::swap::<5>,
    0x95 => SWAP6  => stack::swap::<6>,
    0x96 => SWAP7  => stack::swap::<7>,
    0x97 => SWAP8  => stack::swap::<8>,
    0x98 => SWAP9  => stack::swap::<9>,
    0x99 => SWAP10 => stack::swap::<10>,
    0x9A => SWAP11 => stack::swap::<11>,
    0x9B => SWAP12 => stack::swap::<12>,
    0x9C => SWAP13 => stack::swap::<13>,
    0x9D => SWAP14 => stack::swap::<14>,
    0x9E => SWAP15 => stack::swap::<15>,
    0x9F => SWAP16 => stack::swap::<16>,

    0xA0 => LOG0 => host::log::<0>,
    0xA1 => LOG1 => host::log::<1>,
    0xA2 => LOG2 => host::log::<2>,
    0xA3 => LOG3 => host::log::<3>,
    0xA4 => LOG4 => host::log::<4>,
    // 0xA5
    // 0xA6
    // 0xA7
    // 0xA8
    // 0xA9
    // 0xAA
    // 0xAB
    // 0xAC
    // 0xAD
    // 0xAE
    // 0xAF
    // 0xB0
    // 0xB1
    // 0xB2
    // 0xB3
    // 0xB4
    // 0xB5
    // 0xB6
    // 0xB7
    // 0xB8
    // 0xB9
    // 0xBA
    // 0xBB
    // 0xBC
    // 0xBD
    // 0xBE
    // 0xBF
    // 0xC0
    // 0xC1
    // 0xC2
    // 0xC3
    // 0xC4
    // 0xC5
    // 0xC6
    // 0xC7
    // 0xC8
    // 0xC9
    // 0xCA
    // 0xCB
    // 0xCC
    // 0xCD
    // 0xCE
    // 0xCF
    // 0xD0
    // 0xD1
    // 0xD2
    // 0xD3
    // 0xD4
    // 0xD5
    // 0xD6
    // 0xD7
    // 0xD8
    // 0xD9
    // 0xDA
    // 0xDB
    // 0xDC
    // 0xDD
    // 0xDE
    // 0xDF
    // 0xE0
    // 0xE1
    // 0xE2
    // 0xE3
    // 0xE4
    // 0xE5
    // 0xE6
    // 0xE7
    // 0xE8
    // 0xE9
    // 0xEA
    // 0xEB
    // 0xEC
    // 0xED
    // 0xEE
    // 0xEF
    0xF0 => CREATE       => host::create::<false, SPEC>,
    0xF1 => CALL         => host::call::<SPEC>,
    0xF2 => CALLCODE     => host::call_code::<SPEC>,
    0xF3 => RETURN       => control::ret,
    0xF4 => DELEGATECALL => host::delegate_call::<SPEC>,
    0xF5 => CREATE2      => host::create::<true, SPEC>,
    // 0xF6
    // 0xF7
    // 0xF8
    // 0xF9
    0xFA => STATICCALL   => host::static_call::<SPEC>,
    // 0xFB
    // 0xF
    0xFD => REVERT       => control::revert::<SPEC>,
    0xFE => INVALID      => control::invalid,
    0xFF => SELFDESTRUCT => host::selfdestruct::<SPEC>,
}

/// An EVM opcode.
///
/// This is always a valid opcode, as declared in the [`opcode`][self] module or the
/// [`OPCODE_JUMPMAP`] constant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OpCode(u8);

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.get();
        if let Some(val) = OPCODE_JUMPMAP[n as usize] {
            f.write_str(val)
        } else {
            write!(f, "UNKNOWN(0x{n:02X})")
        }
    }
}

impl OpCode {
    /// Instantiate a new opcode from a u8.
    #[inline]
    pub const fn new(opcode: u8) -> Option<Self> {
        match OPCODE_JUMPMAP[opcode as usize] {
            Some(_) => Some(Self(opcode)),
            None => None,
        }
    }

    /// Instantiate a new opcode from a u8 without checking if it is valid.
    ///
    /// # Safety
    ///
    /// All code using `Opcode` values assume that they are valid opcodes, so providing an invalid
    /// opcode may cause undefined behavior.
    #[inline]
    pub unsafe fn new_unchecked(opcode: u8) -> Self {
        Self(opcode)
    }

    /// Returns the opcode as a string.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        if let Some(str) = OPCODE_JUMPMAP[self.0 as usize] {
            str
        } else {
            "unknown"
        }
    }

    /// Returns the opcode as a u8.
    #[inline]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline]
    #[deprecated(note = "use `new` instead")]
    #[doc(hidden)]
    pub const fn try_from_u8(opcode: u8) -> Option<Self> {
        Self::new(opcode)
    }

    #[inline]
    #[deprecated(note = "use `get` instead")]
    #[doc(hidden)]
    pub const fn u8(self) -> u8 {
        self.get()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpInfo {
    /// Data contains few information packed inside u32:
    /// IS_JUMP (1bit) | IS_GAS_BLOCK_END (1bit) | IS_PUSH (1bit) | gas (29bits)
    data: u32,
}

const JUMP_MASK: u32 = 0x80000000;
const GAS_BLOCK_END_MASK: u32 = 0x40000000;
const IS_PUSH_MASK: u32 = 0x20000000;
const GAS_MASK: u32 = 0x1FFFFFFF;

impl OpInfo {
    /// Creates a new empty [`OpInfo`].
    pub const fn none() -> Self {
        Self { data: 0 }
    }

    /// Creates a new dynamic gas [`OpInfo`].
    pub const fn dynamic_gas() -> Self {
        Self { data: 0 }
    }

    /// Creates a new gas block end [`OpInfo`].
    pub const fn gas_block_end(gas: u64) -> Self {
        Self {
            data: gas as u32 | GAS_BLOCK_END_MASK,
        }
    }

    /// Creates a new [`OpInfo`] with the given gas value.
    pub const fn gas(gas: u64) -> Self {
        Self { data: gas as u32 }
    }

    /// Creates a new push [`OpInfo`].
    pub const fn push_opcode() -> Self {
        Self {
            data: gas::VERYLOW as u32 | IS_PUSH_MASK,
        }
    }

    /// Creates a new jumpdest [`OpInfo`].
    pub const fn jumpdest() -> Self {
        Self {
            data: JUMP_MASK | GAS_BLOCK_END_MASK,
        }
    }

    /// Returns whether the opcode is a jump.
    #[inline]
    pub fn is_jump(self) -> bool {
        self.data & JUMP_MASK == JUMP_MASK
    }

    /// Returns whether the opcode is a gas block end.
    #[inline]
    pub fn is_gas_block_end(self) -> bool {
        self.data & GAS_BLOCK_END_MASK == GAS_BLOCK_END_MASK
    }

    /// Returns whether the opcode is a push.
    #[inline]
    pub fn is_push(self) -> bool {
        self.data & IS_PUSH_MASK == IS_PUSH_MASK
    }

    /// Returns the gas cost of the opcode.
    #[inline]
    pub fn get_gas(self) -> u32 {
        self.data & GAS_MASK
    }
}

const fn opcode_gas_info(opcode: u8, spec: SpecId) -> OpInfo {
    match opcode {
        STOP => OpInfo::gas_block_end(0),
        ADD => OpInfo::gas(gas::VERYLOW),
        MUL => OpInfo::gas(gas::LOW),
        SUB => OpInfo::gas(gas::VERYLOW),
        DIV => OpInfo::gas(gas::LOW),
        SDIV => OpInfo::gas(gas::LOW),
        MOD => OpInfo::gas(gas::LOW),
        SMOD => OpInfo::gas(gas::LOW),
        ADDMOD => OpInfo::gas(gas::MID),
        MULMOD => OpInfo::gas(gas::MID),
        EXP => OpInfo::dynamic_gas(),
        SIGNEXTEND => OpInfo::gas(gas::LOW),

        LT => OpInfo::gas(gas::VERYLOW),
        GT => OpInfo::gas(gas::VERYLOW),
        SLT => OpInfo::gas(gas::VERYLOW),
        SGT => OpInfo::gas(gas::VERYLOW),
        EQ => OpInfo::gas(gas::VERYLOW),
        ISZERO => OpInfo::gas(gas::VERYLOW),
        AND => OpInfo::gas(gas::VERYLOW),
        OR => OpInfo::gas(gas::VERYLOW),
        XOR => OpInfo::gas(gas::VERYLOW),
        NOT => OpInfo::gas(gas::VERYLOW),
        BYTE => OpInfo::gas(gas::VERYLOW),
        SHL => OpInfo::gas(if SpecId::enabled(spec, SpecId::CONSTANTINOPLE) {
            gas::VERYLOW
        } else {
            0
        }),
        SHR => OpInfo::gas(if SpecId::enabled(spec, SpecId::CONSTANTINOPLE) {
            gas::VERYLOW
        } else {
            0
        }),
        SAR => OpInfo::gas(if SpecId::enabled(spec, SpecId::CONSTANTINOPLE) {
            gas::VERYLOW
        } else {
            0
        }),

        KECCAK256 => OpInfo::dynamic_gas(),

        ADDRESS => OpInfo::gas(gas::BASE),
        BALANCE => OpInfo::dynamic_gas(),
        ORIGIN => OpInfo::gas(gas::BASE),
        CALLER => OpInfo::gas(gas::BASE),
        CALLVALUE => OpInfo::gas(gas::BASE),
        CALLDATALOAD => OpInfo::gas(gas::VERYLOW),
        CALLDATASIZE => OpInfo::gas(gas::BASE),
        CALLDATACOPY => OpInfo::dynamic_gas(),
        CODESIZE => OpInfo::gas(gas::BASE),
        CODECOPY => OpInfo::dynamic_gas(),
        GASPRICE => OpInfo::gas(gas::BASE),
        EXTCODESIZE => OpInfo::gas(if SpecId::enabled(spec, SpecId::BERLIN) {
            gas::WARM_STORAGE_READ_COST // add only part of gas
        } else if SpecId::enabled(spec, SpecId::TANGERINE) {
            700
        } else {
            20
        }),
        EXTCODECOPY => OpInfo::gas(if SpecId::enabled(spec, SpecId::BERLIN) {
            gas::WARM_STORAGE_READ_COST // add only part of gas
        } else if SpecId::enabled(spec, SpecId::TANGERINE) {
            700
        } else {
            20
        }),
        RETURNDATASIZE => OpInfo::gas(if SpecId::enabled(spec, SpecId::BYZANTIUM) {
            gas::BASE
        } else {
            0
        }),
        RETURNDATACOPY => OpInfo::dynamic_gas(),
        EXTCODEHASH => OpInfo::gas(if SpecId::enabled(spec, SpecId::BERLIN) {
            gas::WARM_STORAGE_READ_COST // add only part of gas
        } else if SpecId::enabled(spec, SpecId::ISTANBUL) {
            700
        } else if SpecId::enabled(spec, SpecId::PETERSBURG) {
            400 // constantinople
        } else {
            0 // not enabled
        }),
        BLOCKHASH => OpInfo::gas(gas::BLOCKHASH),
        COINBASE => OpInfo::gas(gas::BASE),
        TIMESTAMP => OpInfo::gas(gas::BASE),
        NUMBER => OpInfo::gas(gas::BASE),
        DIFFICULTY => OpInfo::gas(gas::BASE),
        GASLIMIT => OpInfo::gas(gas::BASE),
        CHAINID => OpInfo::gas(if SpecId::enabled(spec, SpecId::ISTANBUL) {
            gas::BASE
        } else {
            0
        }),
        SELFBALANCE => OpInfo::gas(if SpecId::enabled(spec, SpecId::ISTANBUL) {
            gas::LOW
        } else {
            0
        }),
        BASEFEE => OpInfo::gas(if SpecId::enabled(spec, SpecId::LONDON) {
            gas::BASE
        } else {
            0
        }),
        BLOBHASH => OpInfo::gas(if SpecId::enabled(spec, SpecId::CANCUN) {
            gas::VERYLOW
        } else {
            0
        }),

        POP => OpInfo::gas(gas::BASE),
        MLOAD => OpInfo::gas(gas::VERYLOW),
        MSTORE => OpInfo::gas(gas::VERYLOW),
        MSTORE8 => OpInfo::gas(gas::VERYLOW),
        SLOAD => OpInfo::dynamic_gas(),
        SSTORE => OpInfo::gas_block_end(0),
        JUMP => OpInfo::gas_block_end(gas::MID),
        JUMPI => OpInfo::gas_block_end(gas::HIGH),
        PC => OpInfo::gas(gas::BASE),
        MSIZE => OpInfo::gas(gas::BASE),
        GAS => OpInfo::gas_block_end(gas::BASE),
        // gas::JUMPDEST gas is calculated in function call
        JUMPDEST => OpInfo::jumpdest(),
        TLOAD => OpInfo::gas(if SpecId::enabled(spec, SpecId::CANCUN) {
            gas::WARM_STORAGE_READ_COST
        } else {
            0
        }),
        TSTORE => OpInfo::gas(if SpecId::enabled(spec, SpecId::CANCUN) {
            gas::WARM_STORAGE_READ_COST
        } else {
            0
        }),
        MCOPY => OpInfo::dynamic_gas(),

        PUSH0 => OpInfo::gas(if SpecId::enabled(spec, SpecId::SHANGHAI) {
            gas::BASE
        } else {
            0
        }),
        PUSH1 => OpInfo::push_opcode(),
        PUSH2 => OpInfo::push_opcode(),
        PUSH3 => OpInfo::push_opcode(),
        PUSH4 => OpInfo::push_opcode(),
        PUSH5 => OpInfo::push_opcode(),
        PUSH6 => OpInfo::push_opcode(),
        PUSH7 => OpInfo::push_opcode(),
        PUSH8 => OpInfo::push_opcode(),
        PUSH9 => OpInfo::push_opcode(),
        PUSH10 => OpInfo::push_opcode(),
        PUSH11 => OpInfo::push_opcode(),
        PUSH12 => OpInfo::push_opcode(),
        PUSH13 => OpInfo::push_opcode(),
        PUSH14 => OpInfo::push_opcode(),
        PUSH15 => OpInfo::push_opcode(),
        PUSH16 => OpInfo::push_opcode(),
        PUSH17 => OpInfo::push_opcode(),
        PUSH18 => OpInfo::push_opcode(),
        PUSH19 => OpInfo::push_opcode(),
        PUSH20 => OpInfo::push_opcode(),
        PUSH21 => OpInfo::push_opcode(),
        PUSH22 => OpInfo::push_opcode(),
        PUSH23 => OpInfo::push_opcode(),
        PUSH24 => OpInfo::push_opcode(),
        PUSH25 => OpInfo::push_opcode(),
        PUSH26 => OpInfo::push_opcode(),
        PUSH27 => OpInfo::push_opcode(),
        PUSH28 => OpInfo::push_opcode(),
        PUSH29 => OpInfo::push_opcode(),
        PUSH30 => OpInfo::push_opcode(),
        PUSH31 => OpInfo::push_opcode(),
        PUSH32 => OpInfo::push_opcode(),

        DUP1 => OpInfo::gas(gas::VERYLOW),
        DUP2 => OpInfo::gas(gas::VERYLOW),
        DUP3 => OpInfo::gas(gas::VERYLOW),
        DUP4 => OpInfo::gas(gas::VERYLOW),
        DUP5 => OpInfo::gas(gas::VERYLOW),
        DUP6 => OpInfo::gas(gas::VERYLOW),
        DUP7 => OpInfo::gas(gas::VERYLOW),
        DUP8 => OpInfo::gas(gas::VERYLOW),
        DUP9 => OpInfo::gas(gas::VERYLOW),
        DUP10 => OpInfo::gas(gas::VERYLOW),
        DUP11 => OpInfo::gas(gas::VERYLOW),
        DUP12 => OpInfo::gas(gas::VERYLOW),
        DUP13 => OpInfo::gas(gas::VERYLOW),
        DUP14 => OpInfo::gas(gas::VERYLOW),
        DUP15 => OpInfo::gas(gas::VERYLOW),
        DUP16 => OpInfo::gas(gas::VERYLOW),

        SWAP1 => OpInfo::gas(gas::VERYLOW),
        SWAP2 => OpInfo::gas(gas::VERYLOW),
        SWAP3 => OpInfo::gas(gas::VERYLOW),
        SWAP4 => OpInfo::gas(gas::VERYLOW),
        SWAP5 => OpInfo::gas(gas::VERYLOW),
        SWAP6 => OpInfo::gas(gas::VERYLOW),
        SWAP7 => OpInfo::gas(gas::VERYLOW),
        SWAP8 => OpInfo::gas(gas::VERYLOW),
        SWAP9 => OpInfo::gas(gas::VERYLOW),
        SWAP10 => OpInfo::gas(gas::VERYLOW),
        SWAP11 => OpInfo::gas(gas::VERYLOW),
        SWAP12 => OpInfo::gas(gas::VERYLOW),
        SWAP13 => OpInfo::gas(gas::VERYLOW),
        SWAP14 => OpInfo::gas(gas::VERYLOW),
        SWAP15 => OpInfo::gas(gas::VERYLOW),
        SWAP16 => OpInfo::gas(gas::VERYLOW),

        LOG0 => OpInfo::dynamic_gas(),
        LOG1 => OpInfo::dynamic_gas(),
        LOG2 => OpInfo::dynamic_gas(),
        LOG3 => OpInfo::dynamic_gas(),
        LOG4 => OpInfo::dynamic_gas(),

        CREATE => OpInfo::gas_block_end(0),
        CALL => OpInfo::gas_block_end(0),
        CALLCODE => OpInfo::gas_block_end(0),
        RETURN => OpInfo::gas_block_end(0),
        DELEGATECALL => OpInfo::gas_block_end(0),
        CREATE2 => OpInfo::gas_block_end(0),

        STATICCALL => OpInfo::gas_block_end(0),

        REVERT => OpInfo::gas_block_end(0),
        INVALID => OpInfo::gas_block_end(0),
        SELFDESTRUCT => OpInfo::gas_block_end(0),

        _ => OpInfo::none(),
    }
}

const fn make_gas_table(spec: SpecId) -> [OpInfo; 256] {
    let mut table = [OpInfo::none(); 256];
    let mut i = 0;
    while i < 256 {
        table[i] = opcode_gas_info(i as u8, spec);
        i += 1;
    }
    table
}

/// Returns a lookup table of opcode gas info for the given [`SpecId`].
#[inline]
pub const fn spec_opcode_gas(spec_id: SpecId) -> &'static [OpInfo; 256] {
    macro_rules! gas_maps {
        ($($id:ident),* $(,)?) => {
            match spec_id {$(
                SpecId::$id => {
                    const TABLE: &[OpInfo; 256] = &make_gas_table(SpecId::$id);
                    TABLE
                }
            )*}
        };
    }

    gas_maps!(
        FRONTIER,
        FRONTIER_THAWING,
        HOMESTEAD,
        DAO_FORK,
        TANGERINE,
        SPURIOUS_DRAGON,
        BYZANTIUM,
        CONSTANTINOPLE,
        PETERSBURG,
        ISTANBUL,
        MUIR_GLACIER,
        BERLIN,
        LONDON,
        ARROW_GLACIER,
        GRAY_GLACIER,
        MERGE,
        SHANGHAI,
        CANCUN,
        LATEST,
    )
}
