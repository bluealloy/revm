use crate::SpecId;

use super::gas;

pub struct OpCode(u8);

pub const STOP: u8 = 0x00;
pub const ADD: u8 = 0x01;
pub const MUL: u8 = 0x02;
pub const SUB: u8 = 0x03;
pub const DIV: u8 = 0x04;
pub const SDIV: u8 = 0x05;
pub const MOD: u8 = 0x06;
pub const SMOD: u8 = 0x07;
pub const ADDMOD: u8 = 0x08;
pub const MULMOD: u8 = 0x09;
pub const EXP: u8 = 0x0a;
pub const SIGNEXTEND: u8 = 0x0b;

pub const LT: u8 = 0x10;
pub const GT: u8 = 0x11;
pub const SLT: u8 = 0x12;
pub const SGT: u8 = 0x13;
pub const EQ: u8 = 0x14;
pub const ISZERO: u8 = 0x15;
pub const AND: u8 = 0x16;
pub const OR: u8 = 0x17;
pub const XOR: u8 = 0x18;
pub const NOT: u8 = 0x19;
pub const BYTE: u8 = 0x1a;

pub const CALLDATALOAD: u8 = 0x35;
pub const CALLDATASIZE: u8 = 0x36;
pub const CALLDATACOPY: u8 = 0x37;
pub const CODESIZE: u8 = 0x38;
pub const CODECOPY: u8 = 0x39;

pub const SHL: u8 = 0x1b;
pub const SHR: u8 = 0x1c;
pub const SAR: u8 = 0x1d;
pub const SHA3: u8 = 0x20;
pub const POP: u8 = 0x50;
pub const MLOAD: u8 = 0x51;
pub const MSTORE: u8 = 0x52;
pub const MSTORE8: u8 = 0x53;
pub const JUMP: u8 = 0x56;
pub const JUMPI: u8 = 0x57;
pub const PC: u8 = 0x58;
pub const MSIZE: u8 = 0x59;
pub const JUMPDEST: u8 = 0x5b;
pub const PUSH1: u8 = 0x60;
pub const PUSH2: u8 = 0x61;
pub const PUSH3: u8 = 0x62;
pub const PUSH4: u8 = 0x63;
pub const PUSH5: u8 = 0x64;
pub const PUSH6: u8 = 0x65;
pub const PUSH7: u8 = 0x66;
pub const PUSH8: u8 = 0x67;
pub const PUSH9: u8 = 0x68;
pub const PUSH10: u8 = 0x69;
pub const PUSH11: u8 = 0x6a;
pub const PUSH12: u8 = 0x6b;
pub const PUSH13: u8 = 0x6c;
pub const PUSH14: u8 = 0x6d;
pub const PUSH15: u8 = 0x6e;
pub const PUSH16: u8 = 0x6f;
pub const PUSH17: u8 = 0x70;
pub const PUSH18: u8 = 0x71;
pub const PUSH19: u8 = 0x72;
pub const PUSH20: u8 = 0x73;
pub const PUSH21: u8 = 0x74;
pub const PUSH22: u8 = 0x75;
pub const PUSH23: u8 = 0x76;
pub const PUSH24: u8 = 0x77;
pub const PUSH25: u8 = 0x78;
pub const PUSH26: u8 = 0x79;
pub const PUSH27: u8 = 0x7a;
pub const PUSH28: u8 = 0x7b;
pub const PUSH29: u8 = 0x7c;
pub const PUSH30: u8 = 0x7d;
pub const PUSH31: u8 = 0x7e;
pub const PUSH32: u8 = 0x7f;
pub const DUP1: u8 = 0x80;
pub const DUP2: u8 = 0x81;
pub const DUP3: u8 = 0x82;
pub const DUP4: u8 = 0x83;
pub const DUP5: u8 = 0x84;
pub const DUP6: u8 = 0x85;
pub const DUP7: u8 = 0x86;
pub const DUP8: u8 = 0x87;
pub const DUP9: u8 = 0x88;
pub const DUP10: u8 = 0x89;
pub const DUP11: u8 = 0x8a;
pub const DUP12: u8 = 0x8b;
pub const DUP13: u8 = 0x8c;
pub const DUP14: u8 = 0x8d;
pub const DUP15: u8 = 0x8e;
pub const DUP16: u8 = 0x8f;
pub const SWAP1: u8 = 0x90;
pub const SWAP2: u8 = 0x91;
pub const SWAP3: u8 = 0x92;
pub const SWAP4: u8 = 0x93;
pub const SWAP5: u8 = 0x94;
pub const SWAP6: u8 = 0x95;
pub const SWAP7: u8 = 0x96;
pub const SWAP8: u8 = 0x97;
pub const SWAP9: u8 = 0x98;
pub const SWAP10: u8 = 0x99;
pub const SWAP11: u8 = 0x9a;
pub const SWAP12: u8 = 0x9b;
pub const SWAP13: u8 = 0x9c;
pub const SWAP14: u8 = 0x9d;
pub const SWAP15: u8 = 0x9e;
pub const SWAP16: u8 = 0x9f;
pub const RETURN: u8 = 0xf3;
pub const REVERT: u8 = 0xfd;
pub const INVALID: u8 = 0xfe;
pub const ADDRESS: u8 = 0x30;
pub const BALANCE: u8 = 0x31;
pub const BASEFEE: u8 = 0x48;
pub const ORIGIN: u8 = 0x32;
pub const CALLER: u8 = 0x33;
pub const CALLVALUE: u8 = 0x34;
pub const GASPRICE: u8 = 0x3a;
pub const EXTCODESIZE: u8 = 0x3b;
pub const EXTCODECOPY: u8 = 0x3c;
pub const EXTCODEHASH: u8 = 0x3f;
pub const RETURNDATASIZE: u8 = 0x3d;
pub const RETURNDATACOPY: u8 = 0x3e;
pub const BLOCKHASH: u8 = 0x40;
pub const COINBASE: u8 = 0x41;
pub const TIMESTAMP: u8 = 0x42;
pub const NUMBER: u8 = 0x43;
pub const DIFFICULTY: u8 = 0x44;
pub const GASLIMIT: u8 = 0x45;
pub const SELFBALANCE: u8 = 0x47;
pub const SLOAD: u8 = 0x54;
pub const SSTORE: u8 = 0x55;
pub const GAS: u8 = 0x5a;
pub const LOG0: u8 = 0xa0;
pub const LOG1: u8 = 0xa1;
pub const LOG2: u8 = 0xa2;
pub const LOG3: u8 = 0xa3;
pub const LOG4: u8 = 0xa4;
pub const CREATE: u8 = 0xf0;
pub const CREATE2: u8 = 0xf5;
pub const CALL: u8 = 0xf1;
pub const CALLCODE: u8 = 0xf2;
pub const DELEGATECALL: u8 = 0xf4;
pub const STATICCALL: u8 = 0xfa;
pub const SELFDESTRUCT: u8 = 0xff;
pub const CHAINID: u8 = 0x46;

impl OpCode {
    pub fn try_from_u8(opcode: u8) -> Option<OpCode> {
        OPCODE_JUMPMAP[opcode as usize].map(|_| OpCode(opcode))
    }

    #[inline(always)]
    pub fn is_push(opcode: u8) -> Option<u8> {
        if (0x60..=0x7f).contains(&opcode) {
            Some(opcode - 0x60 + 1)
        } else {
            None
        }
    }

    pub const fn as_str(&self) -> &'static str {
        if let Some(str) = OPCODE_JUMPMAP[self.0 as usize] {
            str
        } else {
            "unknown"
        }
    }

    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.0
    }
}
#[derive(Debug)]
pub struct OpInfo {
    pub gas: u64,
    pub gas_block_end: bool,
}

impl OpInfo {
    pub const fn none() -> Self {
        Self {
            gas: u64::MAX,
            gas_block_end: true,
        }
    }
    pub const fn gas_block_end(gas: u64) -> Self {
        Self {
            gas,
            gas_block_end: true,
        }
    }
    pub const fn dynamic_gas() -> Self {
        Self {
            gas: 0,
            gas_block_end: false,
        }
    }
    pub const fn gas(gas: u64) -> Self {
        Self {
            gas,
            gas_block_end: false,
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
            /* 0x0a  EXP */ OpInfo::dynamic_gas(),
            /* 0x0b  SIGNEXTEND */ OpInfo::gas(gas::LOW),
            /* 0x0c */ OpInfo::none(),
            /* 0x0d */ OpInfo::none(),
            /* 0x0e */ OpInfo::none(),
            /* 0x0f */ OpInfo::none(),
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
            /* 0x1a  BYTE */ OpInfo::gas(gas::VERYLOW),
            /* 0x1b  SHL */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1c  SHR */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1d  SAR */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::CONSTANTINOPLE) {
                gas::VERYLOW
            } else {
                0
            }),
            /* 0x1e */ OpInfo::none(),
            /* 0x1f */ OpInfo::none(),
            /* 0x20  SHA3 */ OpInfo::dynamic_gas(),
            /* 0x21 */ OpInfo::none(),
            /* 0x22 */ OpInfo::none(),
            /* 0x23 */ OpInfo::none(),
            /* 0x24 */ OpInfo::none(),
            /* 0x25 */ OpInfo::none(),
            /* 0x26 */ OpInfo::none(),
            /* 0x27 */ OpInfo::none(),
            /* 0x28 */ OpInfo::none(),
            /* 0x29 */ OpInfo::none(),
            /* 0x2a */ OpInfo::none(),
            /* 0x2b */ OpInfo::none(),
            /* 0x2c */ OpInfo::none(),
            /* 0x2d */ OpInfo::none(),
            /* 0x2e */ OpInfo::none(),
            /* 0x2f */ OpInfo::none(),
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
            /* 0x3a  GASPRICE */ OpInfo::gas(gas::BASE),
            /* 0x3b  EXTCODESIZE */ OpInfo::dynamic_gas(),
            /* 0x3c  EXTCODECOPY */ OpInfo::dynamic_gas(),
            /* 0x3d  RETURNDATASIZE */
            OpInfo::gas(if SpecId::enabled($spec_id, SpecId::BYZANTINE) {
                gas::BASE
            } else {
                0
            }),
            /* 0x3e  RETURNDATACOPY */ OpInfo::dynamic_gas(),
            /* 0x3f  EXTCODEHASH */ OpInfo::dynamic_gas(),
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
            /* 0x4a */ OpInfo::none(),
            /* 0x4b */ OpInfo::none(),
            /* 0x4c */ OpInfo::none(),
            /* 0x4d */ OpInfo::none(),
            /* 0x4e */ OpInfo::none(),
            /* 0x4f */ OpInfo::none(),
            /* 0x50  POP */ OpInfo::gas(gas::BASE),
            /* 0x51  MLOAD */ OpInfo::gas(gas::VERYLOW),
            /* 0x52  MSTORE */ OpInfo::gas(gas::VERYLOW),
            /* 0x53  MSTORE8 */ OpInfo::gas(gas::VERYLOW),
            /* 0x54  SLOAD */ OpInfo::dynamic_gas(),
            /* 0x55  SSTORE */ OpInfo::dynamic_gas(),
            /* 0x56  JUMP */ OpInfo::gas_block_end(gas::MID),
            /* 0x57  JUMPI */ OpInfo::gas_block_end(gas::HIGH),
            /* 0x58  PC */ OpInfo::gas(gas::BASE),
            /* 0x59  MSIZE */ OpInfo::gas(gas::BASE),
            /* 0x5a  GAS */ OpInfo::gas_block_end(gas::BASE),
            /* 0x5b  JUMPDEST */
            OpInfo::gas_block_end(0), //gas::JUMPDEST gas is calculated in function call,
            /* 0x5c */ OpInfo::none(),
            /* 0x5d */ OpInfo::none(),
            /* 0x5e */ OpInfo::none(),
            /* 0x5f */ OpInfo::none(),
            /* 0x60  PUSH1 */ OpInfo::gas(gas::VERYLOW),
            /* 0x61  PUSH2 */ OpInfo::gas(gas::VERYLOW),
            /* 0x62  PUSH3 */ OpInfo::gas(gas::VERYLOW),
            /* 0x63  PUSH4 */ OpInfo::gas(gas::VERYLOW),
            /* 0x64  PUSH5 */ OpInfo::gas(gas::VERYLOW),
            /* 0x65  PUSH6 */ OpInfo::gas(gas::VERYLOW),
            /* 0x66  PUSH7 */ OpInfo::gas(gas::VERYLOW),
            /* 0x67  PUSH8 */ OpInfo::gas(gas::VERYLOW),
            /* 0x68  PUSH9 */ OpInfo::gas(gas::VERYLOW),
            /* 0x69  PUSH10 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6a  PUSH11 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6b  PUSH12 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6c  PUSH13 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6d  PUSH14 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6e  PUSH15 */ OpInfo::gas(gas::VERYLOW),
            /* 0x6f  PUSH16 */ OpInfo::gas(gas::VERYLOW),
            /* 0x70  PUSH17 */ OpInfo::gas(gas::VERYLOW),
            /* 0x71  PUSH18 */ OpInfo::gas(gas::VERYLOW),
            /* 0x72  PUSH19 */ OpInfo::gas(gas::VERYLOW),
            /* 0x73  PUSH20 */ OpInfo::gas(gas::VERYLOW),
            /* 0x74  PUSH21 */ OpInfo::gas(gas::VERYLOW),
            /* 0x75  PUSH22 */ OpInfo::gas(gas::VERYLOW),
            /* 0x76  PUSH23 */ OpInfo::gas(gas::VERYLOW),
            /* 0x77  PUSH24 */ OpInfo::gas(gas::VERYLOW),
            /* 0x78  PUSH25 */ OpInfo::gas(gas::VERYLOW),
            /* 0x79  PUSH26 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7a  PUSH27 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7b  PUSH28 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7c  PUSH29 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7d  PUSH30 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7e  PUSH31 */ OpInfo::gas(gas::VERYLOW),
            /* 0x7f  PUSH32 */ OpInfo::gas(gas::VERYLOW),
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
            /* 0x8a  DUP11 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8b  DUP12 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8c  DUP13 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8d  DUP14 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8e  DUP15 */ OpInfo::gas(gas::VERYLOW),
            /* 0x8f  DUP16 */ OpInfo::gas(gas::VERYLOW),
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
            /* 0x9a  SWAP11 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9b  SWAP12 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9c  SWAP13 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9d  SWAP14 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9e  SWAP15 */ OpInfo::gas(gas::VERYLOW),
            /* 0x9f  SWAP16 */ OpInfo::gas(gas::VERYLOW),
            /* 0xa0  LOG0 */ OpInfo::dynamic_gas(),
            /* 0xa1  LOG1 */ OpInfo::dynamic_gas(),
            /* 0xa2  LOG2 */ OpInfo::dynamic_gas(),
            /* 0xa3  LOG3 */ OpInfo::dynamic_gas(),
            /* 0xa4  LOG4 */ OpInfo::dynamic_gas(),
            /* 0xa5 */ OpInfo::none(),
            /* 0xa6 */ OpInfo::none(),
            /* 0xa7 */ OpInfo::none(),
            /* 0xa8 */ OpInfo::none(),
            /* 0xa9 */ OpInfo::none(),
            /* 0xaa */ OpInfo::none(),
            /* 0xab */ OpInfo::none(),
            /* 0xac */ OpInfo::none(),
            /* 0xad */ OpInfo::none(),
            /* 0xae */ OpInfo::none(),
            /* 0xaf */ OpInfo::none(),
            /* 0xb0 */ OpInfo::none(),
            /* 0xb1 */ OpInfo::none(),
            /* 0xb2 */ OpInfo::none(),
            /* 0xb3 */ OpInfo::none(),
            /* 0xb4 */ OpInfo::none(),
            /* 0xb5 */ OpInfo::none(),
            /* 0xb6 */ OpInfo::none(),
            /* 0xb7 */ OpInfo::none(),
            /* 0xb8 */ OpInfo::none(),
            /* 0xb9 */ OpInfo::none(),
            /* 0xba */ OpInfo::none(),
            /* 0xbb */ OpInfo::none(),
            /* 0xbc */ OpInfo::none(),
            /* 0xbd */ OpInfo::none(),
            /* 0xbe */ OpInfo::none(),
            /* 0xbf */ OpInfo::none(),
            /* 0xc0 */ OpInfo::none(),
            /* 0xc1 */ OpInfo::none(),
            /* 0xc2 */ OpInfo::none(),
            /* 0xc3 */ OpInfo::none(),
            /* 0xc4 */ OpInfo::none(),
            /* 0xc5 */ OpInfo::none(),
            /* 0xc6 */ OpInfo::none(),
            /* 0xc7 */ OpInfo::none(),
            /* 0xc8 */ OpInfo::none(),
            /* 0xc9 */ OpInfo::none(),
            /* 0xca */ OpInfo::none(),
            /* 0xcb */ OpInfo::none(),
            /* 0xcc */ OpInfo::none(),
            /* 0xcd */ OpInfo::none(),
            /* 0xce */ OpInfo::none(),
            /* 0xcf */ OpInfo::none(),
            /* 0xd0 */ OpInfo::none(),
            /* 0xd1 */ OpInfo::none(),
            /* 0xd2 */ OpInfo::none(),
            /* 0xd3 */ OpInfo::none(),
            /* 0xd4 */ OpInfo::none(),
            /* 0xd5 */ OpInfo::none(),
            /* 0xd6 */ OpInfo::none(),
            /* 0xd7 */ OpInfo::none(),
            /* 0xd8 */ OpInfo::none(),
            /* 0xd9 */ OpInfo::none(),
            /* 0xda */ OpInfo::none(),
            /* 0xdb */ OpInfo::none(),
            /* 0xdc */ OpInfo::none(),
            /* 0xdd */ OpInfo::none(),
            /* 0xde */ OpInfo::none(),
            /* 0xdf */ OpInfo::none(),
            /* 0xe0 */ OpInfo::none(),
            /* 0xe1 */ OpInfo::none(),
            /* 0xe2 */ OpInfo::none(),
            /* 0xe3 */ OpInfo::none(),
            /* 0xe4 */ OpInfo::none(),
            /* 0xe5 */ OpInfo::none(),
            /* 0xe6 */ OpInfo::none(),
            /* 0xe7 */ OpInfo::none(),
            /* 0xe8 */ OpInfo::none(),
            /* 0xe9 */ OpInfo::none(),
            /* 0xea */ OpInfo::none(),
            /* 0xeb */ OpInfo::none(),
            /* 0xec */ OpInfo::none(),
            /* 0xed */ OpInfo::none(),
            /* 0xee */ OpInfo::none(),
            /* 0xef */ OpInfo::none(),
            /* 0xf0  CREATE */ OpInfo::gas_block_end(0),
            /* 0xf1  CALL */ OpInfo::gas_block_end(0),
            /* 0xf2  CALLCODE */ OpInfo::gas_block_end(0),
            /* 0xf3  RETURN */ OpInfo::gas_block_end(0),
            /* 0xf4  DELEGATECALL */ OpInfo::gas_block_end(0),
            /* 0xf5  CREATE2 */ OpInfo::gas_block_end(0),
            /* 0xf6 */ OpInfo::none(),
            /* 0xf7 */ OpInfo::none(),
            /* 0xf8 */ OpInfo::none(),
            /* 0xf9 */ OpInfo::none(),
            /* 0xfa  STATICCALL */ OpInfo::gas_block_end(0),
            /* 0xfb */ OpInfo::none(),
            /* 0xfc */ OpInfo::none(),
            /* 0xfd  REVERT */ OpInfo::gas_block_end(0),
            /* 0xfe  INVALID */ OpInfo::gas_block_end(0),
            /* 0xff  SELFDESTRUCT */ OpInfo::gas_block_end(0),
        ];
    };
}

pub const fn spec_opcode_gas(spec_id: SpecId) -> &'static [OpInfo; 256] {
    match spec_id {
        SpecId::FRONTIER => {
            gas_opcodee!(O, SpecId::FRONTIER);
            O
        }
        SpecId::HOMESTEAD => {
            gas_opcodee!(O, SpecId::HOMESTEAD);
            O
        }
        SpecId::TANGERINE => {
            gas_opcodee!(O, SpecId::TANGERINE);
            O
        }
        SpecId::SPURIOUS_DRAGON => {
            gas_opcodee!(O, SpecId::SPURIOUS_DRAGON);
            O
        }
        SpecId::BYZANTINE => {
            gas_opcodee!(O, SpecId::BYZANTINE);
            O
        }
        SpecId::CONSTANTINOPLE => {
            gas_opcodee!(O, SpecId::CONSTANTINOPLE);
            O
        }
        SpecId::PETERSBURG => {
            gas_opcodee!(O, SpecId::PETERSBURG);
            O
        }
        SpecId::ISTANBUL => {
            gas_opcodee!(O, SpecId::ISTANBUL);
            O
        }
        SpecId::MUIRGLACIER => {
            gas_opcodee!(O, SpecId::MUIRGLACIER);
            O
        }
        SpecId::BERLIN => {
            gas_opcodee!(O, SpecId::BERLIN);
            O
        }
        SpecId::LONDON => {
            gas_opcodee!(O, SpecId::LONDON);
            O
        }
        SpecId::LATEST => {
            gas_opcodee!(O, SpecId::LATEST);
            O
        }
    }
}

pub const OPCODE_JUMPMAP: [Option<&'static str>; 256] = [
    /* 0x00 */ Some("STOP"),
    /* 0x01 */ Some("ADD"),
    /* 0x02 */ Some("MUL"),
    /* 0x03 */ Some("SUB"),
    /* 0x04 */ Some("DIV"),
    /* 0x05 */ Some("SDIV"),
    /* 0x06 */ Some("MOD"),
    /* 0x07 */ Some("SMOD"),
    /* 0x08 */ Some("ADDMOD"),
    /* 0x09 */ Some("MULMOD"),
    /* 0x0a */ Some("EXP"),
    /* 0x0b */ Some("SIGNEXTEND"),
    /* 0x0c */ None,
    /* 0x0d */ None,
    /* 0x0e */ None,
    /* 0x0f */ None,
    /* 0x10 */ Some("LT"),
    /* 0x11 */ Some("GT"),
    /* 0x12 */ Some("SLT"),
    /* 0x13 */ Some("SGT"),
    /* 0x14 */ Some("EQ"),
    /* 0x15 */ Some("ISZERO"),
    /* 0x16 */ Some("AND"),
    /* 0x17 */ Some("OR"),
    /* 0x18 */ Some("XOR"),
    /* 0x19 */ Some("NOT"),
    /* 0x1a */ Some("BYTE"),
    /* 0x1b */ Some("SHL"),
    /* 0x1c */ Some("SHR"),
    /* 0x1d */ Some("SAR"),
    /* 0x1e */ None,
    /* 0x1f */ None,
    /* 0x20 */ Some("SHA3"),
    /* 0x21 */ None,
    /* 0x22 */ None,
    /* 0x23 */ None,
    /* 0x24 */ None,
    /* 0x25 */ None,
    /* 0x26 */ None,
    /* 0x27 */ None,
    /* 0x28 */ None,
    /* 0x29 */ None,
    /* 0x2a */ None,
    /* 0x2b */ None,
    /* 0x2c */ None,
    /* 0x2d */ None,
    /* 0x2e */ None,
    /* 0x2f */ None,
    /* 0x30 */ Some("ADDRESS"),
    /* 0x31 */ Some("BALANCE"),
    /* 0x32 */ Some("ORIGIN"),
    /* 0x33 */ Some("CALLER"),
    /* 0x34 */ Some("CALLVALUE"),
    /* 0x35 */ Some("CALLDATALOAD"),
    /* 0x36 */ Some("CALLDATASIZE"),
    /* 0x37 */ Some("CALLDATACOPY"),
    /* 0x38 */ Some("CODESIZE"),
    /* 0x39 */ Some("CODECOPY"),
    /* 0x3a */ Some("GASPRICE"),
    /* 0x3b */ Some("EXTCODESIZE"),
    /* 0x3c */ Some("EXTCODECOPY"),
    /* 0x3d */ Some("RETURNDATASIZE"),
    /* 0x3e */ Some("RETURNDATACOPY"),
    /* 0x3f */ Some("EXTCODEHASH"),
    /* 0x40 */ Some("BLOCKHASH"),
    /* 0x41 */ Some("COINBASE"),
    /* 0x42 */ Some("TIMESTAMP"),
    /* 0x43 */ Some("NUMBER"),
    /* 0x44 */ Some("DIFFICULTY"),
    /* 0x45 */ Some("GASLIMIT"),
    /* 0x46 */ Some("CHAINID"),
    /* 0x47 */ Some("SELFBALANCE"),
    /* 0x48 */ Some("BASEFEE"),
    /* 0x49 */ None,
    /* 0x4a */ None,
    /* 0x4b */ None,
    /* 0x4c */ None,
    /* 0x4d */ None,
    /* 0x4e */ None,
    /* 0x4f */ None,
    /* 0x50 */ Some("POP"),
    /* 0x51 */ Some("MLOAD"),
    /* 0x52 */ Some("MSTORE"),
    /* 0x53 */ Some("MSTORE8"),
    /* 0x54 */ Some("SLOAD"),
    /* 0x55 */ Some("SSTORE"),
    /* 0x56 */ Some("JUMP"),
    /* 0x57 */ Some("JUMPI"),
    /* 0x58 */ Some("PC"),
    /* 0x59 */ Some("MSIZE"),
    /* 0x5a */ Some("GAS"),
    /* 0x5b */ Some("JUMPDEST"),
    /* 0x5c */ None,
    /* 0x5d */ None,
    /* 0x5e */ None,
    /* 0x5f */ None,
    /* 0x60 */ Some("PUSH1"),
    /* 0x61 */ Some("PUSH2"),
    /* 0x62 */ Some("PUSH3"),
    /* 0x63 */ Some("PUSH4"),
    /* 0x64 */ Some("PUSH5"),
    /* 0x65 */ Some("PUSH6"),
    /* 0x66 */ Some("PUSH7"),
    /* 0x67 */ Some("PUSH8"),
    /* 0x68 */ Some("PUSH9"),
    /* 0x69 */ Some("PUSH10"),
    /* 0x6a */ Some("PUSH11"),
    /* 0x6b */ Some("PUSH12"),
    /* 0x6c */ Some("PUSH13"),
    /* 0x6d */ Some("PUSH14"),
    /* 0x6e */ Some("PUSH15"),
    /* 0x6f */ Some("PUSH16"),
    /* 0x70 */ Some("PUSH17"),
    /* 0x71 */ Some("PUSH18"),
    /* 0x72 */ Some("PUSH19"),
    /* 0x73 */ Some("PUSH20"),
    /* 0x74 */ Some("PUSH21"),
    /* 0x75 */ Some("PUSH22"),
    /* 0x76 */ Some("PUSH23"),
    /* 0x77 */ Some("PUSH24"),
    /* 0x78 */ Some("PUSH25"),
    /* 0x79 */ Some("PUSH26"),
    /* 0x7a */ Some("PUSH27"),
    /* 0x7b */ Some("PUSH28"),
    /* 0x7c */ Some("PUSH29"),
    /* 0x7d */ Some("PUSH30"),
    /* 0x7e */ Some("PUSH31"),
    /* 0x7f */ Some("PUSH32"),
    /* 0x80 */ Some("DUP1"),
    /* 0x81 */ Some("DUP2"),
    /* 0x82 */ Some("DUP3"),
    /* 0x83 */ Some("DUP4"),
    /* 0x84 */ Some("DUP5"),
    /* 0x85 */ Some("DUP6"),
    /* 0x86 */ Some("DUP7"),
    /* 0x87 */ Some("DUP8"),
    /* 0x88 */ Some("DUP9"),
    /* 0x89 */ Some("DUP10"),
    /* 0x8a */ Some("DUP11"),
    /* 0x8b */ Some("DUP12"),
    /* 0x8c */ Some("DUP13"),
    /* 0x8d */ Some("DUP14"),
    /* 0x8e */ Some("DUP15"),
    /* 0x8f */ Some("DUP16"),
    /* 0x90 */ Some("SWAP1"),
    /* 0x91 */ Some("SWAP2"),
    /* 0x92 */ Some("SWAP3"),
    /* 0x93 */ Some("SWAP4"),
    /* 0x94 */ Some("SWAP5"),
    /* 0x95 */ Some("SWAP6"),
    /* 0x96 */ Some("SWAP7"),
    /* 0x97 */ Some("SWAP8"),
    /* 0x98 */ Some("SWAP9"),
    /* 0x99 */ Some("SWAP10"),
    /* 0x9a */ Some("SWAP11"),
    /* 0x9b */ Some("SWAP12"),
    /* 0x9c */ Some("SWAP13"),
    /* 0x9d */ Some("SWAP14"),
    /* 0x9e */ Some("SWAP15"),
    /* 0x9f */ Some("SWAP16"),
    /* 0xa0 */ Some("LOG0"),
    /* 0xa1 */ Some("LOG1"),
    /* 0xa2 */ Some("LOG2"),
    /* 0xa3 */ Some("LOG3"),
    /* 0xa4 */ Some("LOG4"),
    /* 0xa5 */ None,
    /* 0xa6 */ None,
    /* 0xa7 */ None,
    /* 0xa8 */ None,
    /* 0xa9 */ None,
    /* 0xaa */ None,
    /* 0xab */ None,
    /* 0xac */ None,
    /* 0xad */ None,
    /* 0xae */ None,
    /* 0xaf */ None,
    /* 0xb0 */ None,
    /* 0xb1 */ None,
    /* 0xb2 */ None,
    /* 0xb3 */ None,
    /* 0xb4 */ None,
    /* 0xb5 */ None,
    /* 0xb6 */ None,
    /* 0xb7 */ None,
    /* 0xb8 */ None,
    /* 0xb9 */ None,
    /* 0xba */ None,
    /* 0xbb */ None,
    /* 0xbc */ None,
    /* 0xbd */ None,
    /* 0xbe */ None,
    /* 0xbf */ None,
    /* 0xc0 */ None,
    /* 0xc1 */ None,
    /* 0xc2 */ None,
    /* 0xc3 */ None,
    /* 0xc4 */ None,
    /* 0xc5 */ None,
    /* 0xc6 */ None,
    /* 0xc7 */ None,
    /* 0xc8 */ None,
    /* 0xc9 */ None,
    /* 0xca */ None,
    /* 0xcb */ None,
    /* 0xcc */ None,
    /* 0xcd */ None,
    /* 0xce */ None,
    /* 0xcf */ None,
    /* 0xd0 */ None,
    /* 0xd1 */ None,
    /* 0xd2 */ None,
    /* 0xd3 */ None,
    /* 0xd4 */ None,
    /* 0xd5 */ None,
    /* 0xd6 */ None,
    /* 0xd7 */ None,
    /* 0xd8 */ None,
    /* 0xd9 */ None,
    /* 0xda */ None,
    /* 0xdb */ None,
    /* 0xdc */ None,
    /* 0xdd */ None,
    /* 0xde */ None,
    /* 0xdf */ None,
    /* 0xe0 */ None,
    /* 0xe1 */ None,
    /* 0xe2 */ None,
    /* 0xe3 */ None,
    /* 0xe4 */ None,
    /* 0xe5 */ None,
    /* 0xe6 */ None,
    /* 0xe7 */ None,
    /* 0xe8 */ None,
    /* 0xe9 */ None,
    /* 0xea */ None,
    /* 0xeb */ None,
    /* 0xec */ None,
    /* 0xed */ None,
    /* 0xee */ None,
    /* 0xef */ None,
    /* 0xf0 */ Some("CREATE"),
    /* 0xf1 */ Some("CALL"),
    /* 0xf2 */ Some("CALLCODE"),
    /* 0xf3 */ Some("RETURN"),
    /* 0xf4 */ Some("DELEGATECALL"),
    /* 0xf5 */ Some("CREATE2"),
    /* 0xf6 */ None,
    /* 0xf7 */ None,
    /* 0xf8 */ None,
    /* 0xf9 */ None,
    /* 0xfa */ Some("STATICCALL"),
    /* 0xfb */ None,
    /* 0xfc */ None,
    /* 0xfd */ Some("REVERT"),
    /* 0xfe */ Some("INVALID"),
    /* 0xff */ Some("SELFDESTRUCT"),
];
