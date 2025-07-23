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
    interpreter_types::{InterpreterTypes, Jumps, LoopControl},
    Host, InstructionContext,
};

/// EVM opcode function signature.
pub type Instruction<W, H> = fn(InstructionContext<'_, H, W>);

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<W, H> = [Instruction<W, H>; 256];

/// Returns the default instruction table for the given interpreter types and host.
#[inline]
pub const fn instruction_table<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    const { instruction_table_impl::<WIRE, H>() }
}

const fn instruction_table_impl<WIRE: InterpreterTypes, H: Host + ?Sized>(
) -> [Instruction<WIRE, H>; 256] {
    use bytecode::opcode::*;
    let mut table = [control::unknown as Instruction<WIRE, H>; 256];

    table[STOP as usize] = control::stop;
    table[ADD as usize] = arithmetic::add;
    table[MUL as usize] = arithmetic::mul;
    table[SUB as usize] = arithmetic::sub;
    table[DIV as usize] = arithmetic::div;
    table[SDIV as usize] = arithmetic::sdiv;
    table[MOD as usize] = arithmetic::rem;
    table[SMOD as usize] = arithmetic::smod;
    table[ADDMOD as usize] = arithmetic::addmod;
    table[MULMOD as usize] = arithmetic::mulmod;
    table[EXP as usize] = arithmetic::exp;
    table[SIGNEXTEND as usize] = arithmetic::signextend;

    table[LT as usize] = bitwise::lt;
    table[GT as usize] = bitwise::gt;
    table[SLT as usize] = bitwise::slt;
    table[SGT as usize] = bitwise::sgt;
    table[EQ as usize] = bitwise::eq;
    table[ISZERO as usize] = bitwise::iszero;
    table[AND as usize] = bitwise::bitand;
    table[OR as usize] = bitwise::bitor;
    table[XOR as usize] = bitwise::bitxor;
    table[NOT as usize] = bitwise::not;
    table[BYTE as usize] = bitwise::byte;
    table[SHL as usize] = bitwise::shl;
    table[SHR as usize] = bitwise::shr;
    table[SAR as usize] = bitwise::sar;
    table[CLZ as usize] = bitwise::clz;

    table[KECCAK256 as usize] = system::keccak256;

    table[ADDRESS as usize] = system::address;
    table[BALANCE as usize] = host::balance;
    table[ORIGIN as usize] = tx_info::origin;
    table[CALLER as usize] = system::caller;
    table[CALLVALUE as usize] = system::callvalue;
    table[CALLDATALOAD as usize] = system::calldataload;
    table[CALLDATASIZE as usize] = system::calldatasize;
    table[CALLDATACOPY as usize] = system::calldatacopy;
    table[CODESIZE as usize] = system::codesize;
    table[CODECOPY as usize] = system::codecopy;

    table[GASPRICE as usize] = tx_info::gasprice;
    table[EXTCODESIZE as usize] = host::extcodesize;
    table[EXTCODECOPY as usize] = host::extcodecopy;
    table[RETURNDATASIZE as usize] = system::returndatasize;
    table[RETURNDATACOPY as usize] = system::returndatacopy;
    table[EXTCODEHASH as usize] = host::extcodehash;
    table[BLOCKHASH as usize] = host::blockhash;
    table[COINBASE as usize] = block_info::coinbase;
    table[TIMESTAMP as usize] = block_info::timestamp;
    table[NUMBER as usize] = block_info::block_number;
    table[DIFFICULTY as usize] = block_info::difficulty;
    table[GASLIMIT as usize] = block_info::gaslimit;
    table[CHAINID as usize] = block_info::chainid;
    table[SELFBALANCE as usize] = host::selfbalance;
    table[BASEFEE as usize] = block_info::basefee;
    table[BLOBHASH as usize] = tx_info::blob_hash;
    table[BLOBBASEFEE as usize] = block_info::blob_basefee;

    table[POP as usize] = stack::pop;
    table[MLOAD as usize] = memory::mload;
    table[MSTORE as usize] = memory::mstore;
    table[MSTORE8 as usize] = memory::mstore8;
    table[SLOAD as usize] = host::sload;
    table[SSTORE as usize] = host::sstore;
    table[JUMP as usize] = control::jump;
    table[JUMPI as usize] = control::jumpi;
    table[PC as usize] = control::pc;
    table[MSIZE as usize] = memory::msize;
    table[GAS as usize] = system::gas;
    table[JUMPDEST as usize] = control::jumpdest;
    table[TLOAD as usize] = host::tload;
    table[TSTORE as usize] = host::tstore;
    table[MCOPY as usize] = memory::mcopy;

    table[PUSH0 as usize] = stack::push0;
    table[PUSH1 as usize] = stack::push::<1, _, _>;
    table[PUSH2 as usize] = stack::push::<2, _, _>;
    table[PUSH3 as usize] = stack::push::<3, _, _>;
    table[PUSH4 as usize] = stack::push::<4, _, _>;
    table[PUSH5 as usize] = stack::push::<5, _, _>;
    table[PUSH6 as usize] = stack::push::<6, _, _>;
    table[PUSH7 as usize] = stack::push::<7, _, _>;
    table[PUSH8 as usize] = stack::push::<8, _, _>;
    table[PUSH9 as usize] = stack::push::<9, _, _>;
    table[PUSH10 as usize] = stack::push::<10, _, _>;
    table[PUSH11 as usize] = stack::push::<11, _, _>;
    table[PUSH12 as usize] = stack::push::<12, _, _>;
    table[PUSH13 as usize] = stack::push::<13, _, _>;
    table[PUSH14 as usize] = stack::push::<14, _, _>;
    table[PUSH15 as usize] = stack::push::<15, _, _>;
    table[PUSH16 as usize] = stack::push::<16, _, _>;
    table[PUSH17 as usize] = stack::push::<17, _, _>;
    table[PUSH18 as usize] = stack::push::<18, _, _>;
    table[PUSH19 as usize] = stack::push::<19, _, _>;
    table[PUSH20 as usize] = stack::push::<20, _, _>;
    table[PUSH21 as usize] = stack::push::<21, _, _>;
    table[PUSH22 as usize] = stack::push::<22, _, _>;
    table[PUSH23 as usize] = stack::push::<23, _, _>;
    table[PUSH24 as usize] = stack::push::<24, _, _>;
    table[PUSH25 as usize] = stack::push::<25, _, _>;
    table[PUSH26 as usize] = stack::push::<26, _, _>;
    table[PUSH27 as usize] = stack::push::<27, _, _>;
    table[PUSH28 as usize] = stack::push::<28, _, _>;
    table[PUSH29 as usize] = stack::push::<29, _, _>;
    table[PUSH30 as usize] = stack::push::<30, _, _>;
    table[PUSH31 as usize] = stack::push::<31, _, _>;
    table[PUSH32 as usize] = stack::push::<32, _, _>;

    table[DUP1 as usize] = stack::dup::<1, _, _>;
    table[DUP2 as usize] = stack::dup::<2, _, _>;
    table[DUP3 as usize] = stack::dup::<3, _, _>;
    table[DUP4 as usize] = stack::dup::<4, _, _>;
    table[DUP5 as usize] = stack::dup::<5, _, _>;
    table[DUP6 as usize] = stack::dup::<6, _, _>;
    table[DUP7 as usize] = stack::dup::<7, _, _>;
    table[DUP8 as usize] = stack::dup::<8, _, _>;
    table[DUP9 as usize] = stack::dup::<9, _, _>;
    table[DUP10 as usize] = stack::dup::<10, _, _>;
    table[DUP11 as usize] = stack::dup::<11, _, _>;
    table[DUP12 as usize] = stack::dup::<12, _, _>;
    table[DUP13 as usize] = stack::dup::<13, _, _>;
    table[DUP14 as usize] = stack::dup::<14, _, _>;
    table[DUP15 as usize] = stack::dup::<15, _, _>;
    table[DUP16 as usize] = stack::dup::<16, _, _>;

    table[SWAP1 as usize] = stack::swap::<1, _, _>;
    table[SWAP2 as usize] = stack::swap::<2, _, _>;
    table[SWAP3 as usize] = stack::swap::<3, _, _>;
    table[SWAP4 as usize] = stack::swap::<4, _, _>;
    table[SWAP5 as usize] = stack::swap::<5, _, _>;
    table[SWAP6 as usize] = stack::swap::<6, _, _>;
    table[SWAP7 as usize] = stack::swap::<7, _, _>;
    table[SWAP8 as usize] = stack::swap::<8, _, _>;
    table[SWAP9 as usize] = stack::swap::<9, _, _>;
    table[SWAP10 as usize] = stack::swap::<10, _, _>;
    table[SWAP11 as usize] = stack::swap::<11, _, _>;
    table[SWAP12 as usize] = stack::swap::<12, _, _>;
    table[SWAP13 as usize] = stack::swap::<13, _, _>;
    table[SWAP14 as usize] = stack::swap::<14, _, _>;
    table[SWAP15 as usize] = stack::swap::<15, _, _>;
    table[SWAP16 as usize] = stack::swap::<16, _, _>;

    table[LOG0 as usize] = host::log::<0, _>;
    table[LOG1 as usize] = host::log::<1, _>;
    table[LOG2 as usize] = host::log::<2, _>;
    table[LOG3 as usize] = host::log::<3, _>;
    table[LOG4 as usize] = host::log::<4, _>;

    table[CREATE as usize] = contract::create::<_, false, _>;
    table[CALL as usize] = contract::call;
    table[CALLCODE as usize] = contract::call_code;
    table[RETURN as usize] = control::ret;
    table[DELEGATECALL as usize] = contract::delegate_call;
    table[CREATE2 as usize] = contract::create::<_, true, _>;

    table[STATICCALL as usize] = contract::static_call;
    table[REVERT as usize] = control::revert;
    table[INVALID as usize] = control::invalid;
    table[SELFDESTRUCT as usize] = host::selfdestruct;
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
) {
    (const { instruction_table::<W, H>()[OP as usize] })(context.reborrow());

    if context.interpreter.bytecode.is_end() {
        return;
    }

    let instruction_table = const { &instruction_table_tail::<W, H>() };
    let opcode = context.interpreter.bytecode.opcode();
    context.interpreter.bytecode.relative_jump(1);
    become instruction_table[opcode as usize](context);
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
