
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
/// Opcode enum. One-to-one corresponding to an `u8` value.
//#[derive(Clone, Copy, Debug, Eq, PartialEq)]
//pub struct Opcode(pub u8,

// Core opcodes.
/// Operational codes used to specify instuction
#[repr(u8)]
#[derive(Eq, PartialEq, TryFromPrimitive, Ord, PartialOrd, Clone, Copy, Debug, Hash)]
pub enum OpCode {
	/// `STOP`
	STOP =  0x00,
	/// `ADD`
    ADD = 0x01,
	/// `MUL`
    MUL = 0x02,
	/// `SUB`
    SUB = 0x03,
	/// `DIV`
    DIV = 0x04,
	/// `SDIV`
    SDIV = 0x05,
	/// `MOD`
    MOD = 0x06,
	/// `SMOD`
    SMOD = 0x07,
	/// `ADDMOD`
    ADDMOD = 0x08,
	/// `MULMOD`
    MULMOD = 0x09,
	/// `EXP`
    EXP = 0x0a,
	/// `SIGNEXTEND`
    SIGNEXTEND = 0x0b,

	/// `LT`
    LT = 0x10,
	/// `GT`
    GT = 0x11,
	/// `SLT`
    SLT = 0x12,
	/// `SGT`
    SGT = 0x13,
	/// `EQ`
    EQ = 0x14,
	/// `ISZERO`
    ISZERO = 0x15,
	/// `AND`
    AND = 0x16,
	/// `OR`
    OR = 0x17,
	/// `XOR`
    XOR = 0x18,
	/// `NOT`
    NOT = 0x19,
	/// `BYTE`
    BYTE = 0x1a,

	/// `CALLDATALOAD`
    CALLDATALOAD = 0x35,
	/// `CALLDATASIZE`
    CALLDATASIZE = 0x36,
	/// `CALLDATACOPY`
    CALLDATACOPY = 0x37,
	/// `CODESIZE`
    CODESIZE = 0x38,
	/// `CODECOPY`
    CODECOPY = 0x39,

	/// `SHL`
    SHL = 0x1b,
	/// `SHR`
    SHR = 0x1c,
	/// `SAR`
    SAR = 0x1d,

	/// `POP`
    POP = 0x50,
	/// `MLOAD`
    MLOAD = 0x51,
	/// `MSTORE`
    MSTORE = 0x52,
	/// `MSTORE8`
    MSTORE8 = 0x53,
	/// `JUMP`
    JUMP = 0x56,
	/// `JUMPI`
    JUMPI = 0x57,
	/// `PC`
    PC = 0x58,
	/// `MSIZE`
    MSIZE = 0x59,
	/// `JUMPDEST`
    JUMPDEST = 0x5b,

	/// `PUSHn`
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6a,
    PUSH12 = 0x6b,
    PUSH13 = 0x6c,
    PUSH14 = 0x6d,
    PUSH15 = 0x6e,
    PUSH16 = 0x6f,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7a,
    PUSH28 = 0x7b,
    PUSH29 = 0x7c,
    PUSH30 = 0x7d,
    PUSH31 = 0x7e,
    PUSH32 = 0x7f,

	/// `DUPn`
    DUP1 = 0x80,
    DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8a,
    DUP12 = 0x8b,
    DUP13 = 0x8c,
    DUP14 = 0x8d,
    DUP15 = 0x8e,
    DUP16 = 0x8f,

	/// `SWAPn`
    SWAP1 = 0x90,
    SWAP2 = 0x91,
    SWAP3 = 0x92,
    SWAP4 = 0x93,
    SWAP5 = 0x94,
    SWAP6 = 0x95,
    SWAP7 = 0x96,
    SWAP8 = 0x97,
    SWAP9 = 0x98,
    SWAP10 = 0x99,
    SWAP11 = 0x9a,
    SWAP12 = 0x9b,
    SWAP13 = 0x9c,
    SWAP14 = 0x9d,
    SWAP15 = 0x9e,
    SWAP16 = 0x9f,

	/// `RETURN`
    RETURN = 0xf3,
	/// `REVERT`
    REVERT = 0xfd,

	/// `INVALID`
    INVALID = 0xfe,

	/// `SHA3`
    SHA3 = 0x20,
	/// `ADDRESS`
    ADDRESS = 0x30,
	/// `BALANCE`
    BALANCE = 0x31,
	/// `SELFBALANCE`
    SELFBALANCE = 0x47,
	/// `ORIGIN`
    ORIGIN = 0x32,
	/// `CALLER`
    CALLER = 0x33,
	/// `CALLVALUE`
    CALLVALUE = 0x34,
	/// `GASPRICE`
    GASPRICE = 0x3a,
	/// `EXTCODESIZE`
    EXTCODESIZE = 0x3b,
	/// `EXTCODECOPY`
    EXTCODECOPY = 0x3c,
	/// `EXTCODEHASH`
    EXTCODEHASH = 0x3f,
	/// `RETURNDATASIZE`
    RETURNDATASIZE = 0x3d,
	/// `RETURNDATACOPY`
    RETURNDATACOPY = 0x3e,
	/// `BLOCKHASH`
    BLOCKHASH = 0x40,
	/// `COINBASE`
    COINBASE = 0x41,
	/// `TIMESTAMP`
    TIMESTAMP = 0x42,
	/// `NUMBER`
    NUMBER = 0x43,
	/// `DIFFICULTY`
    DIFFICULTY = 0x44,
	/// `GASLIMIT`
    GASLIMIT = 0x45,
	/// `SLOAD`
    SLOAD = 0x54,
	/// `SSTORE`
    SSTORE = 0x55,
	/// `GAS`
    GAS = 0x5a,
	/// `LOGn`
    LOG0 = 0xa0,
    LOG1 = 0xa1,
    LOG2 = 0xa2,
    LOG3 = 0xa3,
    LOG4 = 0xa4,
	/// `CREATE`
    CREATE = 0xf0,
	/// `CREATE2`
    CREATE2 = 0xf5,
	/// `CALL`
    CALL = 0xf1,
	/// `CALLCODE`
    CALLCODE = 0xf2,
	/// `DELEGATECALL`
    DELEGATECALL = 0xf4,
	/// `STATICCALL`
    STATICCALL = 0xfa,
	/// `SUICIDE`
    SUICIDE = 0xff,
	/// `CHAINID`
    CHAINID = 0x46,
}

impl OpCode {
	/// Whether the opcode is a push opcode.
	pub fn is_push_self(&self) -> Option<u8> {
        Self::is_push(*self as u8)
	}

    pub fn try_from_u8(opcode: u8) -> Option<OpCode> {
        OpCode::try_from(opcode).ok()
    }

    #[inline]
    pub fn is_push(opcode: u8) -> Option<u8> {
        if (0x60..=0x7f).contains(&opcode) {
			Some(opcode - 0x60 + 1)
		} else {
			None
		}
    }

	#[inline]
    pub fn as_u8(&self) -> u8 {
		*self as u8
	}

	#[inline]
    pub const fn as_usize(&self) -> usize {
		*self as usize
	}
}