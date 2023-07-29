use super::{prelude::*, *};
use core::fmt;

macro_rules! opcodes {
    ($($val:literal => $name:ident => $f:expr),* $(,)?) => {
        // Constants for each opcode. This also takes care of duplicates.
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

        /// Evaluates the opcode.
        #[inline(always)]
        pub(crate) fn eval<SPEC: Spec>(opcode: u8, interpreter: &mut Interpreter, host: &mut dyn Host) {
            // type Instruction = fn(&mut Interpreter, &mut dyn Host, SpecId);
            // const INSTRUCTIONS: [Instruction; 256] = {
            //     #[allow(unused_mut)]
            //     let mut instructions: [Instruction; 256] = [control::not_found; 256];
            //     $(
            //         instructions[$val] = $f;
            //     )*
            //     instructions
            // };
            // INSTRUCTIONS[opcode as usize](interpreter, host, spec);

            match opcode {
                $($name => $f(interpreter, host),)*
                _ => control::not_found(interpreter, host),
            }
        }
    };
}

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

    0x20 => KECCAK256 => system::keccak256,

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

    0xF0 => CREATE       => host::create::<SPEC, false>,
    0xF1 => CALL         => host::call::<SPEC>,
    0xF2 => CALLCODE     => host::call_code::<SPEC>,
    0xF3 => RETURN       => control::ret,
    0xF4 => DELEGATECALL => host::delegate_call::<SPEC>,
    0xF5 => CREATE2      => host::create::<SPEC, true>,
    0xFA => STATICCALL   => host::static_call::<SPEC>,
    0xFD => REVERT       => control::revert::<SPEC>,
    0xFE => INVALID      => control::invalid,
    0xFF => SELFDESTRUCT => host::selfdestruct::<SPEC>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    /// This function is unsafe because it does not check if the opcode is valid.
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
    pub const fn try_from_u8(opcode: u8) -> Option<Self> {
        Self::new(opcode)
    }

    #[inline]
    #[deprecated(note = "use `get` instead")]
    pub const fn u8(self) -> u8 {
        self.get()
    }
}

#[derive(Debug)]
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
    #[inline(always)]
    pub fn is_jump(&self) -> bool {
        self.data & JUMP_MASK == JUMP_MASK
    }

    #[inline(always)]
    pub fn is_gas_block_end(&self) -> bool {
        self.data & GAS_BLOCK_END_MASK == GAS_BLOCK_END_MASK
    }

    #[inline(always)]
    pub fn is_push(&self) -> bool {
        self.data & IS_PUSH_MASK == IS_PUSH_MASK
    }

    #[inline(always)]
    pub fn get_gas(&self) -> u32 {
        self.data & GAS_MASK
    }

    pub const fn none() -> Self {
        Self { data: 0 }
    }

    pub const fn gas_block_end(gas: u64) -> Self {
        Self {
            data: gas as u32 | GAS_BLOCK_END_MASK,
        }
    }
    pub const fn dynamic_gas() -> Self {
        Self { data: 0 }
    }

    pub const fn gas(gas: u64) -> Self {
        Self { data: gas as u32 }
    }

    pub const fn push_opcode() -> Self {
        Self {
            data: gas::VERYLOW as u32 | IS_PUSH_MASK,
        }
    }

    pub const fn jumpdest() -> Self {
        Self {
            data: JUMP_MASK | GAS_BLOCK_END_MASK,
        }
    }
}

macro_rules! gas_opcodee {
    ($name:ident, $spec_id:expr) => {
        const $name: &'static [OpInfo; 256] = &[
            /* 0x00  STOP */ OpInfo::gas_block_end(0),
            /* 0x01  ADD */ OpInfo::gas(gas::VERYLOW),
            /* 0x02  MUL */ OpInfo::gas(gas::LOW),
            /* 0x03  SUB */ OpInfo::gas(gas::VERYLOW),
            /* 0x04  DIV */ OpInfo::gas(gas::LOW),
            /* 0x05  SDIV */ OpInfo::gas(gas::LOW),
            /* 0x06  MOD */ OpInfo::gas(gas::LOW),
            /* 0x07  SMOD */ OpInfo::gas(gas::LOW),
            /* 0x08  ADDMOD */ OpInfo::gas(gas::MID),
            /* 0x09  MULMOD */ OpInfo::gas(gas::MID),
            /* 0x0A  EXP */ OpInfo::dynamic_gas(),
            /* 0x0B  SIGNEXTEND */ OpInfo::gas(gas::LOW),
            /* 0x0C */ OpInfo::none(),
            /* 0x0D */ OpInfo::none(),
            /* 0x0E */ OpInfo::none(),
            /* 0x0F */ OpInfo::none(),
            /* 0x10  LT */ OpInfo::gas(gas::VERYLOW),
            /* 0x11  GT */ OpInfo::gas(gas::VERYLOW),
            /* 0x12  SLT */ OpInfo::gas(gas::VERYLOW),
            /* 0x13  SGT */ OpInfo::gas(gas::VERYLOW),
            /* 0x14  EQ */ OpInfo::gas(gas::VERYLOW),
            /* 0x15  ISZERO */ OpInfo::gas(gas::VERYLOW),
            /* 0x16  AND */ OpInfo::gas(gas::VERYLOW),
            /* 0x17  OR */ OpInfo::gas(gas::VERYLOW),
            /* 0x18  XOR */ OpInfo::gas(gas::VERYLOW),
            /* 0x19  NOT */ OpInfo::gas(gas::VERYLOW),
            /* 0x1A  BYTE */ OpInfo::gas(gas::VERYLOW),
            /* 0x1B  SHL */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1C  SHR */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1D  SAR */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1E */ OpInfo::none(),
            /* 0x1F */ OpInfo::none(),
            /* 0x20  KECCAK256 */ OpInfo::dynamic_gas(),
            /* 0x21 */ OpInfo::none(),
            /* 0x22 */ OpInfo::none(),
            /* 0x23 */ OpInfo::none(),
            /* 0x24 */ OpInfo::none(),
            /* 0x25 */ OpInfo::none(),
            /* 0x26 */ OpInfo::none(),
            /* 0x27 */ OpInfo::none(),
            /* 0x28 */ OpInfo::none(),
            /* 0x29 */ OpInfo::none(),
            /* 0x2A */ OpInfo::none(),
            /* 0x2B */ OpInfo::none(),
            /* 0x2C */ OpInfo::none(),
            /* 0x2D */ OpInfo::none(),
            /* 0x2E */ OpInfo::none(),
            /* 0x2F */ OpInfo::none(),
            /* 0x30  ADDRESS */ OpInfo::gas(gas::BASE),
            /* 0x31  BALANCE */ OpInfo::dynamic_gas(),
            /* 0x32  ORIGIN */ OpInfo::gas(gas::BASE),
            /* 0x33  CALLER */ OpInfo::gas(gas::BASE),
            /* 0x34  CALLVALUE */ OpInfo::gas(gas::BASE),
            /* 0x35  CALLDATALOAD */ OpInfo::gas(gas::VERYLOW),
            /* 0x36  CALLDATASIZE */ OpInfo::gas(gas::BASE),
            /* 0x37  CALLDATACOPY */ OpInfo::dynamic_gas(),
            /* 0x38  CODESIZE */ OpInfo::gas(gas::BASE),
            /* 0x39  CODECOPY */ OpInfo::dynamic_gas(),
            /* 0x3A  GASPRICE */ OpInfo::gas(gas::BASE),
            /* 0x3B  EXTCODESIZE */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::BERLIN) {
                gas::WARM_STORAGE_READ_COST // add only part of gas
            } else if SpecId::enabled($spec_id, SpecId::TANGERINE) {
                700
            } else {
                20
            }),
            /* 0x3C  EXTCODECOPY */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::BERLIN) {
                gas::WARM_STORAGE_READ_COST // add only part of gas
            } else if SpecId::enabled($spec_id, SpecId::TANGERINE) {
                700
            } else {
                20
            }),
            /* 0x3D  RETURNDATASIZE */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::BYZANTIUM) {
                gas::BASE
            } else {
                0
            }),
            /* 0x3E  RETURNDATACOPY */ OpInfo::dynamic_gas(),
            /* 0x3F  EXTCODEHASH */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::BERLIN) {
                gas::WARM_STORAGE_READ_COST // add only part of gas
            } else if SpecId::enabled($spec_id, SpecId::ISTANBUL) {
                700
            } else if SpecId::enabled($spec_id, SpecId::PETERSBURG) {
                // constantinople
                400
            } else {
                0 // not enabled
            }),
            /* 0x40  BLOCKHASH */ OpInfo::gas(gas::BLOCKHASH),
            /* 0x41  COINBASE */ OpInfo::gas(gas::BASE),
            /* 0x42  TIMESTAMP */ OpInfo::gas(gas::BASE),
            /* 0x43  NUMBER */ OpInfo::gas(gas::BASE),
            /* 0x44  DIFFICULTY */ OpInfo::gas(gas::BASE),
            /* 0x45  GASLIMIT */ OpInfo::gas(gas::BASE),
            /* 0x46  CHAINID */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::ISTANBUL) {
                gas::BASE
            } else {
                0
            }),
            /* 0x47  SELFBALANCE */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::ISTANBUL) {
                gas::LOW
            } else {
                0
            }),
            /* 0x48  BASEFEE */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::LONDON) {
                gas::BASE
            } else {
                0
            }),
            /* 0x49 */ OpInfo::none(),
            /* 0x4A */ OpInfo::none(),
            /* 0x4B */ OpInfo::none(),
            /* 0x4C */ OpInfo::none(),
            /* 0x4D */ OpInfo::none(),
            /* 0x4E */ OpInfo::none(),
            /* 0x4F */ OpInfo::none(),
            /* 0x50  POP */ OpInfo::gas(gas::BASE),
            /* 0x51  MLOAD */ OpInfo::gas(gas::VERYLOW),
            /* 0x52  MSTORE */ OpInfo::gas(gas::VERYLOW),
            /* 0x53  MSTORE8 */ OpInfo::gas(gas::VERYLOW),
            /* 0x54  SLOAD */ OpInfo::dynamic_gas(),
            /* 0x55  SSTORE */ OpInfo::gas_block_end(0),
            /* 0x56  JUMP */ OpInfo::gas_block_end(gas::MID),
            /* 0x57  JUMPI */ OpInfo::gas_block_end(gas::HIGH),
            /* 0x58  PC */ OpInfo::gas(gas::BASE),
            /* 0x59  MSIZE */ OpInfo::gas(gas::BASE),
            /* 0x5A  GAS */ OpInfo::gas_block_end(gas::BASE),
            /* 0x5B  JUMPDEST */
            // gas::JUMPDEST gas is calculated in function call,
            OpInfo::jumpdest(),
            /* 0x5C */ OpInfo::none(),
            /* 0x5D */ OpInfo::none(),
            /* 0x5E  MCOPY */ OpInfo::dynamic_gas(),
            /* 0x5F PUSH0 */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::SHANGHAI) {
                gas::BASE
            } else {
                0
            }),
            /* 0x60  PUSH1 */ OpInfo::push_opcode(),
            /* 0x61  PUSH2 */ OpInfo::push_opcode(),
            /* 0x62  PUSH3 */ OpInfo::push_opcode(),
            /* 0x63  PUSH4 */ OpInfo::push_opcode(),
            /* 0x64  PUSH5 */ OpInfo::push_opcode(),
            /* 0x65  PUSH6 */ OpInfo::push_opcode(),
            /* 0x66  PUSH7 */ OpInfo::push_opcode(),
            /* 0x67  PUSH8 */ OpInfo::push_opcode(),
            /* 0x68  PUSH9 */ OpInfo::push_opcode(),
            /* 0x69  PUSH10 */ OpInfo::push_opcode(),
            /* 0x6A  PUSH11 */ OpInfo::push_opcode(),
            /* 0x6B  PUSH12 */ OpInfo::push_opcode(),
            /* 0x6C  PUSH13 */ OpInfo::push_opcode(),
            /* 0x6D  PUSH14 */ OpInfo::push_opcode(),
            /* 0x6E  PUSH15 */ OpInfo::push_opcode(),
            /* 0x6F  PUSH16 */ OpInfo::push_opcode(),
            /* 0x70  PUSH17 */ OpInfo::push_opcode(),
            /* 0x71  PUSH18 */ OpInfo::push_opcode(),
            /* 0x72  PUSH19 */ OpInfo::push_opcode(),
            /* 0x73  PUSH20 */ OpInfo::push_opcode(),
            /* 0x74  PUSH21 */ OpInfo::push_opcode(),
            /* 0x75  PUSH22 */ OpInfo::push_opcode(),
            /* 0x76  PUSH23 */ OpInfo::push_opcode(),
            /* 0x77  PUSH24 */ OpInfo::push_opcode(),
            /* 0x78  PUSH25 */ OpInfo::push_opcode(),
            /* 0x79  PUSH26 */ OpInfo::push_opcode(),
            /* 0x7A  PUSH27 */ OpInfo::push_opcode(),
            /* 0x7B  PUSH28 */ OpInfo::push_opcode(),
            /* 0x7C  PUSH29 */ OpInfo::push_opcode(),
            /* 0x7D  PUSH30 */ OpInfo::push_opcode(),
            /* 0x7E  PUSH31 */ OpInfo::push_opcode(),
            /* 0x7F  PUSH32 */ OpInfo::push_opcode(),
            /* 0x80  DUP1 */ OpInfo::gas(gas::VERYLOW),
            /* 0x81  DUP2 */ OpInfo::gas(gas::VERYLOW),
            /* 0x82  DUP3 */ OpInfo::gas(gas::VERYLOW),
            /* 0x83  DUP4 */ OpInfo::gas(gas::VERYLOW),
            /* 0x84  DUP5 */ OpInfo::gas(gas::VERYLOW),
            /* 0x85  DUP6 */ OpInfo::gas(gas::VERYLOW),
            /* 0x86  DUP7 */ OpInfo::gas(gas::VERYLOW),
            /* 0x87  DUP8 */ OpInfo::gas(gas::VERYLOW),
            /* 0x88  DUP9 */ OpInfo::gas(gas::VERYLOW),
            /* 0x89  DUP10 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8A  DUP11 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8B  DUP12 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8C  DUP13 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8D  DUP14 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8E  DUP15 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8F  DUP16 */ OpInfo::gas(gas::VERYLOW),
            /* 0x90  SWAP1 */ OpInfo::gas(gas::VERYLOW),
            /* 0x91  SWAP2 */ OpInfo::gas(gas::VERYLOW),
            /* 0x92  SWAP3 */ OpInfo::gas(gas::VERYLOW),
            /* 0x93  SWAP4 */ OpInfo::gas(gas::VERYLOW),
            /* 0x94  SWAP5 */ OpInfo::gas(gas::VERYLOW),
            /* 0x95  SWAP6 */ OpInfo::gas(gas::VERYLOW),
            /* 0x96  SWAP7 */ OpInfo::gas(gas::VERYLOW),
            /* 0x97  SWAP8 */ OpInfo::gas(gas::VERYLOW),
            /* 0x98  SWAP9 */ OpInfo::gas(gas::VERYLOW),
            /* 0x99  SWAP10 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9A  SWAP11 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9B  SWAP12 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9C  SWAP13 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9D  SWAP14 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9E  SWAP15 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9F  SWAP16 */ OpInfo::gas(gas::VERYLOW),
            /* 0xA0  LOG0 */ OpInfo::dynamic_gas(),
            /* 0xA1  LOG1 */ OpInfo::dynamic_gas(),
            /* 0xA2  LOG2 */ OpInfo::dynamic_gas(),
            /* 0xA3  LOG3 */ OpInfo::dynamic_gas(),
            /* 0xA4  LOG4 */ OpInfo::dynamic_gas(),
            /* 0xA5 */ OpInfo::none(),
            /* 0xA6 */ OpInfo::none(),
            /* 0xA7 */ OpInfo::none(),
            /* 0xA8 */ OpInfo::none(),
            /* 0xA9 */ OpInfo::none(),
            /* 0xAA */ OpInfo::none(),
            /* 0xAB */ OpInfo::none(),
            /* 0xAC */ OpInfo::none(),
            /* 0xAD */ OpInfo::none(),
            /* 0xAE */ OpInfo::none(),
            /* 0xAF */ OpInfo::none(),
            /* 0xB0 */ OpInfo::none(),
            /* 0xB1 */ OpInfo::none(),
            /* 0xB2 */ OpInfo::none(),
            /* 0xB3 */ OpInfo::none(),
            /* 0xB4 */ OpInfo::none(),
            /* 0xB5 */ OpInfo::none(),
            /* 0xB6 */ OpInfo::none(),
            /* 0xB7 */ OpInfo::none(),
            /* 0xB8 */ OpInfo::none(),
            /* 0xB9 */ OpInfo::none(),
            /* 0xBA */ OpInfo::none(),
            /* 0xBB */ OpInfo::none(),
            /* 0xBC */ OpInfo::none(),
            /* 0xBD */ OpInfo::none(),
            /* 0xBE */ OpInfo::none(),
            /* 0xBF */ OpInfo::none(),
            /* 0xC0 */ OpInfo::none(),
            /* 0xC1 */ OpInfo::none(),
            /* 0xC2 */ OpInfo::none(),
            /* 0xC3 */ OpInfo::none(),
            /* 0xC4 */ OpInfo::none(),
            /* 0xC5 */ OpInfo::none(),
            /* 0xC6 */ OpInfo::none(),
            /* 0xC7 */ OpInfo::none(),
            /* 0xC8 */ OpInfo::none(),
            /* 0xC9 */ OpInfo::none(),
            /* 0xCA */ OpInfo::none(),
            /* 0xCB */ OpInfo::none(),
            /* 0xCC */ OpInfo::none(),
            /* 0xCD */ OpInfo::none(),
            /* 0xCE */ OpInfo::none(),
            /* 0xCF */ OpInfo::none(),
            /* 0xD0 */ OpInfo::none(),
            /* 0xD1 */ OpInfo::none(),
            /* 0xD2 */ OpInfo::none(),
            /* 0xD3 */ OpInfo::none(),
            /* 0xD4 */ OpInfo::none(),
            /* 0xD5 */ OpInfo::none(),
            /* 0xD6 */ OpInfo::none(),
            /* 0xD7 */ OpInfo::none(),
            /* 0xD8 */ OpInfo::none(),
            /* 0xD9 */ OpInfo::none(),
            /* 0xDA */ OpInfo::none(),
            /* 0xDB */ OpInfo::none(),
            /* 0xDC */ OpInfo::none(),
            /* 0xDD */ OpInfo::none(),
            /* 0xDE */ OpInfo::none(),
            /* 0xDF */ OpInfo::none(),
            /* 0xE0 */ OpInfo::none(),
            /* 0xE1 */ OpInfo::none(),
            /* 0xE2 */ OpInfo::none(),
            /* 0xE3 */ OpInfo::none(),
            /* 0xE4 */ OpInfo::none(),
            /* 0xE5 */ OpInfo::none(),
            /* 0xE6 */ OpInfo::none(),
            /* 0xE7 */ OpInfo::none(),
            /* 0xE8 */ OpInfo::none(),
            /* 0xE9 */ OpInfo::none(),
            /* 0xEA */ OpInfo::none(),
            /* 0xEB */ OpInfo::none(),
            /* 0xEC */ OpInfo::none(),
            /* 0xED */ OpInfo::none(),
            /* 0xEE */ OpInfo::none(),
            /* 0xEF */ OpInfo::none(),
            /* 0xF0  CREATE */ OpInfo::gas_block_end(0),
            /* 0xF1  CALL */ OpInfo::gas_block_end(0),
            /* 0xF2  CALLCODE */ OpInfo::gas_block_end(0),
            /* 0xF3  RETURN */ OpInfo::gas_block_end(0),
            /* 0xF4  DELEGATECALL */ OpInfo::gas_block_end(0),
            /* 0xF5  CREATE2 */ OpInfo::gas_block_end(0),
            /* 0xF6 */ OpInfo::none(),
            /* 0xF7 */ OpInfo::none(),
            /* 0xF8 */ OpInfo::none(),
            /* 0xF9 */ OpInfo::none(),
            /* 0xFA  STATICCALL */ OpInfo::gas_block_end(0),
            /* 0xFB */ OpInfo::none(),
            /* 0xFC */ OpInfo::none(),
            /* 0xFD  REVERT */ OpInfo::gas_block_end(0),
            /* 0xFE  INVALID */ OpInfo::gas_block_end(0),
            /* 0xFF  SELFDESTRUCT */ OpInfo::gas_block_end(0),
        ];
    };
}

pub const fn spec_opcode_gas(spec_id: SpecId) -> &'static [OpInfo; 256] {
    match spec_id {
        SpecId::FRONTIER => {
            gas_opcodee!(FRONTIER, SpecId::FRONTIER);
            FRONTIER
        }
        SpecId::FRONTIER_THAWING => {
            gas_opcodee!(FRONTIER_THAWING, SpecId::FRONTIER_THAWING);
            FRONTIER_THAWING
        }
        SpecId::HOMESTEAD => {
            gas_opcodee!(HOMESTEAD, SpecId::HOMESTEAD);
            HOMESTEAD
        }
        SpecId::DAO_FORK => {
            gas_opcodee!(DAO_FORK, SpecId::DAO_FORK);
            DAO_FORK
        }
        SpecId::TANGERINE => {
            gas_opcodee!(TANGERINE, SpecId::TANGERINE);
            TANGERINE
        }
        SpecId::SPURIOUS_DRAGON => {
            gas_opcodee!(SPURIOUS_DRAGON, SpecId::SPURIOUS_DRAGON);
            SPURIOUS_DRAGON
        }
        SpecId::BYZANTIUM => {
            gas_opcodee!(BYZANTIUM, SpecId::BYZANTIUM);
            BYZANTIUM
        }
        SpecId::CONSTANTINOPLE => {
            gas_opcodee!(CONSTANTINOPLE, SpecId::CONSTANTINOPLE);
            CONSTANTINOPLE
        }
        SpecId::PETERSBURG => {
            gas_opcodee!(PETERSBURG, SpecId::PETERSBURG);
            PETERSBURG
        }
        SpecId::ISTANBUL => {
            gas_opcodee!(ISTANBUL, SpecId::ISTANBUL);
            ISTANBUL
        }
        SpecId::MUIR_GLACIER => {
            gas_opcodee!(MUIRGLACIER, SpecId::MUIR_GLACIER);
            MUIRGLACIER
        }
        SpecId::BERLIN => {
            gas_opcodee!(BERLIN, SpecId::BERLIN);
            BERLIN
        }
        SpecId::LONDON => {
            gas_opcodee!(LONDON, SpecId::LONDON);
            LONDON
        }
        SpecId::ARROW_GLACIER => {
            gas_opcodee!(ARROW_GLACIER, SpecId::ARROW_GLACIER);
            ARROW_GLACIER
        }
        SpecId::GRAY_GLACIER => {
            gas_opcodee!(GRAY_GLACIER, SpecId::GRAY_GLACIER);
            GRAY_GLACIER
        }
        SpecId::MERGE => {
            gas_opcodee!(MERGE, SpecId::MERGE);
            MERGE
        }
        SpecId::SHANGHAI => {
            gas_opcodee!(SHANGAI, SpecId::SHANGHAI);
            SHANGAI
        }
        SpecId::CANCUN => {
            gas_opcodee!(CANCUN, SpecId::CANCUN);
            CANCUN
        }
        SpecId::LATEST => {
            gas_opcodee!(LATEST, SpecId::LATEST);
            LATEST
        }
    }
}
