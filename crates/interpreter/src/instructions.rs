//! EVM opcode implementations.

#[macro_use]
pub mod macros;
/// Arithmetic operations (ADD, SUB, MUL, DIV, etc.).
pub mod arithmetic;
/// Bitwise operations (AND, OR, XOR, NOT, etc.).
pub mod bitwise;
/// Block information instructions (COINBASE, TIMESTAMP, etc.).
pub mod block_info;
/// Contract operations (CALL, CREATE, DELEGATECALL, etc.).
pub mod contract;
/// Control flow instructions (JUMP, JUMPI, REVERT, etc.).
pub mod control;
/// Host environment interactions (SLOAD, SSTORE, LOG, etc.).
pub mod host;
/// Signed 256-bit integer operations.
pub mod i256;
/// Memory operations (MLOAD, MSTORE, MSIZE, etc.).
pub mod memory;
/// Stack operations (PUSH, POP, DUP, SWAP, etc.).
pub mod stack;
/// System information instructions (ADDRESS, CALLER, etc.).
pub mod system;
/// Transaction information instructions (ORIGIN, GASPRICE, etc.).
pub mod tx_info;
/// Utility functions and helpers for instruction implementation.
pub mod utility;

use crate::{
    interpreter_types::{InterpreterTypes, Jumps},
    Host, InstructionContext,
};

/// EVM opcode function signature.
///
/// Returns `true` if execution should continue, `false` if execution should halt (`next_action` has been set).
pub type Instruction<W, H> = fn(InstructionContext<'_, H, W>) -> InstructionReturn;

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<W, H> = [Instruction<W, H>; 256];

/// Return value of an [`Instruction`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionReturn {
    halt: bool,
}

impl InstructionReturn {
    /// Continue execution.
    #[inline]
    pub fn cont() -> Self {
        Self { halt: false }
    }

    /// Halt execution.
    #[inline]
    pub fn halt() -> Self {
        Self { halt: true }
    }

    /// Check if execution should continue.
    #[inline]
    pub fn can_continue(&self) -> bool {
        !self.halt
    }
}

/// Returns the default instruction table for the given interpreter types and host.
#[inline]
pub const fn instruction_table<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    const { instruction_table_impl::<WIRE, H>() }
}

const fn instruction_table_impl<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    use bytecode::opcode::*;
    let mut table: [Instruction<WIRE, H>; 256] = [control::unknown::<WIRE, H>; 256];

    macro_rules! assign {
        ($($idx:ident = $instr:expr;)*) => {
            $(
                table[$idx as usize] = $instr;
            )*
        };
    }

    assign! {
    STOP = control::stop;
    ADD = arithmetic::add;
    MUL = arithmetic::mul;
    SUB = arithmetic::sub;
    DIV = arithmetic::div;
    SDIV = arithmetic::sdiv;
    MOD = arithmetic::rem;
    SMOD = arithmetic::smod;
    ADDMOD = arithmetic::addmod;
    MULMOD = arithmetic::mulmod;
    EXP = arithmetic::exp;
    SIGNEXTEND = arithmetic::signextend;

    LT = bitwise::lt;
    GT = bitwise::gt;
    SLT = bitwise::slt;
    SGT = bitwise::sgt;
    EQ = bitwise::eq;
    ISZERO = bitwise::iszero;
    AND = bitwise::bitand;
    OR = bitwise::bitor;
    XOR = bitwise::bitxor;
    NOT = bitwise::not;
    BYTE = bitwise::byte;
    SHL = bitwise::shl;
    SHR = bitwise::shr;
    SAR = bitwise::sar;
    CLZ = bitwise::clz;

    KECCAK256 = system::keccak256;

    ADDRESS = system::address;
    BALANCE = host::balance;
    ORIGIN = tx_info::origin;
    CALLER = system::caller;
    CALLVALUE = system::callvalue;
    CALLDATALOAD = system::calldataload;
    CALLDATASIZE = system::calldatasize;
    CALLDATACOPY = system::calldatacopy;
    CODESIZE = system::codesize;
    CODECOPY = system::codecopy;

    GASPRICE = tx_info::gasprice;
    EXTCODESIZE = host::extcodesize;
    EXTCODECOPY = host::extcodecopy;
    RETURNDATASIZE = system::returndatasize;
    RETURNDATACOPY = system::returndatacopy;
    EXTCODEHASH = host::extcodehash;
    BLOCKHASH = host::blockhash;
    COINBASE = block_info::coinbase;
    TIMESTAMP = block_info::timestamp;
    NUMBER = block_info::block_number;
    DIFFICULTY = block_info::difficulty;
    GASLIMIT = block_info::gaslimit;
    CHAINID = block_info::chainid;
    SELFBALANCE = host::selfbalance;
    BASEFEE = block_info::basefee;
    BLOBHASH = tx_info::blob_hash;
    BLOBBASEFEE = block_info::blob_basefee;

    POP = stack::pop;
    MLOAD = memory::mload;
    MSTORE = memory::mstore;
    MSTORE8 = memory::mstore8;
    SLOAD = host::sload;
    SSTORE = host::sstore;
    JUMP = control::jump;
    JUMPI = control::jumpi;
    PC = control::pc;
    MSIZE = memory::msize;
    GAS = system::gas;
    JUMPDEST = control::jumpdest;
    TLOAD = host::tload;
    TSTORE = host::tstore;
    MCOPY = memory::mcopy;

    PUSH0 = stack::push0;
    PUSH1 = stack::push::<1, _, _>;
    PUSH2 = stack::push::<2, _, _>;
    PUSH3 = stack::push::<3, _, _>;
    PUSH4 = stack::push::<4, _, _>;
    PUSH5 = stack::push::<5, _, _>;
    PUSH6 = stack::push::<6, _, _>;
    PUSH7 = stack::push::<7, _, _>;
    PUSH8 = stack::push::<8, _, _>;
    PUSH9 = stack::push::<9, _, _>;
    PUSH10 = stack::push::<10, _, _>;
    PUSH11 = stack::push::<11, _, _>;
    PUSH12 = stack::push::<12, _, _>;
    PUSH13 = stack::push::<13, _, _>;
    PUSH14 = stack::push::<14, _, _>;
    PUSH15 = stack::push::<15, _, _>;
    PUSH16 = stack::push::<16, _, _>;
    PUSH17 = stack::push::<17, _, _>;
    PUSH18 = stack::push::<18, _, _>;
    PUSH19 = stack::push::<19, _, _>;
    PUSH20 = stack::push::<20, _, _>;
    PUSH21 = stack::push::<21, _, _>;
    PUSH22 = stack::push::<22, _, _>;
    PUSH23 = stack::push::<23, _, _>;
    PUSH24 = stack::push::<24, _, _>;
    PUSH25 = stack::push::<25, _, _>;
    PUSH26 = stack::push::<26, _, _>;
    PUSH27 = stack::push::<27, _, _>;
    PUSH28 = stack::push::<28, _, _>;
    PUSH29 = stack::push::<29, _, _>;
    PUSH30 = stack::push::<30, _, _>;
    PUSH31 = stack::push::<31, _, _>;
    PUSH32 = stack::push::<32, _, _>;

    DUP1 = stack::dup::<1, _, _>;
    DUP2 = stack::dup::<2, _, _>;
    DUP3 = stack::dup::<3, _, _>;
    DUP4 = stack::dup::<4, _, _>;
    DUP5 = stack::dup::<5, _, _>;
    DUP6 = stack::dup::<6, _, _>;
    DUP7 = stack::dup::<7, _, _>;
    DUP8 = stack::dup::<8, _, _>;
    DUP9 = stack::dup::<9, _, _>;
    DUP10 = stack::dup::<10, _, _>;
    DUP11 = stack::dup::<11, _, _>;
    DUP12 = stack::dup::<12, _, _>;
    DUP13 = stack::dup::<13, _, _>;
    DUP14 = stack::dup::<14, _, _>;
    DUP15 = stack::dup::<15, _, _>;
    DUP16 = stack::dup::<16, _, _>;

    SWAP1 = stack::swap::<1, _, _>;
    SWAP2 = stack::swap::<2, _, _>;
    SWAP3 = stack::swap::<3, _, _>;
    SWAP4 = stack::swap::<4, _, _>;
    SWAP5 = stack::swap::<5, _, _>;
    SWAP6 = stack::swap::<6, _, _>;
    SWAP7 = stack::swap::<7, _, _>;
    SWAP8 = stack::swap::<8, _, _>;
    SWAP9 = stack::swap::<9, _, _>;
    SWAP10 = stack::swap::<10, _, _>;
    SWAP11 = stack::swap::<11, _, _>;
    SWAP12 = stack::swap::<12, _, _>;
    SWAP13 = stack::swap::<13, _, _>;
    SWAP14 = stack::swap::<14, _, _>;
    SWAP15 = stack::swap::<15, _, _>;
    SWAP16 = stack::swap::<16, _, _>;

    LOG0 = host::log::<0, _>;
    LOG1 = host::log::<1, _>;
    LOG2 = host::log::<2, _>;
    LOG3 = host::log::<3, _>;
    LOG4 = host::log::<4, _>;

    CREATE = contract::create::<_, false, _>;
    CALL = contract::call;
    CALLCODE = contract::call_code;
    RETURN = control::ret;
    DELEGATECALL = contract::delegate_call;
    CREATE2 = contract::create::<_, true, _>;

    STATICCALL = contract::static_call;
    REVERT = control::revert;
    INVALID = control::invalid;
    SELFDESTRUCT = host::selfdestruct;
    }

    table
}

/// Returns the tail call instruction table for the given interpreter types and host.
#[inline]
pub const fn instruction_table_tail<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    const {
        macro_rules! wrap {
            ($($idx:expr),* $(,)?) => {
                [
                    $(
                        tail_call_instr::<$idx, H, WIRE>,
                    )*
                ]
            };
        }
        #[rustfmt::skip]
        let x = wrap!(
            0,   1,   2,   3,   4,   5,   6,   7,   8,   9,  10,  11,  12,  13,  14,  15,
            16,  17,  18,  19,  20,  21,  22,  23,  24,  25,  26,  27,  28,  29,  30,  31,
            32,  33,  34,  35,  36,  37,  38,  39,  40,  41,  42,  43,  44,  45,  46,  47,
            48,  49,  50,  51,  52,  53,  54,  55,  56,  57,  58,  59,  60,  61,  62,  63,
            64,  65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,  77,  78,  79,
            80,  81,  82,  83,  84,  85,  86,  87,  88,  89,  90,  91,  92,  93,  94,  95,
            96,  97,  98,  99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
           112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
           128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143,
           144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
           160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175,
           176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191,
           192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207,
           208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223,
           224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239,
           240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        );
        x
    }
}

pub(crate) fn tail_call_instr<const OP: u8, H: Host + ?Sized, W: InterpreterTypes>(
    mut context: InstructionContext<'_, H, W>,
) -> InstructionReturn {
    let ret = (const { instruction_table::<W, H>()[OP as usize] })(context.reborrow());
    if !ret.can_continue() {
        return ret;
    }

    let opcode = context.interpreter.bytecode.opcode();
    context.interpreter.bytecode.relative_jump(1);
    (const { &instruction_table_tail::<W, H>() })[opcode as usize](context)
}

#[cfg(test)]
mod tests {
    use super::instruction_table;
    use crate::{host::DummyHost, interpreter::EthInterpreter};
    use bytecode::opcode::*;

    #[test]
    fn all_instructions_and_opcodes_used() {
        // known unknown instruction we compare it with other instructions from table.
        let unknown_instruction = 0x0C_usize;
        let instr_table = instruction_table::<EthInterpreter, DummyHost>();

        let unknown_istr = instr_table[unknown_instruction];
        for (i, instr) in instr_table.iter().enumerate() {
            let is_opcode_unknown = OpCode::new(i as u8).is_none();
            //
            let is_instr_unknown = std::ptr::fn_addr_eq(*instr, unknown_istr);
            assert_eq!(
                is_instr_unknown, is_opcode_unknown,
                "Opcode 0x{i:X?} is not handled",
            );
        }
    }
}
