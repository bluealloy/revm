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
    /// Can be slightly faster than calling `inputs` and `outputs` separately.
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
    /// <https://docs.rs/revm-interpreter/latest/revm_interpreter/instructions/index.html>
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
                | OpCode::KECCAK256
                | OpCode::CODECOPY
                | OpCode::CALLDATACOPY
                | OpCode::RETURNDATACOPY
                | OpCode::CALL
                | OpCode::CALLCODE
                | OpCode::DELEGATECALL
                | OpCode::STATICCALL
                | OpCode::LOG0
                | OpCode::LOG1
                | OpCode::LOG2
                | OpCode::LOG3
                | OpCode::LOG4
                | OpCode::CREATE
                | OpCode::CREATE2
        )
    }

    /// Returns true if the opcode is valid
    #[inline]
    pub const fn is_valid(&self) -> bool {
        OPCODE_INFO[self.0 as usize].is_some()
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
#[inline]
pub const fn immediate_size(mut op: OpCodeInfo, n: u8) -> OpCodeInfo {
    op.immediate_size = n;
    op
}

/// Used for [`OPCODE_INFO`] to set the terminating flag to true in the [`OpCodeInfo`].
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
    ($d:tt $($val:literal => $name:ident => $f:expr => $($modifier:ident $(( $($modifier_arg:expr),* ))?),*);* $(;)?) => {
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

        /// Higher-order macro to iterate over all opcodes with their instruction function paths.
        ///
        /// Invokes `$m` with the provided extra tokens and a list of `(NAME, fn_path),` entries.
        #[macro_export]
        macro_rules! for_each_opcode {
            ([$d ($d extra:tt)*] $d m:path) => {
                $d m! { [$d ($d extra)*]
                    $(
                        ($name, $f),
                    )*
                }
            };
        }

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
opcodes! {$
    0x00 => STOP       => control::stop              => stack_io(0, 0), terminating;
    0x01 => ADD        => arithmetic::add            => stack_io(2, 1);
    0x02 => MUL        => arithmetic::mul            => stack_io(2, 1);
    0x03 => SUB        => arithmetic::sub            => stack_io(2, 1);
    0x04 => DIV        => arithmetic::div            => stack_io(2, 1);
    0x05 => SDIV       => arithmetic::sdiv           => stack_io(2, 1);
    0x06 => MOD        => arithmetic::rem            => stack_io(2, 1);
    0x07 => SMOD       => arithmetic::smod           => stack_io(2, 1);
    0x08 => ADDMOD     => arithmetic::addmod         => stack_io(3, 1);
    0x09 => MULMOD     => arithmetic::mulmod         => stack_io(3, 1);
    0x0A => EXP        => arithmetic::exp            => stack_io(2, 1);
    0x0B => SIGNEXTEND => arithmetic::signextend     => stack_io(2, 1);
    // 0x0C
    // 0x0D
    // 0x0E
    // 0x0F
    0x10 => LT     => bitwise::lt                    => stack_io(2, 1);
    0x11 => GT     => bitwise::gt                    => stack_io(2, 1);
    0x12 => SLT    => bitwise::slt                   => stack_io(2, 1);
    0x13 => SGT    => bitwise::sgt                   => stack_io(2, 1);
    0x14 => EQ     => bitwise::eq                    => stack_io(2, 1);
    0x15 => ISZERO => bitwise::iszero                => stack_io(1, 1);
    0x16 => AND    => bitwise::bitand                => stack_io(2, 1);
    0x17 => OR     => bitwise::bitor                 => stack_io(2, 1);
    0x18 => XOR    => bitwise::bitxor                => stack_io(2, 1);
    0x19 => NOT    => bitwise::not                   => stack_io(1, 1);
    0x1A => BYTE   => bitwise::byte                  => stack_io(2, 1);
    0x1B => SHL    => bitwise::shl                   => stack_io(2, 1);
    0x1C => SHR    => bitwise::shr                   => stack_io(2, 1);
    0x1D => SAR    => bitwise::sar                   => stack_io(2, 1);
    0x1E => CLZ    => bitwise::clz                   => stack_io(1, 1);
    // 0x1F
    0x20 => KECCAK256 => system::keccak256           => stack_io(2, 1);
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
    0x30 => ADDRESS        => system::address        => stack_io(0, 1);
    0x31 => BALANCE        => host::balance          => stack_io(1, 1);
    0x32 => ORIGIN         => tx_info::origin        => stack_io(0, 1);
    0x33 => CALLER         => system::caller         => stack_io(0, 1);
    0x34 => CALLVALUE      => system::callvalue      => stack_io(0, 1);
    0x35 => CALLDATALOAD   => system::calldataload   => stack_io(1, 1);
    0x36 => CALLDATASIZE   => system::calldatasize   => stack_io(0, 1);
    0x37 => CALLDATACOPY   => system::calldatacopy   => stack_io(3, 0);
    0x38 => CODESIZE       => system::codesize       => stack_io(0, 1);
    0x39 => CODECOPY       => system::codecopy       => stack_io(3, 0);

    0x3A => GASPRICE       => tx_info::gasprice      => stack_io(0, 1);
    0x3B => EXTCODESIZE    => host::extcodesize      => stack_io(1, 1);
    0x3C => EXTCODECOPY    => host::extcodecopy      => stack_io(4, 0);
    0x3D => RETURNDATASIZE => system::returndatasize => stack_io(0, 1);
    0x3E => RETURNDATACOPY => system::returndatacopy => stack_io(3, 0);
    0x3F => EXTCODEHASH    => host::extcodehash      => stack_io(1, 1);
    0x40 => BLOCKHASH      => host::blockhash        => stack_io(1, 1);
    0x41 => COINBASE       => block_info::coinbase   => stack_io(0, 1);
    0x42 => TIMESTAMP      => block_info::timestamp  => stack_io(0, 1);
    0x43 => NUMBER         => block_info::block_number => stack_io(0, 1);
    0x44 => DIFFICULTY     => block_info::difficulty  => stack_io(0, 1);
    0x45 => GASLIMIT       => block_info::gaslimit   => stack_io(0, 1);
    0x46 => CHAINID        => block_info::chainid    => stack_io(0, 1);
    0x47 => SELFBALANCE    => host::selfbalance      => stack_io(0, 1);
    0x48 => BASEFEE        => block_info::basefee    => stack_io(0, 1);
    0x49 => BLOBHASH       => tx_info::blob_hash     => stack_io(1, 1);
    0x4A => BLOBBASEFEE    => block_info::blob_basefee => stack_io(0, 1);
    0x4B => SLOTNUM        => block_info::slot_num   => stack_io(0, 1);
    // 0x4C
    // 0x4D
    // 0x4E
    // 0x4F
    0x50 => POP      => stack::pop                   => stack_io(1, 0);
    0x51 => MLOAD    => memory::mload                => stack_io(1, 1);
    0x52 => MSTORE   => memory::mstore               => stack_io(2, 0);
    0x53 => MSTORE8  => memory::mstore8              => stack_io(2, 0);
    0x54 => SLOAD    => host::sload                  => stack_io(1, 1);
    0x55 => SSTORE   => host::sstore                 => stack_io(2, 0);
    0x56 => JUMP     => control::jump                => stack_io(1, 0);
    0x57 => JUMPI    => control::jumpi               => stack_io(2, 0);
    0x58 => PC       => control::pc                  => stack_io(0, 1);
    0x59 => MSIZE    => memory::msize                => stack_io(0, 1);
    0x5A => GAS      => system::gas                  => stack_io(0, 1);
    0x5B => JUMPDEST => control::jumpdest            => stack_io(0, 0);
    0x5C => TLOAD    => host::tload                  => stack_io(1, 1);
    0x5D => TSTORE   => host::tstore                 => stack_io(2, 0);
    0x5E => MCOPY    => memory::mcopy                => stack_io(3, 0);

    0x5F => PUSH0  => stack::push0                   => stack_io(0, 1);
    0x60 => PUSH1  => stack::push::<1, _, _>         => stack_io(0, 1), immediate_size(1);
    0x61 => PUSH2  => stack::push::<2, _, _>         => stack_io(0, 1), immediate_size(2);
    0x62 => PUSH3  => stack::push::<3, _, _>         => stack_io(0, 1), immediate_size(3);
    0x63 => PUSH4  => stack::push::<4, _, _>         => stack_io(0, 1), immediate_size(4);
    0x64 => PUSH5  => stack::push::<5, _, _>         => stack_io(0, 1), immediate_size(5);
    0x65 => PUSH6  => stack::push::<6, _, _>         => stack_io(0, 1), immediate_size(6);
    0x66 => PUSH7  => stack::push::<7, _, _>         => stack_io(0, 1), immediate_size(7);
    0x67 => PUSH8  => stack::push::<8, _, _>         => stack_io(0, 1), immediate_size(8);
    0x68 => PUSH9  => stack::push::<9, _, _>         => stack_io(0, 1), immediate_size(9);
    0x69 => PUSH10 => stack::push::<10, _, _>        => stack_io(0, 1), immediate_size(10);
    0x6A => PUSH11 => stack::push::<11, _, _>        => stack_io(0, 1), immediate_size(11);
    0x6B => PUSH12 => stack::push::<12, _, _>        => stack_io(0, 1), immediate_size(12);
    0x6C => PUSH13 => stack::push::<13, _, _>        => stack_io(0, 1), immediate_size(13);
    0x6D => PUSH14 => stack::push::<14, _, _>        => stack_io(0, 1), immediate_size(14);
    0x6E => PUSH15 => stack::push::<15, _, _>        => stack_io(0, 1), immediate_size(15);
    0x6F => PUSH16 => stack::push::<16, _, _>        => stack_io(0, 1), immediate_size(16);
    0x70 => PUSH17 => stack::push::<17, _, _>        => stack_io(0, 1), immediate_size(17);
    0x71 => PUSH18 => stack::push::<18, _, _>        => stack_io(0, 1), immediate_size(18);
    0x72 => PUSH19 => stack::push::<19, _, _>        => stack_io(0, 1), immediate_size(19);
    0x73 => PUSH20 => stack::push::<20, _, _>        => stack_io(0, 1), immediate_size(20);
    0x74 => PUSH21 => stack::push::<21, _, _>        => stack_io(0, 1), immediate_size(21);
    0x75 => PUSH22 => stack::push::<22, _, _>        => stack_io(0, 1), immediate_size(22);
    0x76 => PUSH23 => stack::push::<23, _, _>        => stack_io(0, 1), immediate_size(23);
    0x77 => PUSH24 => stack::push::<24, _, _>        => stack_io(0, 1), immediate_size(24);
    0x78 => PUSH25 => stack::push::<25, _, _>        => stack_io(0, 1), immediate_size(25);
    0x79 => PUSH26 => stack::push::<26, _, _>        => stack_io(0, 1), immediate_size(26);
    0x7A => PUSH27 => stack::push::<27, _, _>        => stack_io(0, 1), immediate_size(27);
    0x7B => PUSH28 => stack::push::<28, _, _>        => stack_io(0, 1), immediate_size(28);
    0x7C => PUSH29 => stack::push::<29, _, _>        => stack_io(0, 1), immediate_size(29);
    0x7D => PUSH30 => stack::push::<30, _, _>        => stack_io(0, 1), immediate_size(30);
    0x7E => PUSH31 => stack::push::<31, _, _>        => stack_io(0, 1), immediate_size(31);
    0x7F => PUSH32 => stack::push::<32, _, _>        => stack_io(0, 1), immediate_size(32);

    0x80 => DUP1  => stack::dup::<1, _, _>           => stack_io(1, 2);
    0x81 => DUP2  => stack::dup::<2, _, _>           => stack_io(2, 3);
    0x82 => DUP3  => stack::dup::<3, _, _>           => stack_io(3, 4);
    0x83 => DUP4  => stack::dup::<4, _, _>           => stack_io(4, 5);
    0x84 => DUP5  => stack::dup::<5, _, _>           => stack_io(5, 6);
    0x85 => DUP6  => stack::dup::<6, _, _>           => stack_io(6, 7);
    0x86 => DUP7  => stack::dup::<7, _, _>           => stack_io(7, 8);
    0x87 => DUP8  => stack::dup::<8, _, _>           => stack_io(8, 9);
    0x88 => DUP9  => stack::dup::<9, _, _>           => stack_io(9, 10);
    0x89 => DUP10 => stack::dup::<10, _, _>          => stack_io(10, 11);
    0x8A => DUP11 => stack::dup::<11, _, _>          => stack_io(11, 12);
    0x8B => DUP12 => stack::dup::<12, _, _>          => stack_io(12, 13);
    0x8C => DUP13 => stack::dup::<13, _, _>          => stack_io(13, 14);
    0x8D => DUP14 => stack::dup::<14, _, _>          => stack_io(14, 15);
    0x8E => DUP15 => stack::dup::<15, _, _>          => stack_io(15, 16);
    0x8F => DUP16 => stack::dup::<16, _, _>          => stack_io(16, 17);

    0x90 => SWAP1  => stack::swap::<1, _, _>         => stack_io(2, 2);
    0x91 => SWAP2  => stack::swap::<2, _, _>         => stack_io(3, 3);
    0x92 => SWAP3  => stack::swap::<3, _, _>         => stack_io(4, 4);
    0x93 => SWAP4  => stack::swap::<4, _, _>         => stack_io(5, 5);
    0x94 => SWAP5  => stack::swap::<5, _, _>         => stack_io(6, 6);
    0x95 => SWAP6  => stack::swap::<6, _, _>         => stack_io(7, 7);
    0x96 => SWAP7  => stack::swap::<7, _, _>         => stack_io(8, 8);
    0x97 => SWAP8  => stack::swap::<8, _, _>         => stack_io(9, 9);
    0x98 => SWAP9  => stack::swap::<9, _, _>         => stack_io(10, 10);
    0x99 => SWAP10 => stack::swap::<10, _, _>        => stack_io(11, 11);
    0x9A => SWAP11 => stack::swap::<11, _, _>        => stack_io(12, 12);
    0x9B => SWAP12 => stack::swap::<12, _, _>        => stack_io(13, 13);
    0x9C => SWAP13 => stack::swap::<13, _, _>        => stack_io(14, 14);
    0x9D => SWAP14 => stack::swap::<14, _, _>        => stack_io(15, 15);
    0x9E => SWAP15 => stack::swap::<15, _, _>        => stack_io(16, 16);
    0x9F => SWAP16 => stack::swap::<16, _, _>        => stack_io(17, 17);

    0xA0 => LOG0 => host::log::<0, _>                => stack_io(2, 0);
    0xA1 => LOG1 => host::log::<1, _>                => stack_io(3, 0);
    0xA2 => LOG2 => host::log::<2, _>                => stack_io(4, 0);
    0xA3 => LOG3 => host::log::<3, _>                => stack_io(5, 0);
    0xA4 => LOG4 => host::log::<4, _>                => stack_io(6, 0);
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
    0xE6 => DUPN     => stack::dupn                   => stack_io(0, 1), immediate_size(1);
    0xE7 => SWAPN    => stack::swapn                  => stack_io(0, 0), immediate_size(1);
    0xE8 => EXCHANGE => stack::exchange               => stack_io(0, 0), immediate_size(1);
    // 0xE9
    // 0xEA
    // 0xEB
    // 0xEC
    // 0xED
    // 0xEE
    // 0xEF
    0xF0 => CREATE       => contract::create::<false, _, _> => stack_io(3, 1);
    0xF1 => CALL         => contract::call            => stack_io(7, 1);
    0xF2 => CALLCODE     => contract::call_code       => stack_io(7, 1);
    0xF3 => RETURN       => control::ret              => stack_io(2, 0), terminating;
    0xF4 => DELEGATECALL => contract::delegate_call   => stack_io(6, 1);
    0xF5 => CREATE2      => contract::create::<true, _, _> => stack_io(4, 1);
    // 0xF6
    // 0xF7
    // 0xF8
    // 0xF9
    0xFA => STATICCALL   => contract::static_call     => stack_io(6, 1);
    // 0xFB
    // 0xFC
    0xFD => REVERT       => control::revert           => stack_io(2, 0), terminating;
    0xFE => INVALID      => control::invalid          => stack_io(0, 0), terminating;
    0xFF => SELFDESTRUCT => host::selfdestruct        => stack_io(1, 0), terminating;
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

        for push in PUSH1..=PUSH32 {
            expected[push as usize] = push - PUSH1 + 1;
        }

        for stack_op in [DUPN, SWAPN, EXCHANGE] {
            expected[stack_op as usize] = 1;
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
        assert_eq!(opcode_num, 154);
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
    #[should_panic(expected = "opcode not found")]
    fn test_new_unchecked_invalid() {
        let op = unsafe { OpCode::new_unchecked(0x0C) };
        op.info();
    }

    #[test]
    fn test_op_code_valid() {
        let op1 = OpCode::new(ADD).unwrap();
        let op2 = OpCode::new(MUL).unwrap();
        assert!(op1.is_valid());
        assert!(op2.is_valid());

        let op3 = unsafe { OpCode::new_unchecked(0x0C) };
        assert!(!op3.is_valid());
    }

    #[test]
    fn test_modifies_memory() {
        assert!(OpCode::new(MLOAD).unwrap().modifies_memory());
        assert!(OpCode::new(MSTORE).unwrap().modifies_memory());
        assert!(OpCode::new(KECCAK256).unwrap().modifies_memory());
        assert!(!OpCode::new(ADD).unwrap().modifies_memory());
        assert!(OpCode::new(LOG0).unwrap().modifies_memory());
        assert!(OpCode::new(LOG1).unwrap().modifies_memory());
        assert!(OpCode::new(LOG2).unwrap().modifies_memory());
        assert!(OpCode::new(LOG3).unwrap().modifies_memory());
        assert!(OpCode::new(LOG4).unwrap().modifies_memory());
        assert!(OpCode::new(CREATE).unwrap().modifies_memory());
        assert!(OpCode::new(CREATE2).unwrap().modifies_memory());
    }
}
