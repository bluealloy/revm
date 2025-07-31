//! EVM opcode definitions and utilities. It contains opcode information and utilities to work with opcodes.

#[cfg(feature = "parse")]
pub mod parse;

use core::{fmt, ptr::NonNull};

/// An EVM opcode
///
/// This is always a valid opcode, as declared in the [`opcode`][self] module or the
/// [`OPCODE_INFO`] constant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OpCode(u8);

impl fmt::Display for OpCode {
    /// Formats the opcode as a string
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.get();
        if let Some(val) = OPCODE_INFO[n as usize] {
            f.write_str(val.name())
        } else {
            write!(f, "UNKNOWN(0x{n:02X})")
        }
    }
}

impl OpCode {
    /// Instantiates a new opcode from a u8.
    ///
    /// Returns None if the opcode is not valid.
    #[inline]
    pub const fn new(opcode: u8) -> Option<Self> {
        match OPCODE_INFO[opcode as usize] {
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
    #[inline]
    pub const fn is_jumpdest_by_op(opcode: u8) -> bool {
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
    #[inline]
    pub const fn is_jump_by_op(opcode: u8) -> bool {
        if let Some(opcode) = Self::new(opcode) {
            opcode.is_jump()
        } else {
            false
        }
    }

    /// Returns true if the opcode is a `PUSH` instruction.
    #[inline]
    pub const fn is_push(self) -> bool {
        self.0 >= PUSH1 && self.0 <= PUSH32
    }

    /// Takes a u8 and returns true if it is a push instruction.
    #[inline]
    pub fn is_push_by_op(opcode: u8) -> bool {
        if let Some(opcode) = Self::new(opcode) {
            opcode.is_push()
        } else {
            false
        }
    }

    /// Instantiates a new opcode from a u8 without checking if it is valid.
    ///
    /// # Safety
    ///
    /// All code using `Opcode` values assume that they are valid opcodes, so providing an invalid
    /// opcode may cause undefined behavior.
    #[inline]
    pub unsafe fn new_unchecked(opcode: u8) -> Self {
        Self(opcode)
    }

    /// Returns the opcode as a string. This is the inverse of [`parse`](Self::parse).
    #[doc(alias = "name")]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        self.info().name()
    }

    /// Returns the opcode name.
    #[inline]
    pub const fn name_by_op(opcode: u8) -> &'static str {
        if let Some(opcode) = Self::new(opcode) {
            opcode.as_str()
        } else {
            "Unknown"
        }
    }

    /// Returns the number of input stack elements.
    #[inline]
    pub const fn inputs(&self) -> u8 {
        self.info().inputs()
    }

    /// Returns the number of output stack elements.
    #[inline]
    pub const fn outputs(&self) -> u8 {
        self.info().outputs()
    }

    /// Calculates the difference between the number of input and output stack elements.
    #[inline]
    pub const fn io_diff(&self) -> i16 {
        self.info().io_diff()
    }

    /// Returns the opcode information for the given opcode.
    /// Check [OpCodeInfo] for more information.
    #[inline]
    pub const fn info_by_op(opcode: u8) -> Option<OpCodeInfo> {
        if let Some(opcode) = Self::new(opcode) {
            Some(opcode.info())
        } else {
            None
        }
    }

    /// Returns the opcode as a usize.
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Returns the opcode information.
    #[inline]
    pub const fn info(&self) -> OpCodeInfo {
        if let Some(t) = OPCODE_INFO[self.0 as usize] {
            t
        } else {
            panic!("opcode not found")
        }
    }

    /// Returns the number of both input and output stack elements.
    ///
    /// Can be slightly faster that calling `inputs` and `outputs` separately.
    pub const fn input_output(&self) -> (u8, u8) {
        let info = self.info();
        (info.inputs, info.outputs)
    }

    /// Returns the opcode as a u8.
    #[inline]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns true if the opcode modifies memory.
    ///
    /// <https://bluealloy.github.io/revm/crates/interpreter/memory.html#opcodes>
    ///
    /// <https://github.com/crytic/evm-opcodes>
    #[inline]
    pub const fn modifies_memory(&self) -> bool {
        matches!(
            *self,
            OpCode::EXTCODECOPY
                | OpCode::MLOAD
                | OpCode::MSTORE
                | OpCode::MSTORE8
                | OpCode::MCOPY
                | OpCode::CODECOPY
                | OpCode::CALLDATACOPY
                | OpCode::RETURNDATACOPY
                | OpCode::CALL
                | OpCode::CALLCODE
                | OpCode::DELEGATECALL
                | OpCode::STATICCALL
        )
    }
}

impl PartialEq<u8> for OpCode {
    fn eq(&self, other: &u8) -> bool {
        self.get().eq(other)
    }
}

/// Information about opcode, such as name, and stack inputs and outputs
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpCodeInfo {
    /// Invariant: `(name_ptr, name_len)` is a [`&'static str`][str].
    ///
    /// It is a shorted variant of [`str`] as
    /// the name length is always less than 256 characters.
    name_ptr: NonNull<u8>,
    name_len: u8,
    /// Stack inputs
    inputs: u8,
    /// Stack outputs
    outputs: u8,
    /// Number of intermediate bytes
    ///
    /// RJUMPV is a special case where the bytes len depends on bytecode value,
    /// for RJUMV size will be set to one byte as it is the minimum immediate size.
    immediate_size: u8,
    /// If the opcode stops execution. aka STOP, RETURN, ..
    terminating: bool,
}

// SAFETY: The `NonNull` is just a `&'static str`.
unsafe impl Send for OpCodeInfo {}
unsafe impl Sync for OpCodeInfo {}

impl fmt::Debug for OpCodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpCodeInfo")
            .field("name", &self.name())
            .field("inputs", &self.inputs())
            .field("outputs", &self.outputs())
            .field("terminating", &self.is_terminating())
            .field("immediate_size", &self.immediate_size())
            .finish()
    }
}

impl OpCodeInfo {
    /// Creates a new opcode info with the given name and default values.
    pub const fn new(name: &'static str) -> Self {
        assert!(name.len() < 256, "opcode name is too long");
        Self {
            name_ptr: unsafe { NonNull::new_unchecked(name.as_ptr().cast_mut()) },
            name_len: name.len() as u8,
            inputs: 0,
            outputs: 0,
            terminating: false,
            immediate_size: 0,
        }
    }

    /// Returns the opcode name.
    #[inline]
    pub const fn name(&self) -> &'static str {
        // SAFETY: `self.name_*` can only be initialized with a valid `&'static str`.
        unsafe {
            let slice = std::slice::from_raw_parts(self.name_ptr.as_ptr(), self.name_len as usize);
            core::str::from_utf8_unchecked(slice)
        }
    }

    /// Calculates the difference between the number of input and output stack elements.
    #[inline]
    pub const fn io_diff(&self) -> i16 {
        self.outputs as i16 - self.inputs as i16
    }

    /// Returns the number of input stack elements.
    #[inline]
    pub const fn inputs(&self) -> u8 {
        self.inputs
    }

    /// Returns the number of output stack elements.
    #[inline]
    pub const fn outputs(&self) -> u8 {
        self.outputs
    }

    /// Returns whether this opcode terminates execution, e.g. `STOP`, `RETURN`, etc.
    #[inline]
    pub const fn is_terminating(&self) -> bool {
        self.terminating
    }

    /// Returns the size of the immediate value in bytes.
    #[inline]
    pub const fn immediate_size(&self) -> u8 {
        self.immediate_size
    }
}

/// Used for [`OPCODE_INFO`] to set the immediate bytes number in the [`OpCodeInfo`].
///
/// RJUMPV is special case where the bytes len is depending on bytecode value,
/// for RJUMPV size will be set to one byte while minimum is two.
#[inline]
pub const fn immediate_size(mut op: OpCodeInfo, n: u8) -> OpCodeInfo {
    op.immediate_size = n;
    op
}

/// Use for [`OPCODE_INFO`] to set the terminating flag to true in the [`OpCodeInfo`].
#[inline]
pub const fn terminating(mut op: OpCodeInfo) -> OpCodeInfo {
    op.terminating = true;
    op
}

/// Use for [`OPCODE_INFO`] to sets the number of stack inputs and outputs in the [`OpCodeInfo`].
#[inline]
pub const fn stack_io(mut op: OpCodeInfo, inputs: u8, outputs: u8) -> OpCodeInfo {
    op.inputs = inputs;
    op.outputs = outputs;
    op
}

/// Alias for the [`JUMPDEST`] opcode
pub const NOP: u8 = JUMPDEST;

/// Created all opcodes constants and two maps:
///  * `OPCODE_INFO` maps opcode number to the opcode info
///  * `NAME_TO_OPCODE` that maps opcode name to the opcode number.
macro_rules! opcodes {
    ($($val:literal => $name:ident => $($modifier:ident $(( $($modifier_arg:expr),* ))?),*);* $(;)?) => {
        // Constants for each opcode. This also takes care of duplicate names.
        $(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($name),"\") opcode.")]
            pub const $name: u8 = $val;
        )*
        impl OpCode {$(
            #[doc = concat!("The `", stringify!($val), "` (\"", stringify!($name),"\") opcode.")]
            pub const $name: Self = Self($val);
        )*}

        /// Maps each opcode to its info.
        pub static OPCODE_INFO: [Option<OpCodeInfo>; 256] = {
            let mut map = [None; 256];
            let mut prev: u8 = 0;
            $(
                let val: u8 = $val;
                assert!(val == 0 || val > prev, "opcodes must be sorted in ascending order");
                prev = val;
                let info = OpCodeInfo::new(stringify!($name));
                $(
                let info = $modifier(info, $($($modifier_arg),*)?);
                )*
                map[$val] = Some(info);
            )*
            let _ = prev;
            map
        };


        /// Maps each name to its opcode.
        #[cfg(feature = "parse")]
        pub(crate) static NAME_TO_OPCODE: phf::Map<&'static str, OpCode> = stringify_with_cb! { phf_map_cb; $($name)* };
    };
}

/// Callback for creating a [`phf`] map with `stringify_with_cb`.
#[cfg(feature = "parse")]
macro_rules! phf_map_cb {
    ($(#[doc = $s:literal] $id:ident)*) => {
        phf::phf_map! {
            $($s => OpCode::$id),*
        }
    };
}

/// Stringifies identifiers with `paste` so that they are available as literals.
///
/// This doesn't work with [`stringify!`] because it cannot be expanded inside of another macro.
#[cfg(feature = "parse")]
macro_rules! stringify_with_cb {
    ($callback:ident; $($id:ident)*) => { paste::paste! {
        $callback! { $(#[doc = "" $id ""] $id)* }
    }};
}

// When adding new opcodes:
// 1. add the opcode to the list below; make sure it's sorted by opcode value
// 2. implement the opcode in the corresponding module;
//    the function signature must be the exact same as the others
opcodes! {
    0x00 => STOP     => stack_io(0, 0), terminating;
    0x01 => ADD      => stack_io(2, 1);
    0x02 => MUL      => stack_io(2, 1);
    0x03 => SUB      => stack_io(2, 1);
    0x04 => DIV      => stack_io(2, 1);
    0x05 => SDIV     => stack_io(2, 1);
    0x06 => MOD      => stack_io(2, 1);
    0x07 => SMOD     => stack_io(2, 1);
    0x08 => ADDMOD   => stack_io(3, 1);
    0x09 => MULMOD   => stack_io(3, 1);
    0x0A => EXP      => stack_io(2, 1);
    0x0B => SIGNEXTEND => stack_io(2, 1);
    // 0x0C
    // 0x0D
    // 0x0E
    // 0x0F
    0x10 => LT   => stack_io(2, 1);
    0x11 => GT   => stack_io(2, 1);
    0x12 => SLT  => stack_io(2, 1);
    0x13 => SGT  => stack_io(2, 1);
    0x14 => EQ   => stack_io(2, 1);
    0x15 => ISZERO => stack_io(1, 1);
    0x16 => AND  => stack_io(2, 1);
    0x17 => OR   => stack_io(2, 1);
    0x18 => XOR  => stack_io(2, 1);
    0x19 => NOT  => stack_io(1, 1);
    0x1A => BYTE => stack_io(2, 1);
    0x1B => SHL  => stack_io(2, 1);
    0x1C => SHR  => stack_io(2, 1);
    0x1D => SAR  => stack_io(2, 1);
    0x1E => CLZ => stack_io(1, 1);
    // 0x1F
    0x20 => KECCAK256 => stack_io(2, 1);
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
    0x30 => ADDRESS    => stack_io(0, 1);
    0x31 => BALANCE    => stack_io(1, 1);
    0x32 => ORIGIN     => stack_io(0, 1);
    0x33 => CALLER     => stack_io(0, 1);
    0x34 => CALLVALUE  => stack_io(0, 1);
    0x35 => CALLDATALOAD => stack_io(1, 1);
    0x36 => CALLDATASIZE => stack_io(0, 1);
    0x37 => CALLDATACOPY => stack_io(3, 0);
    0x38 => CODESIZE   => stack_io(0, 1);
    0x39 => CODECOPY   => stack_io(3, 0);

    0x3A => GASPRICE     => stack_io(0, 1);
    0x3B => EXTCODESIZE  => stack_io(1, 1);
    0x3C => EXTCODECOPY  => stack_io(4, 0);
    0x3D => RETURNDATASIZE => stack_io(0, 1);
    0x3E => RETURNDATACOPY => stack_io(3, 0);
    0x3F => EXTCODEHASH  => stack_io(1, 1);
    0x40 => BLOCKHASH    => stack_io(1, 1);
    0x41 => COINBASE     => stack_io(0, 1);
    0x42 => TIMESTAMP    => stack_io(0, 1);
    0x43 => NUMBER       => stack_io(0, 1);
    0x44 => DIFFICULTY   => stack_io(0, 1);
    0x45 => GASLIMIT     => stack_io(0, 1);
    0x46 => CHAINID      => stack_io(0, 1);
    0x47 => SELFBALANCE  => stack_io(0, 1);
    0x48 => BASEFEE      => stack_io(0, 1);
    0x49 => BLOBHASH     => stack_io(1, 1);
    0x4A => BLOBBASEFEE  => stack_io(0, 1);
    // 0x4B
    // 0x4C
    // 0x4D
    // 0x4E
    // 0x4F
    0x50 => POP      => stack_io(1, 0);
    0x51 => MLOAD    => stack_io(1, 1);
    0x52 => MSTORE   => stack_io(2, 0);
    0x53 => MSTORE8  => stack_io(2, 0);
    0x54 => SLOAD    => stack_io(1, 1);
    0x55 => SSTORE   => stack_io(2, 0);
    0x56 => JUMP     => stack_io(1, 0);
    0x57 => JUMPI    => stack_io(2, 0);
    0x58 => PC       => stack_io(0, 1);
    0x59 => MSIZE    => stack_io(0, 1);
    0x5A => GAS      => stack_io(0, 1);
    0x5B => JUMPDEST => stack_io(0, 0);
    0x5C => TLOAD    => stack_io(1, 1);
    0x5D => TSTORE   => stack_io(2, 0);
    0x5E => MCOPY    => stack_io(3, 0);

    0x5F => PUSH0  => stack_io(0, 1);
    0x60 => PUSH1  => stack_io(0, 1), immediate_size(1);
    0x61 => PUSH2  => stack_io(0, 1), immediate_size(2);
    0x62 => PUSH3  => stack_io(0, 1), immediate_size(3);
    0x63 => PUSH4  => stack_io(0, 1), immediate_size(4);
    0x64 => PUSH5  => stack_io(0, 1), immediate_size(5);
    0x65 => PUSH6  => stack_io(0, 1), immediate_size(6);
    0x66 => PUSH7  => stack_io(0, 1), immediate_size(7);
    0x67 => PUSH8  => stack_io(0, 1), immediate_size(8);
    0x68 => PUSH9  => stack_io(0, 1), immediate_size(9);
    0x69 => PUSH10 => stack_io(0, 1), immediate_size(10);
    0x6A => PUSH11 => stack_io(0, 1), immediate_size(11);
    0x6B => PUSH12 => stack_io(0, 1), immediate_size(12);
    0x6C => PUSH13 => stack_io(0, 1), immediate_size(13);
    0x6D => PUSH14 => stack_io(0, 1), immediate_size(14);
    0x6E => PUSH15 => stack_io(0, 1), immediate_size(15);
    0x6F => PUSH16 => stack_io(0, 1), immediate_size(16);
    0x70 => PUSH17 => stack_io(0, 1), immediate_size(17);
    0x71 => PUSH18 => stack_io(0, 1), immediate_size(18);
    0x72 => PUSH19 => stack_io(0, 1), immediate_size(19);
    0x73 => PUSH20 => stack_io(0, 1), immediate_size(20);
    0x74 => PUSH21 => stack_io(0, 1), immediate_size(21);
    0x75 => PUSH22 => stack_io(0, 1), immediate_size(22);
    0x76 => PUSH23 => stack_io(0, 1), immediate_size(23);
    0x77 => PUSH24 => stack_io(0, 1), immediate_size(24);
    0x78 => PUSH25 => stack_io(0, 1), immediate_size(25);
    0x79 => PUSH26 => stack_io(0, 1), immediate_size(26);
    0x7A => PUSH27 => stack_io(0, 1), immediate_size(27);
    0x7B => PUSH28 => stack_io(0, 1), immediate_size(28);
    0x7C => PUSH29 => stack_io(0, 1), immediate_size(29);
    0x7D => PUSH30 => stack_io(0, 1), immediate_size(30);
    0x7E => PUSH31 => stack_io(0, 1), immediate_size(31);
    0x7F => PUSH32 => stack_io(0, 1), immediate_size(32);

    0x80 => DUP1  => stack_io(1, 2);
    0x81 => DUP2  => stack_io(2, 3);
    0x82 => DUP3  => stack_io(3, 4);
    0x83 => DUP4  => stack_io(4, 5);
    0x84 => DUP5  => stack_io(5, 6);
    0x85 => DUP6  => stack_io(6, 7);
    0x86 => DUP7  => stack_io(7, 8);
    0x87 => DUP8  => stack_io(8, 9);
    0x88 => DUP9  => stack_io(9, 10);
    0x89 => DUP10 => stack_io(10, 11);
    0x8A => DUP11 => stack_io(11, 12);
    0x8B => DUP12 => stack_io(12, 13);
    0x8C => DUP13 => stack_io(13, 14);
    0x8D => DUP14 => stack_io(14, 15);
    0x8E => DUP15 => stack_io(15, 16);
    0x8F => DUP16 => stack_io(16, 17);

    0x90 => SWAP1  => stack_io(2, 2);
    0x91 => SWAP2  => stack_io(3, 3);
    0x92 => SWAP3  => stack_io(4, 4);
    0x93 => SWAP4  => stack_io(5, 5);
    0x94 => SWAP5  => stack_io(6, 6);
    0x95 => SWAP6  => stack_io(7, 7);
    0x96 => SWAP7  => stack_io(8, 8);
    0x97 => SWAP8  => stack_io(9, 9);
    0x98 => SWAP9  => stack_io(10, 10);
    0x99 => SWAP10 => stack_io(11, 11);
    0x9A => SWAP11 => stack_io(12, 12);
    0x9B => SWAP12 => stack_io(13, 13);
    0x9C => SWAP13 => stack_io(14, 14);
    0x9D => SWAP14 => stack_io(15, 15);
    0x9E => SWAP15 => stack_io(16, 16);
    0x9F => SWAP16 => stack_io(17, 17);

    0xA0 => LOG0 => stack_io(2, 0);
    0xA1 => LOG1 => stack_io(3, 0);
    0xA2 => LOG2 => stack_io(4, 0);
    0xA3 => LOG3 => stack_io(5, 0);
    0xA4 => LOG4 => stack_io(6, 0);
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
    0xF0 => CREATE       => stack_io(3, 1);
    0xF1 => CALL         => stack_io(7, 1);
    0xF2 => CALLCODE     => stack_io(7, 1);
    0xF3 => RETURN       => stack_io(2, 0), terminating;
    0xF4 => DELEGATECALL => stack_io(6, 1);
    0xF5 => CREATE2      => stack_io(4, 1);
    // 0xF6
    // 0xF7
    // 0xF8
    // 0xF9
    0xFA => STATICCALL      => stack_io(6, 1);
    // 0xFB
    // 0xFC
    0xFD => REVERT       => stack_io(2, 0), terminating;
    0xFE => INVALID      => stack_io(0, 0), terminating;
    0xFF => SELFDESTRUCT => stack_io(1, 0), terminating;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode() {
        let opcode = OpCode::new(0x00).unwrap();
        assert!(!opcode.is_jumpdest());
        assert!(!opcode.is_jump());
        assert!(!opcode.is_push());
        assert_eq!(opcode.as_str(), "STOP");
        assert_eq!(opcode.get(), 0x00);
    }

    #[test]
    fn test_immediate_size() {
        let mut expected = [0u8; 256];
        // PUSH opcodes
        for push in PUSH1..=PUSH32 {
            expected[push as usize] = push - PUSH1 + 1;
        }

        for (i, opcode) in OPCODE_INFO.iter().enumerate() {
            if let Some(opcode) = opcode {
                assert_eq!(
                    opcode.immediate_size(),
                    expected[i],
                    "immediate_size check failed for {opcode:#?}",
                );
            }
        }
    }

    #[test]
    fn test_enabled_opcodes() {
        // List obtained from https://eips.ethereum.org/EIPS/eip-3670
        let opcodes = [
            0x10..=0x1d,
            0x20..=0x20,
            0x30..=0x3f,
            0x40..=0x48,
            0x50..=0x5b,
            0x54..=0x5f,
            0x60..=0x6f,
            0x70..=0x7f,
            0x80..=0x8f,
            0x90..=0x9f,
            0xa0..=0xa4,
            0xf0..=0xf5,
            0xfa..=0xfa,
            0xfd..=0xfd,
            //0xfe,
            0xff..=0xff,
        ];
        for i in opcodes {
            for opcode in i {
                OpCode::new(opcode).expect("Opcode should be valid and enabled");
            }
        }
    }

    #[test]
    fn count_opcodes() {
        let mut opcode_num = 0;
        for _ in OPCODE_INFO.into_iter().flatten() {
            opcode_num += 1;
        }
        assert_eq!(opcode_num, 150);
    }

    #[test]
    fn test_terminating_opcodes() {
        let terminating = [REVERT, RETURN, INVALID, SELFDESTRUCT, STOP];
        let mut opcodes = [false; 256];
        for terminating in terminating.iter() {
            opcodes[*terminating as usize] = true;
        }

        for (i, opcode) in OPCODE_INFO.into_iter().enumerate() {
            assert_eq!(
                opcode.map(|opcode| opcode.terminating).unwrap_or_default(),
                opcodes[i],
                "Opcode {opcode:?} terminating check failed."
            );
        }
    }

    #[test]
    #[cfg(feature = "parse")]
    fn test_parsing() {
        for i in 0..=u8::MAX {
            if let Some(op) = OpCode::new(i) {
                assert_eq!(OpCode::parse(op.as_str()), Some(op));
            }
        }
    }

    #[test]
    fn test_opcode_display() {
        // Test valid opcodes
        let stop = OpCode::new(0x00).unwrap();
        assert_eq!(format!("{stop}"), "STOP");

        let add = OpCode::new(0x01).unwrap();
        assert_eq!(format!("{add}"), "ADD");

        let push1 = OpCode::new(0x60).unwrap();
        assert_eq!(format!("{push1}"), "PUSH1");
    }

    #[test]
    fn test_is_jumpdest() {
        let jumpdest = OpCode::new(JUMPDEST).unwrap();
        assert!(jumpdest.is_jumpdest());

        let stop = OpCode::new(0x00).unwrap();
        assert!(!stop.is_jumpdest());
    }

    #[test]
    fn test_is_jumpdest_by_op() {
        assert!(OpCode::is_jumpdest_by_op(JUMPDEST));
        assert!(!OpCode::is_jumpdest_by_op(0x00));
        // Test invalid opcode
        assert!(!OpCode::is_jumpdest_by_op(0xEF));
    }

    #[test]
    fn test_is_jump() {
        let jump = OpCode::new(JUMP).unwrap();
        assert!(jump.is_jump());

        let stop = OpCode::new(0x00).unwrap();
        assert!(!stop.is_jump());
    }

    #[test]
    fn test_is_jump_by_op() {
        assert!(OpCode::is_jump_by_op(JUMP));
        assert!(!OpCode::is_jump_by_op(0x00));
        // Test invalid opcode
        assert!(!OpCode::is_jump_by_op(0xEF));
    }

    #[test]
    fn test_is_push() {
        for i in PUSH1..=PUSH32 {
            let push = OpCode::new(i).unwrap();
            assert!(push.is_push(), "PUSH{} should be push", i - PUSH1 + 1);
        }

        let stop = OpCode::new(0x00).unwrap();
        assert!(!stop.is_push());
    }

    #[test]
    fn test_is_push_by_op() {
        for i in PUSH1..=PUSH32 {
            assert!(OpCode::is_push_by_op(i));
        }
        assert!(!OpCode::is_push_by_op(0x00));
        // Test invalid opcode
        assert!(!OpCode::is_push_by_op(0xEF));
    }

    #[test]
    fn test_new_unchecked() {
        // Test that unchecked creation works
        unsafe {
            let opcode = OpCode::new_unchecked(0x00);
            assert_eq!(opcode.get(), 0x00);

            // Even invalid opcodes can be created
            let invalid = OpCode::new_unchecked(0xEF);
            assert_eq!(invalid.get(), 0xEF);
        }
    }

    #[test]
    fn test_name_by_op() {
        assert_eq!(OpCode::name_by_op(0x00), "STOP");
        assert_eq!(OpCode::name_by_op(0x01), "ADD");
        // Test invalid opcode
        assert_eq!(OpCode::name_by_op(0xEF), "Unknown");
    }

    #[test]
    fn test_inputs_outputs() {
        let add = OpCode::new(0x01).unwrap(); // ADD
        assert_eq!(add.inputs(), 2);
        assert_eq!(add.outputs(), 1);

        let stop = OpCode::new(0x00).unwrap(); // STOP
        assert_eq!(stop.inputs(), 0);
        assert_eq!(stop.outputs(), 0);
    }

    #[test]
    fn test_io_diff() {
        let add = OpCode::new(0x01).unwrap(); // ADD: 2 inputs, 1 output
        assert_eq!(add.io_diff(), -1);

        let push1 = OpCode::new(0x60).unwrap(); // PUSH1: 0 inputs, 1 output
        assert_eq!(push1.io_diff(), 1);

        let stop = OpCode::new(0x00).unwrap(); // STOP: 0 inputs, 0 outputs
        assert_eq!(stop.io_diff(), 0);
    }

    #[test]
    fn test_info_by_op() {
        let info = OpCode::info_by_op(0x00).unwrap();
        assert_eq!(info.name(), "STOP");

        // Test invalid opcode
        assert!(OpCode::info_by_op(0xEF).is_none());
    }

    #[test]
    fn test_as_usize() {
        let stop = OpCode::new(0x00).unwrap();
        assert_eq!(stop.as_usize(), 0);

        let invalid = OpCode::new(0xFE).unwrap();
        assert_eq!(invalid.as_usize(), 254);
    }

    #[test]
    fn test_input_output() {
        let add = OpCode::new(0x01).unwrap(); // ADD
        assert_eq!(add.input_output(), (2, 1));

        let push1 = OpCode::new(0x60).unwrap(); // PUSH1
        assert_eq!(push1.input_output(), (0, 1));
    }

    #[test]
    fn test_modifies_memory() {
        // Test opcodes that modify memory
        assert!(OpCode::new(EXTCODECOPY).unwrap().modifies_memory());
        assert!(OpCode::new(MLOAD).unwrap().modifies_memory());
        assert!(OpCode::new(MSTORE).unwrap().modifies_memory());
        assert!(OpCode::new(MSTORE8).unwrap().modifies_memory());
        assert!(OpCode::new(MCOPY).unwrap().modifies_memory());
        assert!(OpCode::new(CODECOPY).unwrap().modifies_memory());
        assert!(OpCode::new(CALLDATACOPY).unwrap().modifies_memory());
        assert!(OpCode::new(RETURNDATACOPY).unwrap().modifies_memory());
        assert!(OpCode::new(CALL).unwrap().modifies_memory());
        assert!(OpCode::new(CALLCODE).unwrap().modifies_memory());
        assert!(OpCode::new(DELEGATECALL).unwrap().modifies_memory());
        assert!(OpCode::new(STATICCALL).unwrap().modifies_memory());

        // Test opcodes that don't modify memory
        assert!(!OpCode::new(0x00).unwrap().modifies_memory()); // STOP
        assert!(!OpCode::new(0x01).unwrap().modifies_memory()); // ADD
        assert!(!OpCode::new(0x60).unwrap().modifies_memory()); // PUSH1
    }

    #[test]
    fn test_partial_eq_u8() {
        let stop = OpCode::new(0x00).unwrap();
        assert!(stop == 0x00);
        assert!((stop != 0x01));
    }

    #[test]
    fn test_opcode_info_new() {
        let info = OpCodeInfo::new("TEST_OPCODE");
        assert_eq!(info.name(), "TEST_OPCODE");
        assert_eq!(info.inputs(), 0);
        assert_eq!(info.outputs(), 0);
        assert!(!info.is_terminating());
        assert_eq!(info.immediate_size(), 0);
    }

    #[test]
    fn test_opcode_info_methods() {
        // Get info from a real opcode
        let add = OpCode::new(0x01).unwrap();
        let info = add.info();

        assert_eq!(info.name(), "ADD");
        assert_eq!(info.inputs(), 2);
        assert_eq!(info.outputs(), 1);
        assert_eq!(info.io_diff(), -1);
        assert!(!info.is_terminating());
        assert_eq!(info.immediate_size(), 0);
    }

    #[test]
    fn test_opcode_info_debug() {
        let info = OpCodeInfo::new("TEST");
        let debug_str = format!("{info:?}");
        assert!(debug_str.contains("TEST"));
        assert!(debug_str.contains("inputs"));
        assert!(debug_str.contains("outputs"));
        assert!(debug_str.contains("terminating"));
        assert!(debug_str.contains("immediate_size"));
    }

    #[test]
    fn test_invalid_opcode_display() {
        // Create an invalid opcode using unsafe
        let invalid = unsafe { OpCode::new_unchecked(0xEF) };
        let display = format!("{invalid}");
        assert!(display.starts_with("UNKNOWN"));
        assert!(display.contains("0xEF"));
    }

    #[test]
    #[should_panic(expected = "opcode not found")]
    fn test_info_panic_on_invalid() {
        // Create an invalid opcode using unsafe and try to get info
        let invalid = unsafe { OpCode::new_unchecked(0xEF) };
        let _ = invalid.info(); // This should panic
    }

    #[test]
    fn test_all_terminating_opcodes() {
        // Test that terminating opcodes are properly marked
        assert!(OpCode::new(STOP).unwrap().info().is_terminating());
        assert!(OpCode::new(RETURN).unwrap().info().is_terminating());
        assert!(OpCode::new(REVERT).unwrap().info().is_terminating());
        assert!(OpCode::new(INVALID).unwrap().info().is_terminating());
        assert!(OpCode::new(SELFDESTRUCT).unwrap().info().is_terminating());

        // Non-terminating
        assert!(!OpCode::new(ADD).unwrap().info().is_terminating());
        assert!(!OpCode::new(PUSH1).unwrap().info().is_terminating());
    }

    #[test]
    fn test_push_immediate_sizes() {
        for i in 1..=32 {
            let push_op = OpCode::new(PUSH1 + i - 1).unwrap();
            assert_eq!(
                push_op.info().immediate_size(),
                i,
                "PUSH{i} should have immediate size {i}"
            );
        }

        // Non-push opcodes should have 0 immediate size
        assert_eq!(OpCode::new(ADD).unwrap().info().immediate_size(), 0);
        assert_eq!(OpCode::new(STOP).unwrap().info().immediate_size(), 0);
    }

    #[test]
    fn test_stack_io_for_various_opcodes() {
        // Test various opcodes for correct stack I/O
        let test_cases = vec![
            (STOP, 0, 0),
            (ADD, 2, 1),
            (MUL, 2, 1),
            (SUB, 2, 1),
            (DIV, 2, 1),
            (LT, 2, 1),
            (GT, 2, 1),
            (PUSH1, 0, 1),
            (PUSH32, 0, 1),
            (DUP1, 1, 2),
            (DUP16, 16, 17),
            (SWAP1, 2, 2),
            (SWAP16, 17, 17),
            (LOG0, 2, 0),
            (LOG4, 6, 0),
            (CALL, 7, 1),
            (RETURN, 2, 0),
        ];

        for (opcode, expected_inputs, expected_outputs) in test_cases {
            let op = OpCode::new(opcode).unwrap();
            assert_eq!(
                op.inputs(),
                expected_inputs,
                "Opcode 0x{opcode:02x} inputs mismatch"
            );
            assert_eq!(
                op.outputs(),
                expected_outputs,
                "Opcode 0x{opcode:02x} outputs mismatch"
            );
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_opcode_info_serde_trait() {
        // OpCodeInfo should have Send and Sync traits
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OpCodeInfo>();
    }

    #[test]
    fn test_default_opcode() {
        // Test Default trait
        let default_opcode = OpCode::default();
        assert_eq!(default_opcode.get(), 0);
        assert_eq!(default_opcode.as_str(), "STOP");
    }

    #[test]
    fn test_opcode_traits() {
        let op1 = OpCode::new(0x00).unwrap();
        let op2 = OpCode::new(0x00).unwrap();
        let op3 = OpCode::new(0x01).unwrap();

        // Test Clone
        let cloned = op1;
        assert_eq!(op1, cloned);

        // Test Copy
        let copied = op1;
        assert_eq!(op1, copied);

        // Test PartialEq
        assert_eq!(op1, op2);
        assert_ne!(op1, op3);

        // Test Ord
        assert!(op1 < op3);

        // Test Hash
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(op1, "STOP");
        assert_eq!(map.get(&op1), Some(&"STOP"));
    }

    #[test]
    fn test_opcode_info_helpers() {
        // These helper functions are used in the opcode macro but might not be covered

        // Test immediate_size helper
        let info = OpCodeInfo::new("TEST");
        let info_with_size = immediate_size(info, 5);
        assert_eq!(info_with_size.immediate_size(), 5);

        // Test terminating helper
        let info2 = OpCodeInfo::new("TEST2");
        let info_terminating = terminating(info2);
        assert!(info_terminating.is_terminating());

        // Test stack_io helper
        let info3 = OpCodeInfo::new("TEST3");
        let info_io = stack_io(info3, 3, 2);
        assert_eq!(info_io.inputs(), 3);
        assert_eq!(info_io.outputs(), 2);
        assert_eq!(info_io.io_diff(), -1);
    }

    #[test]
    #[should_panic(expected = "opcode name is too long")]
    fn test_opcode_info_name_too_long() {
        // Create a string longer than 255 characters
        let long_name = "a".repeat(256);
        let _ = OpCodeInfo::new(Box::leak(long_name.into_boxed_str()));
    }

    #[test]
    fn test_opcode_info_equality() {
        let info1 = OpCodeInfo::new("TEST");
        let info2 = OpCodeInfo::new("TEST");
        let info3 = OpCodeInfo::new("OTHER");

        // OpCodeInfo doesn't derive PartialEq, but we can test through OpCode
        // Test that two infos with same properties are functionally equivalent
        assert_eq!(info1.name(), info2.name());
        assert_eq!(info1.inputs(), info2.inputs());
        assert_eq!(info1.outputs(), info2.outputs());
        assert_ne!(info1.name(), info3.name());
    }

    #[test]
    fn test_send_sync_traits() {
        // Verify that OpCodeInfo implements Send and Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<OpCodeInfo>();
        assert_sync::<OpCodeInfo>();
    }

    #[test]
    fn test_new_invalid_opcodes() {
        // Test OpCode::new with various invalid opcodes
        assert!(OpCode::new(0x0C).is_none()); // Unused
        assert!(OpCode::new(0x0D).is_none()); // Unused
        assert!(OpCode::new(0x0E).is_none()); // Unused
        assert!(OpCode::new(0x0F).is_none()); // Unused
        assert!(OpCode::new(0x1F).is_none()); // Unused after CLZ
        assert!(OpCode::new(0x21).is_none()); // Unused after KECCAK256
        assert!(OpCode::new(0x2F).is_none()); // Unused
        assert!(OpCode::new(0x4B).is_none()); // Unused after BLOBBASEFEE
        assert!(OpCode::new(0xEF).is_none()); // Reserved for EOF
    }
}
