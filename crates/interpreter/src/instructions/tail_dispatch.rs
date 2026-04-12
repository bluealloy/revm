//! Tail-call dispatch for the EVM interpreter.
//!
//! Uses nightly `become` keyword with `extern "rust-preserve-none"` calling convention
//! to achieve threaded-code-style dispatch where each opcode handler tail-calls the next.
//!
//! Key ideas from <https://www.mattkeeter.com/blog/2026-04-05-tailcall/>:
//! - `extern "rust-preserve-none"`: maximizes register usage on x86-64 (critical for perf)
//! - `become`: guaranteed tail call elimination, no stack growth
//! - Table pointer threaded through arguments: stays in a register

use crate::{
    instructions::InstructionTable,
    interpreter_types::{InterpreterTypes, Jumps, LoopControl},
    Host, InstructionContext, Interpreter,
};

/// Tail-call handler: one per opcode slot.
///
/// Uses `extern "rust-preserve-none"` so the compiler keeps all arguments in registers
/// across the tail-call chain. On x86-64 the default calling convention only uses ~6
/// argument registers; `rust-preserve-none` lets the compiler use nearly all GPRs.
///
/// The `tail_table` argument is a raw pointer to the 256-entry table of these handlers.
/// We pass a raw pointer (instead of `&[…; 256]`) to break the recursive type alias cycle.
type TailHandler<W, H> = extern "rust-preserve-none" fn(
    interpreter: &mut Interpreter<W>,
    host: &mut H,
    table: &InstructionTable<W, H>,
    tail_table: *const u8, // actually *const TailHandler<W, H>, but we cast to break recursion
);

/// Cast the raw `tail_table` pointer back to a typed slice of handlers.
///
/// # Safety
/// Caller must ensure `ptr` points to a valid `[TailHandler<W,H>; 256]`.
#[inline(always)]
unsafe fn tail_table_ref<'a, W: InterpreterTypes, H: Host + ?Sized>(
    ptr: *const u8,
) -> &'a [TailHandler<W, H>; 256] {
    &*(ptr as *const [TailHandler<W, H>; 256])
}

/// Dispatch to the next opcode using a tail call.
///
/// Reads the current opcode, advances the PC, charges static gas, and `become`s
/// the next handler. This is the core of the threaded dispatch.
#[inline(always)]
extern "rust-preserve-none" fn dispatch_next<W: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<W>,
    host: &mut H,
    table: &InstructionTable<W, H>,
    tail_table: *const u8,
) {
    if !interpreter.bytecode.is_not_end() {
        return;
    }

    let opcode = interpreter.bytecode.opcode();
    interpreter.bytecode.relative_jump(1);

    let instruction = unsafe { table.get_unchecked(opcode as usize) };
    if interpreter.gas.record_cost_unsafe(instruction.static_gas()) {
        interpreter.halt_oog();
        return;
    }

    let tt = unsafe { tail_table_ref::<W, H>(tail_table) };
    let handler = unsafe { *tt.get_unchecked(opcode as usize) };
    become handler(interpreter, host, table, tail_table);
}

/// Generate a tail-call wrapper for a regular instruction function.
///
/// The wrapper:
/// 1. Calls the original instruction fn
/// 2. Tail-calls `dispatch_next` to continue to the next opcode
macro_rules! tail_wrapper {
    ($name:ident, $instr_fn:expr) => {
        extern "rust-preserve-none" fn $name<W: InterpreterTypes, H: Host + ?Sized>(
            interpreter: &mut Interpreter<W>,
            host: &mut H,
            table: &InstructionTable<W, H>,
            tail_table: *const u8,
        ) {
            let context = InstructionContext { interpreter, host };
            ($instr_fn)(context);
            become dispatch_next(interpreter, host, table, tail_table);
        }
    };
}

// Generate all tail-call wrappers
tail_wrapper!(tail_stop, super::control::stop);
tail_wrapper!(tail_add, super::arithmetic::add);
tail_wrapper!(tail_mul, super::arithmetic::mul);
tail_wrapper!(tail_sub, super::arithmetic::sub);
tail_wrapper!(tail_div, super::arithmetic::div);
tail_wrapper!(tail_sdiv, super::arithmetic::sdiv);
tail_wrapper!(tail_rem, super::arithmetic::rem);
tail_wrapper!(tail_smod, super::arithmetic::smod);
tail_wrapper!(tail_addmod, super::arithmetic::addmod);
tail_wrapper!(tail_mulmod, super::arithmetic::mulmod);
tail_wrapper!(tail_exp, super::arithmetic::exp);
tail_wrapper!(tail_signextend, super::arithmetic::signextend);
tail_wrapper!(tail_lt, super::bitwise::lt);
tail_wrapper!(tail_gt, super::bitwise::gt);
tail_wrapper!(tail_slt, super::bitwise::slt);
tail_wrapper!(tail_sgt, super::bitwise::sgt);
tail_wrapper!(tail_eq, super::bitwise::eq);
tail_wrapper!(tail_iszero, super::bitwise::iszero);
tail_wrapper!(tail_bitand, super::bitwise::bitand);
tail_wrapper!(tail_bitor, super::bitwise::bitor);
tail_wrapper!(tail_bitxor, super::bitwise::bitxor);
tail_wrapper!(tail_not, super::bitwise::not);
tail_wrapper!(tail_byte, super::bitwise::byte);
tail_wrapper!(tail_shl, super::bitwise::shl);
tail_wrapper!(tail_shr, super::bitwise::shr);
tail_wrapper!(tail_sar, super::bitwise::sar);
tail_wrapper!(tail_clz, super::bitwise::clz);
tail_wrapper!(tail_keccak256, super::system::keccak256);
tail_wrapper!(tail_address, super::system::address);
tail_wrapper!(tail_balance, super::host::balance);
tail_wrapper!(tail_origin, super::tx_info::origin);
tail_wrapper!(tail_caller, super::system::caller);
tail_wrapper!(tail_callvalue, super::system::callvalue);
tail_wrapper!(tail_calldataload, super::system::calldataload);
tail_wrapper!(tail_calldatasize, super::system::calldatasize);
tail_wrapper!(tail_calldatacopy, super::system::calldatacopy);
tail_wrapper!(tail_codesize, super::system::codesize);
tail_wrapper!(tail_codecopy, super::system::codecopy);
tail_wrapper!(tail_gasprice, super::tx_info::gasprice);
tail_wrapper!(tail_extcodesize, super::host::extcodesize);
tail_wrapper!(tail_extcodecopy, super::host::extcodecopy);
tail_wrapper!(tail_returndatasize, super::system::returndatasize);
tail_wrapper!(tail_returndatacopy, super::system::returndatacopy);
tail_wrapper!(tail_extcodehash, super::host::extcodehash);
tail_wrapper!(tail_blockhash, super::host::blockhash);
tail_wrapper!(tail_coinbase, super::block_info::coinbase);
tail_wrapper!(tail_timestamp, super::block_info::timestamp);
tail_wrapper!(tail_block_number, super::block_info::block_number);
tail_wrapper!(tail_difficulty, super::block_info::difficulty);
tail_wrapper!(tail_gaslimit, super::block_info::gaslimit);
tail_wrapper!(tail_chainid, super::block_info::chainid);
tail_wrapper!(tail_selfbalance, super::host::selfbalance);
tail_wrapper!(tail_basefee, super::block_info::basefee);
tail_wrapper!(tail_blob_hash, super::tx_info::blob_hash);
tail_wrapper!(tail_blob_basefee, super::block_info::blob_basefee);
tail_wrapper!(tail_slot_num, super::block_info::slot_num);
tail_wrapper!(tail_pop, super::stack::pop);
tail_wrapper!(tail_mload, super::memory::mload);
tail_wrapper!(tail_mstore, super::memory::mstore);
tail_wrapper!(tail_mstore8, super::memory::mstore8);
tail_wrapper!(tail_sload, super::host::sload);
tail_wrapper!(tail_sstore, super::host::sstore);
tail_wrapper!(tail_jump, super::control::jump);
tail_wrapper!(tail_jumpi, super::control::jumpi);
tail_wrapper!(tail_pc, super::control::pc);
tail_wrapper!(tail_msize, super::memory::msize);
tail_wrapper!(tail_gas, super::system::gas);
tail_wrapper!(tail_jumpdest, super::control::jumpdest);
tail_wrapper!(tail_tload, super::host::tload);
tail_wrapper!(tail_tstore, super::host::tstore);
tail_wrapper!(tail_mcopy, super::memory::mcopy);
tail_wrapper!(tail_push0, super::stack::push0);

// PUSH1-PUSH32
macro_rules! tail_push_wrappers {
    ($($n:literal),*) => {
        $(
            paste::paste! {
                tail_wrapper!([<tail_push $n>], super::stack::push::<$n, _, _>);
            }
        )*
    };
}
tail_push_wrappers!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);

// DUP1-DUP16
macro_rules! tail_dup_wrappers {
    ($($n:literal),*) => {
        $(
            paste::paste! {
                tail_wrapper!([<tail_dup $n>], super::stack::dup::<$n, _, _>);
            }
        )*
    };
}
tail_dup_wrappers!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

// SWAP1-SWAP16
macro_rules! tail_swap_wrappers {
    ($($n:literal),*) => {
        $(
            paste::paste! {
                tail_wrapper!([<tail_swap $n>], super::stack::swap::<$n, _, _>);
            }
        )*
    };
}
tail_swap_wrappers!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

tail_wrapper!(tail_dupn, super::stack::dupn);
tail_wrapper!(tail_swapn, super::stack::swapn);
tail_wrapper!(tail_exchange, super::stack::exchange);

// LOG0-LOG4
tail_wrapper!(tail_log0, super::host::log::<0, _>);
tail_wrapper!(tail_log1, super::host::log::<1, _>);
tail_wrapper!(tail_log2, super::host::log::<2, _>);
tail_wrapper!(tail_log3, super::host::log::<3, _>);
tail_wrapper!(tail_log4, super::host::log::<4, _>);

tail_wrapper!(tail_create, super::contract::create::<_, false, _>);
tail_wrapper!(tail_call, super::contract::call);
tail_wrapper!(tail_call_code, super::contract::call_code);
tail_wrapper!(tail_ret, super::control::ret);
tail_wrapper!(tail_delegate_call, super::contract::delegate_call);
tail_wrapper!(tail_create2, super::contract::create::<_, true, _>);
tail_wrapper!(tail_static_call, super::contract::static_call);
tail_wrapper!(tail_revert, super::control::revert);
tail_wrapper!(tail_invalid, super::control::invalid);
tail_wrapper!(tail_selfdestruct, super::host::selfdestruct);
tail_wrapper!(tail_unknown, super::control::unknown);

/// Build the tail-call dispatch table mapping opcodes to tail-call wrappers.
const fn build_tail_table<W: InterpreterTypes, H: Host + ?Sized>() -> [TailHandler<W, H>; 256] {
    use bytecode::opcode::*;
    let mut table: [TailHandler<W, H>; 256] = [tail_unknown::<W, H>; 256];

    table[STOP as usize] = tail_stop;
    table[ADD as usize] = tail_add;
    table[MUL as usize] = tail_mul;
    table[SUB as usize] = tail_sub;
    table[DIV as usize] = tail_div;
    table[SDIV as usize] = tail_sdiv;
    table[MOD as usize] = tail_rem;
    table[SMOD as usize] = tail_smod;
    table[ADDMOD as usize] = tail_addmod;
    table[MULMOD as usize] = tail_mulmod;
    table[EXP as usize] = tail_exp;
    table[SIGNEXTEND as usize] = tail_signextend;

    table[LT as usize] = tail_lt;
    table[GT as usize] = tail_gt;
    table[SLT as usize] = tail_slt;
    table[SGT as usize] = tail_sgt;
    table[EQ as usize] = tail_eq;
    table[ISZERO as usize] = tail_iszero;
    table[AND as usize] = tail_bitand;
    table[OR as usize] = tail_bitor;
    table[XOR as usize] = tail_bitxor;
    table[NOT as usize] = tail_not;
    table[BYTE as usize] = tail_byte;
    table[SHL as usize] = tail_shl;
    table[SHR as usize] = tail_shr;
    table[SAR as usize] = tail_sar;
    table[CLZ as usize] = tail_clz;

    table[KECCAK256 as usize] = tail_keccak256;

    table[ADDRESS as usize] = tail_address;
    table[BALANCE as usize] = tail_balance;
    table[ORIGIN as usize] = tail_origin;
    table[CALLER as usize] = tail_caller;
    table[CALLVALUE as usize] = tail_callvalue;
    table[CALLDATALOAD as usize] = tail_calldataload;
    table[CALLDATASIZE as usize] = tail_calldatasize;
    table[CALLDATACOPY as usize] = tail_calldatacopy;
    table[CODESIZE as usize] = tail_codesize;
    table[CODECOPY as usize] = tail_codecopy;

    table[GASPRICE as usize] = tail_gasprice;
    table[EXTCODESIZE as usize] = tail_extcodesize;
    table[EXTCODECOPY as usize] = tail_extcodecopy;
    table[RETURNDATASIZE as usize] = tail_returndatasize;
    table[RETURNDATACOPY as usize] = tail_returndatacopy;
    table[EXTCODEHASH as usize] = tail_extcodehash;
    table[BLOCKHASH as usize] = tail_blockhash;
    table[COINBASE as usize] = tail_coinbase;
    table[TIMESTAMP as usize] = tail_timestamp;
    table[NUMBER as usize] = tail_block_number;
    table[DIFFICULTY as usize] = tail_difficulty;
    table[GASLIMIT as usize] = tail_gaslimit;
    table[CHAINID as usize] = tail_chainid;
    table[SELFBALANCE as usize] = tail_selfbalance;
    table[BASEFEE as usize] = tail_basefee;
    table[BLOBHASH as usize] = tail_blob_hash;
    table[BLOBBASEFEE as usize] = tail_blob_basefee;
    table[SLOTNUM as usize] = tail_slot_num;

    table[POP as usize] = tail_pop;
    table[MLOAD as usize] = tail_mload;
    table[MSTORE as usize] = tail_mstore;
    table[MSTORE8 as usize] = tail_mstore8;
    table[SLOAD as usize] = tail_sload;
    table[SSTORE as usize] = tail_sstore;
    table[JUMP as usize] = tail_jump;
    table[JUMPI as usize] = tail_jumpi;
    table[PC as usize] = tail_pc;
    table[MSIZE as usize] = tail_msize;
    table[GAS as usize] = tail_gas;
    table[JUMPDEST as usize] = tail_jumpdest;
    table[TLOAD as usize] = tail_tload;
    table[TSTORE as usize] = tail_tstore;
    table[MCOPY as usize] = tail_mcopy;

    table[PUSH0 as usize] = tail_push0;
    paste::paste! {
        table[PUSH1 as usize] = [<tail_push 1>];
        table[PUSH2 as usize] = [<tail_push 2>];
        table[PUSH3 as usize] = [<tail_push 3>];
        table[PUSH4 as usize] = [<tail_push 4>];
        table[PUSH5 as usize] = [<tail_push 5>];
        table[PUSH6 as usize] = [<tail_push 6>];
        table[PUSH7 as usize] = [<tail_push 7>];
        table[PUSH8 as usize] = [<tail_push 8>];
        table[PUSH9 as usize] = [<tail_push 9>];
        table[PUSH10 as usize] = [<tail_push 10>];
        table[PUSH11 as usize] = [<tail_push 11>];
        table[PUSH12 as usize] = [<tail_push 12>];
        table[PUSH13 as usize] = [<tail_push 13>];
        table[PUSH14 as usize] = [<tail_push 14>];
        table[PUSH15 as usize] = [<tail_push 15>];
        table[PUSH16 as usize] = [<tail_push 16>];
        table[PUSH17 as usize] = [<tail_push 17>];
        table[PUSH18 as usize] = [<tail_push 18>];
        table[PUSH19 as usize] = [<tail_push 19>];
        table[PUSH20 as usize] = [<tail_push 20>];
        table[PUSH21 as usize] = [<tail_push 21>];
        table[PUSH22 as usize] = [<tail_push 22>];
        table[PUSH23 as usize] = [<tail_push 23>];
        table[PUSH24 as usize] = [<tail_push 24>];
        table[PUSH25 as usize] = [<tail_push 25>];
        table[PUSH26 as usize] = [<tail_push 26>];
        table[PUSH27 as usize] = [<tail_push 27>];
        table[PUSH28 as usize] = [<tail_push 28>];
        table[PUSH29 as usize] = [<tail_push 29>];
        table[PUSH30 as usize] = [<tail_push 30>];
        table[PUSH31 as usize] = [<tail_push 31>];
        table[PUSH32 as usize] = [<tail_push 32>];
    }

    paste::paste! {
        table[DUP1 as usize] = [<tail_dup 1>];
        table[DUP2 as usize] = [<tail_dup 2>];
        table[DUP3 as usize] = [<tail_dup 3>];
        table[DUP4 as usize] = [<tail_dup 4>];
        table[DUP5 as usize] = [<tail_dup 5>];
        table[DUP6 as usize] = [<tail_dup 6>];
        table[DUP7 as usize] = [<tail_dup 7>];
        table[DUP8 as usize] = [<tail_dup 8>];
        table[DUP9 as usize] = [<tail_dup 9>];
        table[DUP10 as usize] = [<tail_dup 10>];
        table[DUP11 as usize] = [<tail_dup 11>];
        table[DUP12 as usize] = [<tail_dup 12>];
        table[DUP13 as usize] = [<tail_dup 13>];
        table[DUP14 as usize] = [<tail_dup 14>];
        table[DUP15 as usize] = [<tail_dup 15>];
        table[DUP16 as usize] = [<tail_dup 16>];
    }

    paste::paste! {
        table[SWAP1 as usize] = [<tail_swap 1>];
        table[SWAP2 as usize] = [<tail_swap 2>];
        table[SWAP3 as usize] = [<tail_swap 3>];
        table[SWAP4 as usize] = [<tail_swap 4>];
        table[SWAP5 as usize] = [<tail_swap 5>];
        table[SWAP6 as usize] = [<tail_swap 6>];
        table[SWAP7 as usize] = [<tail_swap 7>];
        table[SWAP8 as usize] = [<tail_swap 8>];
        table[SWAP9 as usize] = [<tail_swap 9>];
        table[SWAP10 as usize] = [<tail_swap 10>];
        table[SWAP11 as usize] = [<tail_swap 11>];
        table[SWAP12 as usize] = [<tail_swap 12>];
        table[SWAP13 as usize] = [<tail_swap 13>];
        table[SWAP14 as usize] = [<tail_swap 14>];
        table[SWAP15 as usize] = [<tail_swap 15>];
        table[SWAP16 as usize] = [<tail_swap 16>];
    }

    table[DUPN as usize] = tail_dupn;
    table[SWAPN as usize] = tail_swapn;
    table[EXCHANGE as usize] = tail_exchange;

    table[LOG0 as usize] = tail_log0;
    table[LOG1 as usize] = tail_log1;
    table[LOG2 as usize] = tail_log2;
    table[LOG3 as usize] = tail_log3;
    table[LOG4 as usize] = tail_log4;

    table[CREATE as usize] = tail_create;
    table[CALL as usize] = tail_call;
    table[CALLCODE as usize] = tail_call_code;
    table[RETURN as usize] = tail_ret;
    table[DELEGATECALL as usize] = tail_delegate_call;
    table[CREATE2 as usize] = tail_create2;
    table[STATICCALL as usize] = tail_static_call;
    table[REVERT as usize] = tail_revert;
    table[INVALID as usize] = tail_invalid;
    table[SELFDESTRUCT as usize] = tail_selfdestruct;

    table
}

/// Run the interpreter using tail-call dispatch.
///
/// Uses a const-evaluated tail table so no per-call setup cost.
#[inline]
pub fn run_tail_call<IW: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IW>,
    instruction_table: &InstructionTable<IW, H>,
    host: &mut H,
) {
    let tail_table = const { &build_tail_table::<IW, H>() };
    let tail_ptr = tail_table.as_ptr() as *const u8;
    dispatch_next(interpreter, host, instruction_table, tail_ptr);
}
