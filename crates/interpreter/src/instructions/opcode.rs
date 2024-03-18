//! EVM opcode definitions and utilities.

use super::*;
use crate::{
    gas,
    primitives::{Spec, SpecId},
    Host, Interpreter,
};
use core::fmt;
use std::boxed::Box;

/// EVM opcode function signature.
pub type Instruction<H> = fn(&mut Interpreter, &mut H);

/// Instruction table is list of instruction function pointers mapped to
/// 256 EVM opcodes.
pub type InstructionTable<H> = [Instruction<H>; 256];

/// EVM opcode function signature.
pub type BoxedInstruction<'a, H> = Box<dyn Fn(&mut Interpreter, &mut H) + 'a>;

/// A table of instructions.
pub type BoxedInstructionTable<'a, H> = [BoxedInstruction<'a, H>; 256];

/// Instruction set that contains plain instruction table that contains simple `fn` function pointer.
/// and Boxed `Fn` variant that contains `Box<dyn Fn()>` function pointer that can be used with closured.
///
/// Note that `Plain` variant gives us 10-20% faster Interpreter execution.
///
/// Boxed variant can be used to wrap plain function pointer with closure.
pub enum InstructionTables<'a, H> {
    Plain(InstructionTable<H>),
    Boxed(BoxedInstructionTable<'a, H>),
}

impl<H: Host> InstructionTables<'_, H> {
    /// Creates a plain instruction table for the given spec.
    #[inline]
    pub const fn new_plain<SPEC: Spec>() -> Self {
        Self::Plain(make_instruction_table::<H, SPEC>())
    }
}

impl<'a, H: Host + 'a> InstructionTables<'a, H> {
    /// Inserts the instruction into the table with the specified index.
    #[inline]
    pub fn insert(&mut self, opcode: u8, instruction: Instruction<H>) {
        match self {
            Self::Plain(table) => {
                table[opcode as usize] = instruction;
            }
            Self::Boxed(table) => {
                table[opcode as usize] = Box::new(instruction);
            }
        }
    }
}

/// Make instruction table.
#[inline]
pub const fn make_instruction_table<H: Host, SPEC: Spec>() -> InstructionTable<H> {
    // Force const-eval of the table creation, making this function trivial.
    // TODO: Replace this with a `const {}` block once it is stable.
    struct ConstTable<H: Host, SPEC: Spec> {
        _phantom: core::marker::PhantomData<(H, SPEC)>,
    }
    impl<H: Host, SPEC: Spec> ConstTable<H, SPEC> {
        const NEW: InstructionTable<H> = {
            let mut tables: InstructionTable<H> = [control::unknown; 256];
            let mut i = 0;
            while i < 256 {
                tables[i] = instruction::<H, SPEC>(i as u8);
                i += 1;
            }
            tables
        };
    }
    ConstTable::<H, SPEC>::NEW
}

/// Make boxed instruction table that calls `outer` closure for every instruction.
#[inline]
pub fn make_boxed_instruction_table<'a, H, SPEC, FN>(
    table: InstructionTable<H>,
    mut outer: FN,
) -> BoxedInstructionTable<'a, H>
where
    H: Host,
    SPEC: Spec + 'a,
    FN: FnMut(Instruction<H>) -> BoxedInstruction<'a, H>,
{
    core::array::from_fn(|i| outer(table[i]))
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

    /// Returns true if the opcode is a jump destination.
    #[inline]
    pub const fn is_jumpdest(&self) -> bool {
        self.0 == JUMPDEST
    }

    /// Takes a u8 and returns true if it is a jump destination.
    pub const fn is_jumpdest_op(opcode: u8) -> bool {
        if let Some(opcode) = Self::new(opcode) {
            opcode.is_jumpdest()
        } else {
            false
        }
    }

    /// Returns true if the opcode is a legacy jump instruction.
    #[inline]
    pub const fn is_jump(self) -> bool {
        self.0 == JUMP
    }

    /// Takes a u8 and returns true if it is a jump instruction.
    pub const fn is_jump_op(opcode: u8) -> bool {
        if let Some(opcode) = Self::new(opcode) {
            opcode.is_jump()
        } else {
            false
        }
    }

    /// Returns true if the opcode is a push instruction.
    #[inline]
    pub const fn is_push(self) -> bool {
        self.0 >= PUSH1 && self.0 <= PUSH32
    }

    /// Takes a u8 and returns true if it is a push instruction.
    pub fn is_push_op(opcode: u8) -> bool {
        if let Some(opcode) = Self::new(opcode) {
            opcode.is_push()
        } else {
            false
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
}

pub const NOP: u8 = JUMPDEST;

macro_rules! opcodes {
    ($($val:literal => $name:ident => $f:expr => ($inputs:literal,$outputs:literal)),* $(,)?) => {
        // Constants for each opcode. This also takes care of duplicate names.
        $(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($name),"\") opcode.")]
            pub const $name: u8 = $val;
        )*
        impl OpCode {$(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($name),"\") opcode.")]
            pub const $name: Self = Self($val);
        )*}

        impl OpCode {

            /// Returns inputs for the given opcode.
            pub const fn inputs(&self) -> u8 {
                match self.0 {
                    $($val => $inputs,)*
                    _ => 0,
                }
            }

            /// Returns outputs for the given opcode.
            pub const fn outputs(&self) -> u8 {
                match self.0 {
                    $($val => $outputs,)*
                    _ => 0,
                }
            }

            /// Returns a difference between input and output.
            pub const fn diff(&self) -> i8 {
                match self.0 {
                    $($val => $outputs as i8 - $inputs as i8,)*
                    _ => 0,
                }
            }

            /// Returns a tuple of input and output.
            /// Can be slightly faster that calling `inputs` and `outputs` separately.
            pub const fn input_output(&self) -> (u8,u8) {
                match self.0 {
                    $($val => ($inputs,$outputs),)*
                    _ => (0,0),
                }
            }
        }

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

        /// Returns the instruction function for the given opcode and spec.
        pub const fn instruction<H: Host, SPEC: Spec>(opcode: u8) -> Instruction<H> {
            match opcode {
                $($name => $f,)*
                _ => control::unknown,
            }
        }
    };
}

// When adding new opcodes:
// 1. add the opcode to the list below; make sure it's sorted by opcode value
// 2. add its gas info in the `opcode_gas_info` function below
// 3. implement the opcode in the corresponding module;
//    the function signature must be the exact same as the others
opcodes! {
    0x00 => STOP => control::stop => (0,0),

    0x01 => ADD        => arithmetic::wrapping_add   => (2,1),
    0x02 => MUL        => arithmetic::wrapping_mul   => (2,1),
    0x03 => SUB        => arithmetic::wrapping_sub   => (2,1),
    0x04 => DIV        => arithmetic::div            => (2,1),
    0x05 => SDIV       => arithmetic::sdiv           => (2,1),
    0x06 => MOD        => arithmetic::rem            => (2,1),
    0x07 => SMOD       => arithmetic::smod           => (2,1),
    0x08 => ADDMOD     => arithmetic::addmod         => (3,1),
    0x09 => MULMOD     => arithmetic::mulmod         => (3,1),
    0x0A => EXP        => arithmetic::exp::<H, SPEC> => (2,1),
    0x0B => SIGNEXTEND => arithmetic::signextend     => (2,1),
    // 0x0C
    // 0x0D
    // 0x0E
    // 0x0F
    0x10 => LT     => bitwise::lt             => (2,1),
    0x11 => GT     => bitwise::gt             => (2,1),
    0x12 => SLT    => bitwise::slt            => (2,1),
    0x13 => SGT    => bitwise::sgt            => (2,1),
    0x14 => EQ     => bitwise::eq             => (2,1),
    0x15 => ISZERO => bitwise::iszero         => (1,1),
    0x16 => AND    => bitwise::bitand         => (2,1),
    0x17 => OR     => bitwise::bitor          => (2,1),
    0x18 => XOR    => bitwise::bitxor         => (2,1),
    0x19 => NOT    => bitwise::not            => (1,1),
    0x1A => BYTE   => bitwise::byte           => (2,1),
    0x1B => SHL    => bitwise::shl::<H, SPEC> => (2,1),
    0x1C => SHR    => bitwise::shr::<H, SPEC> => (2,1),
    0x1D => SAR    => bitwise::sar::<H, SPEC> => (2,1),
    // 0x1E
    // 0x1F
    0x20 => KECCAK256 => system::keccak256    => (2,1),
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
    0x30 => ADDRESS      => system::address          => (0,1),
    0x31 => BALANCE      => host::balance::<H, SPEC> => (1,1),
    0x32 => ORIGIN       => host_env::origin         => (0,1),
    0x33 => CALLER       => system::caller           => (0,1),
    0x34 => CALLVALUE    => system::callvalue        => (0,1),
    0x35 => CALLDATALOAD => system::calldataload     => (1,1),
    0x36 => CALLDATASIZE => system::calldatasize     => (0,1),
    0x37 => CALLDATACOPY => system::calldatacopy     => (0,1),
    0x38 => CODESIZE     => system::codesize         => (0,1),
    0x39 => CODECOPY     => system::codecopy         => (3,0),

    0x3A => GASPRICE       => host_env::gasprice                => (0,1),
    0x3B => EXTCODESIZE    => host::extcodesize::<H, SPEC>      => (1,1),
    0x3C => EXTCODECOPY    => host::extcodecopy::<H, SPEC>      => (4,0),
    0x3D => RETURNDATASIZE => system::returndatasize::<H, SPEC> => (0,1),
    0x3E => RETURNDATACOPY => system::returndatacopy::<H, SPEC> => (3,0),
    0x3F => EXTCODEHASH    => host::extcodehash::<H, SPEC>      => (1,1),
    0x40 => BLOCKHASH      => host::blockhash                   => (1,1),
    0x41 => COINBASE       => host_env::coinbase                => (0,1),
    0x42 => TIMESTAMP      => host_env::timestamp               => (0,1),
    0x43 => NUMBER         => host_env::block_number            => (0,1),
    0x44 => DIFFICULTY     => host_env::difficulty::<H, SPEC>   => (0,1),
    0x45 => GASLIMIT       => host_env::gaslimit                => (0,1),
    0x46 => CHAINID        => host_env::chainid::<H, SPEC>      => (0,1),
    0x47 => SELFBALANCE    => host::selfbalance::<H, SPEC>      => (0,1),
    0x48 => BASEFEE        => host_env::basefee::<H, SPEC>      => (0,1),
    0x49 => BLOBHASH       => host_env::blob_hash::<H, SPEC>    => (1,1),
    0x4A => BLOBBASEFEE    => host_env::blob_basefee::<H, SPEC> => (0,1),
    // 0x4B
    // 0x4C
    // 0x4D
    // 0x4E
    // 0x4F
    0x50 => POP      => stack::pop               => (1,0),
    0x51 => MLOAD    => memory::mload            => (1,1),
    0x52 => MSTORE   => memory::mstore           => (2,0),
    0x53 => MSTORE8  => memory::mstore8          => (2,0),
    0x54 => SLOAD    => host::sload::<H, SPEC>   => (1,1),
    0x55 => SSTORE   => host::sstore::<H, SPEC>  => (2,0),
    0x56 => JUMP     => control::jump            => (1,0),
    0x57 => JUMPI    => control::jumpi           => (2,0),
    0x58 => PC       => control::pc              => (0,1),
    0x59 => MSIZE    => memory::msize            => (0,1),
    0x5A => GAS      => system::gas              => (0,1),
    0x5B => JUMPDEST => control::jumpdest_or_nop => (0,0),
    0x5C => TLOAD    => host::tload::<H, SPEC>   => (1,1),
    0x5D => TSTORE   => host::tstore::<H, SPEC>  => (2,0),
    0x5E => MCOPY    => memory::mcopy::<H, SPEC> => (3,0),

    0x5F => PUSH0  => stack::push0::<H, SPEC> => (0,1),
    0x60 => PUSH1  => stack::push::<1, H>  => (0,1),
    0x61 => PUSH2  => stack::push::<2, H>  => (0,1),
    0x62 => PUSH3  => stack::push::<3, H>  => (0,1),
    0x63 => PUSH4  => stack::push::<4, H>  => (0,1),
    0x64 => PUSH5  => stack::push::<5, H>  => (0,1),
    0x65 => PUSH6  => stack::push::<6, H>  => (0,1),
    0x66 => PUSH7  => stack::push::<7, H>  => (0,1),
    0x67 => PUSH8  => stack::push::<8, H>  => (0,1),
    0x68 => PUSH9  => stack::push::<9, H>  => (0,1),
    0x69 => PUSH10 => stack::push::<10, H> => (0,1),
    0x6A => PUSH11 => stack::push::<11, H> => (0,1),
    0x6B => PUSH12 => stack::push::<12, H> => (0,1),
    0x6C => PUSH13 => stack::push::<13, H> => (0,1),
    0x6D => PUSH14 => stack::push::<14, H> => (0,1),
    0x6E => PUSH15 => stack::push::<15, H> => (0,1),
    0x6F => PUSH16 => stack::push::<16, H> => (0,1),
    0x70 => PUSH17 => stack::push::<17, H> => (0,1),
    0x71 => PUSH18 => stack::push::<18, H> => (0,1),
    0x72 => PUSH19 => stack::push::<19, H> => (0,1),
    0x73 => PUSH20 => stack::push::<20, H> => (0,1),
    0x74 => PUSH21 => stack::push::<21, H> => (0,1),
    0x75 => PUSH22 => stack::push::<22, H> => (0,1),
    0x76 => PUSH23 => stack::push::<23, H> => (0,1),
    0x77 => PUSH24 => stack::push::<24, H> => (0,1),
    0x78 => PUSH25 => stack::push::<25, H> => (0,1),
    0x79 => PUSH26 => stack::push::<26, H> => (0,1),
    0x7A => PUSH27 => stack::push::<27, H> => (0,1),
    0x7B => PUSH28 => stack::push::<28, H> => (0,1),
    0x7C => PUSH29 => stack::push::<29, H> => (0,1),
    0x7D => PUSH30 => stack::push::<30, H> => (0,1),
    0x7E => PUSH31 => stack::push::<31, H> => (0,1),
    0x7F => PUSH32 => stack::push::<32, H> => (0,1),

    0x80 => DUP1  => stack::dup::<1, H> => (0,1),
    0x81 => DUP2  => stack::dup::<2, H> => (0,1),
    0x82 => DUP3  => stack::dup::<3, H> => (0,1),
    0x83 => DUP4  => stack::dup::<4, H> => (0,1),
    0x84 => DUP5  => stack::dup::<5, H> => (0,1),
    0x85 => DUP6  => stack::dup::<6, H> => (0,1),
    0x86 => DUP7  => stack::dup::<7, H> => (0,1),
    0x87 => DUP8  => stack::dup::<8, H> => (0,1),
    0x88 => DUP9  => stack::dup::<9, H> => (0,1),
    0x89 => DUP10 => stack::dup::<10, H> => (0,1),
    0x8A => DUP11 => stack::dup::<11, H> => (0,1),
    0x8B => DUP12 => stack::dup::<12, H> => (0,1),
    0x8C => DUP13 => stack::dup::<13, H> => (0,1),
    0x8D => DUP14 => stack::dup::<14, H> => (0,1),
    0x8E => DUP15 => stack::dup::<15, H> => (0,1),
    0x8F => DUP16 => stack::dup::<16, H> => (0,1),

    0x90 => SWAP1  => stack::swap::<1, H> => (0,0),
    0x91 => SWAP2  => stack::swap::<2, H> => (0,0),
    0x92 => SWAP3  => stack::swap::<3, H> => (0,0),
    0x93 => SWAP4  => stack::swap::<4, H> => (0,0),
    0x94 => SWAP5  => stack::swap::<5, H> => (0,0),
    0x95 => SWAP6  => stack::swap::<6, H> => (0,0),
    0x96 => SWAP7  => stack::swap::<7, H> => (0,0),
    0x97 => SWAP8  => stack::swap::<8, H> => (0,0),
    0x98 => SWAP9  => stack::swap::<9, H> => (0,0),
    0x99 => SWAP10 => stack::swap::<10, H> => (0,0),
    0x9A => SWAP11 => stack::swap::<11, H> => (0,0),
    0x9B => SWAP12 => stack::swap::<12, H> => (0,0),
    0x9C => SWAP13 => stack::swap::<13, H> => (0,0),
    0x9D => SWAP14 => stack::swap::<14, H> => (0,0),
    0x9E => SWAP15 => stack::swap::<15, H> => (0,0),
    0x9F => SWAP16 => stack::swap::<16, H> => (0,0),

    0xA0 => LOG0 => host::log::<0, H> => (2,0),
    0xA1 => LOG1 => host::log::<1, H> => (3,0),
    0xA2 => LOG2 => host::log::<2, H> => (4,0),
    0xA3 => LOG3 => host::log::<3, H> => (5,0),
    0xA4 => LOG4 => host::log::<4, H> => (6,0),
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
    0xD0 => DATALOAD  => data::data_load   => (1,1),
    0xD1 => DATALOADN => data::data_loadn  => (0,1),
    0xD2 => DATASIZE  => data::data_size   => (0,1),
    0xD3 => DATACOPY  => data::data_copy   => (3,0),
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
    0xE0 => RJUMP => control::rjump => (0,0),
    0xE1 => RJUMPI => control::rjumpi => (1,0),
    0xE2 => RJUMPV => control::rjumpv => (1,0),
    0xE3 => CALLF => control::callf => (0,0),
    0xE4 => RETF => control::retf => (0,0),
    0xE5 => JUMPF => control::jumpf => (0,0),
    0xE6 => DUPN => stack::dupn => (0,0),
    0xE7 => SWAPN => stack::swapn => (0,0),
    0xE8 => EXCHANGE => stack::exchange => (0,0),
    // 0xE9
    // 0xEA
    // 0xEB
    0xEC => EOFCREATE => contract::eofcreate::<H> => (4,1),
    0xED => CREATE4 => contract::txcreate::<H> => (5,1),
    0xEE => RETURNCONTRACT => contract::return_contract::<H> => (0,0), // TODO(EOF) input/output
    // 0xEF
    0xF0 => CREATE       => contract::create::<false, H, SPEC> => (0,0),
    0xF1 => CALL         => contract::call::<H, SPEC> => (0,0),
    0xF2 => CALLCODE     => contract::call_code::<H, SPEC> => (0,0),
    0xF3 => RETURN       => control::ret => (0,0),
    0xF4 => DELEGATECALL => contract::delegate_call::<H, SPEC> => (0,0),
    0xF5 => CREATE2      => contract::create::<true, H, SPEC> => (0,0),
    // 0xF6
    0xF7 => RETURNDATALOAD => system::returndataload::<H> => (0,0),
    0xF8 => EXTCALL => contract::extcall::<H,SPEC> => (0,0),
    0xF9 => EXFCALL => contract::extdcall::<H, SPEC> => (0,0),
    0xFA => STATICCALL   => contract::static_call::<H, SPEC> => (0,0),
    0xFB => EXTSCALL => contract::extscall::<H> => (0,0),
    // 0xFC
    0xFD => REVERT       => control::revert::<H, SPEC> => (0,0),
    0xFE => INVALID      => control::invalid => (0,0),
    0xFF => SELFDESTRUCT => host::selfdestruct::<H, SPEC> => (0,0),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test opcode.
    pub fn test_opcode() {
        let opcode = OpCode::new(0x00).unwrap();
        assert_eq!(opcode.is_jumpdest(), true);
        assert_eq!(opcode.is_jump(), false);
        assert_eq!(opcode.is_push(), false);
        assert_eq!(opcode.as_str(), "STOP");
        assert_eq!(opcode.get(), 0x00);
    }
}
