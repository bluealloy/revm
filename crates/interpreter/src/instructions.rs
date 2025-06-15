//! EVM opcode implementations.

#[macro_use]
pub mod macros;
pub mod arithmetic;
pub mod bitwise;
pub mod block_info;
pub mod contract;
pub mod control;
pub mod data;
pub mod host;
pub mod i256;
pub mod memory;
pub mod stack;
pub mod system;
pub mod tx_info;
pub mod utility;

use crate::{interpreter_types::InterpreterTypes, Host, InstructionContext};
use core::future::Future;
use core::pin::Pin;
use std::boxed::Box;

/// Boxed future used by asynchronous opcode implementations.
pub type BoxFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

/// Helper that boxes any future into the common `BoxFuture` type.
#[inline]
pub fn boxed<'a, Fut>(fut: Fut) -> BoxFuture<'a>
where
    Fut: Future<Output = ()> + 'a,
{
    Box::pin(fut)
}

/// Macro that converts a synchronous **or** asynchronous function implementing an opcode into the
/// required `Instruction` signature.
#[macro_export]
macro_rules! wrap {
    ($name:path) => {
        |ctx| $crate::instructions::boxed($name(ctx))
    };
}

/// Wraps a synchronous opcode implementation, producing a function that returns a boxed future.
#[macro_export]
macro_rules! wrap_sync {
    ($name:path) => {
        |ctx| $crate::instructions::boxed(async move { $name(ctx) })
    };
}

/// Wraps an asynchronous opcode implementation (i.e. an `async fn`).
#[macro_export]
macro_rules! wrap_async {
    ($name:path) => {
        |ctx| $crate::instructions::boxed($name(ctx))
    };
}

/// EVM opcode function signature.
pub type Instruction<W, H> = for<'a> fn(InstructionContext<'a, H, W>) -> BoxFuture<'a>;

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<W, H> = [Instruction<W, H>; 256];

/// Returns the instruction table for the given spec.
pub fn instruction_table<WIRE: InterpreterTypes, H: Host + ?Sized>() -> [Instruction<WIRE, H>; 256]
{
    use bytecode::opcode::*;
    // Start with all slots pointing to `unknown` opcode implementation.
    let mut table: [Instruction<WIRE, H>; 256] = [wrap_sync!(control::unknown::<_, _>); 256];

    table[STOP as usize] = wrap_sync!(control::stop::<_, _>);
    table[ADD as usize] = wrap_sync!(arithmetic::add::<_, _>);
    table[MUL as usize] = wrap_sync!(arithmetic::mul::<_, _>);
    table[SUB as usize] = wrap_sync!(arithmetic::sub::<_, _>);
    table[DIV as usize] = wrap_sync!(arithmetic::div::<_, _>);
    table[SDIV as usize] = wrap_sync!(arithmetic::sdiv::<_, _>);
    table[MOD as usize] = wrap_sync!(arithmetic::rem::<_, _>);
    table[SMOD as usize] = wrap_sync!(arithmetic::smod::<_, _>);
    table[ADDMOD as usize] = wrap_sync!(arithmetic::addmod::<_, _>);
    table[MULMOD as usize] = wrap_sync!(arithmetic::mulmod::<_, _>);
    table[EXP as usize] = wrap_sync!(arithmetic::exp::<_, _>);
    table[SIGNEXTEND as usize] = wrap_sync!(arithmetic::signextend::<_, _>);

    table[LT as usize] = wrap_sync!(bitwise::lt::<_, _>);
    table[GT as usize] = wrap_sync!(bitwise::gt::<_, _>);
    table[SLT as usize] = wrap_sync!(bitwise::slt::<_, _>);
    table[SGT as usize] = wrap_sync!(bitwise::sgt::<_, _>);
    table[EQ as usize] = wrap_sync!(bitwise::eq::<_, _>);
    table[ISZERO as usize] = wrap_sync!(bitwise::iszero::<_, _>);
    table[AND as usize] = wrap_sync!(bitwise::bitand::<_, _>);
    table[OR as usize] = wrap_sync!(bitwise::bitor::<_, _>);
    table[XOR as usize] = wrap_sync!(bitwise::bitxor::<_, _>);
    table[NOT as usize] = wrap_sync!(bitwise::not::<_, _>);
    table[BYTE as usize] = wrap_sync!(bitwise::byte::<_, _>);
    table[SHL as usize] = wrap_sync!(bitwise::shl::<_, _>);
    table[SHR as usize] = wrap_sync!(bitwise::shr::<_, _>);
    table[SAR as usize] = wrap_sync!(bitwise::sar::<_, _>);

    table[KECCAK256 as usize] = wrap_sync!(system::keccak256::<_, _>);

    table[ADDRESS as usize] = wrap_sync!(system::address::<_, _>);
    table[BALANCE as usize] = wrap_async!(host::balance::<_, _>);
    table[ORIGIN as usize] = wrap_sync!(tx_info::origin::<_, _>);
    table[CALLER as usize] = wrap_sync!(system::caller::<_, _>);
    table[CALLVALUE as usize] = wrap_sync!(system::callvalue::<_, _>);
    table[CALLDATALOAD as usize] = wrap_sync!(system::calldataload::<_, _>);
    table[CALLDATASIZE as usize] = wrap_sync!(system::calldatasize::<_, _>);
    table[CALLDATACOPY as usize] = wrap_sync!(system::calldatacopy::<_, _>);
    table[CODESIZE as usize] = wrap_sync!(system::codesize::<_, _>);
    table[CODECOPY as usize] = wrap_sync!(system::codecopy::<_, _>);

    table[GASPRICE as usize] = wrap_sync!(tx_info::gasprice::<_, _>);
    table[EXTCODESIZE as usize] = wrap_async!(host::extcodesize::<_, _>);
    table[EXTCODECOPY as usize] = wrap_async!(host::extcodecopy::<_, _>);
    table[RETURNDATASIZE as usize] = wrap_sync!(system::returndatasize::<_, _>);
    table[RETURNDATACOPY as usize] = wrap_sync!(system::returndatacopy::<_, _>);
    table[EXTCODEHASH as usize] = wrap_async!(host::extcodehash::<_, _>);
    table[BLOCKHASH as usize] = wrap_async!(host::blockhash::<_, _>);
    table[COINBASE as usize] = wrap_sync!(block_info::coinbase::<_, _>);
    table[TIMESTAMP as usize] = wrap_sync!(block_info::timestamp::<_, _>);
    table[NUMBER as usize] = wrap_sync!(block_info::block_number::<_, _>);
    table[DIFFICULTY as usize] = wrap_sync!(block_info::difficulty::<_, _>);
    table[GASLIMIT as usize] = wrap_sync!(block_info::gaslimit::<_, _>);
    table[CHAINID as usize] = wrap_sync!(block_info::chainid::<_, _>);
    table[SELFBALANCE as usize] = wrap_async!(host::selfbalance::<_, _>);
    table[BASEFEE as usize] = wrap_sync!(block_info::basefee::<_, _>);
    table[BLOBHASH as usize] = wrap_sync!(tx_info::blob_hash::<_, _>);
    table[BLOBBASEFEE as usize] = wrap_sync!(block_info::blob_basefee::<_, _>);

    table[POP as usize] = wrap_sync!(stack::pop::<_, _>);
    table[MLOAD as usize] = wrap_sync!(memory::mload::<_, _>);
    table[MSTORE as usize] = wrap_sync!(memory::mstore::<_, _>);
    table[MSTORE8 as usize] = wrap_sync!(memory::mstore8::<_, _>);
    table[SLOAD as usize] = wrap_async!(host::sload::<_, _>);
    table[SSTORE as usize] = wrap_async!(host::sstore::<_, _>);
    table[JUMP as usize] = wrap_sync!(control::jump::<_, _>);
    table[JUMPI as usize] = wrap_sync!(control::jumpi::<_, _>);
    table[PC as usize] = wrap_sync!(control::pc::<_, _>);
    table[MSIZE as usize] = wrap_sync!(memory::msize::<_, _>);
    table[GAS as usize] = wrap_sync!(system::gas::<_, _>);
    table[JUMPDEST as usize] = wrap_sync!(control::jumpdest_or_nop::<_, _>);
    table[TLOAD as usize] = wrap_async!(host::tload::<_, _>);
    table[TSTORE as usize] = wrap_async!(host::tstore::<_, _>);
    table[MCOPY as usize] = wrap_sync!(memory::mcopy::<_, _>);

    table[PUSH0 as usize] = wrap_sync!(stack::push0::<_, _>);
    table[PUSH1 as usize] = wrap_sync!(stack::push::<1, _, _>);
    table[PUSH2 as usize] = wrap_sync!(stack::push::<2, _, _>);
    table[PUSH3 as usize] = wrap_sync!(stack::push::<3, _, _>);
    table[PUSH4 as usize] = wrap_sync!(stack::push::<4, _, _>);
    table[PUSH5 as usize] = wrap_sync!(stack::push::<5, _, _>);
    table[PUSH6 as usize] = wrap_sync!(stack::push::<6, _, _>);
    table[PUSH7 as usize] = wrap_sync!(stack::push::<7, _, _>);
    table[PUSH8 as usize] = wrap_sync!(stack::push::<8, _, _>);
    table[PUSH9 as usize] = wrap_sync!(stack::push::<9, _, _>);
    table[PUSH10 as usize] = wrap_sync!(stack::push::<10, _, _>);
    table[PUSH11 as usize] = wrap_sync!(stack::push::<11, _, _>);
    table[PUSH12 as usize] = wrap_sync!(stack::push::<12, _, _>);
    table[PUSH13 as usize] = wrap_sync!(stack::push::<13, _, _>);
    table[PUSH14 as usize] = wrap_sync!(stack::push::<14, _, _>);
    table[PUSH15 as usize] = wrap_sync!(stack::push::<15, _, _>);
    table[PUSH16 as usize] = wrap_sync!(stack::push::<16, _, _>);
    table[PUSH17 as usize] = wrap_sync!(stack::push::<17, _, _>);
    table[PUSH18 as usize] = wrap_sync!(stack::push::<18, _, _>);
    table[PUSH19 as usize] = wrap_sync!(stack::push::<19, _, _>);
    table[PUSH20 as usize] = wrap_sync!(stack::push::<20, _, _>);
    table[PUSH21 as usize] = wrap_sync!(stack::push::<21, _, _>);
    table[PUSH22 as usize] = wrap_sync!(stack::push::<22, _, _>);
    table[PUSH23 as usize] = wrap_sync!(stack::push::<23, _, _>);
    table[PUSH24 as usize] = wrap_sync!(stack::push::<24, _, _>);
    table[PUSH25 as usize] = wrap_sync!(stack::push::<25, _, _>);
    table[PUSH26 as usize] = wrap_sync!(stack::push::<26, _, _>);
    table[PUSH27 as usize] = wrap_sync!(stack::push::<27, _, _>);
    table[PUSH28 as usize] = wrap_sync!(stack::push::<28, _, _>);
    table[PUSH29 as usize] = wrap_sync!(stack::push::<29, _, _>);
    table[PUSH30 as usize] = wrap_sync!(stack::push::<30, _, _>);
    table[PUSH31 as usize] = wrap_sync!(stack::push::<31, _, _>);
    table[PUSH32 as usize] = wrap_sync!(stack::push::<32, _, _>);

    table[DUP1 as usize] = wrap_sync!(stack::dup::<1, _, _>);
    table[DUP2 as usize] = wrap_sync!(stack::dup::<2, _, _>);
    table[DUP3 as usize] = wrap_sync!(stack::dup::<3, _, _>);
    table[DUP4 as usize] = wrap_sync!(stack::dup::<4, _, _>);
    table[DUP5 as usize] = wrap_sync!(stack::dup::<5, _, _>);
    table[DUP6 as usize] = wrap_sync!(stack::dup::<6, _, _>);
    table[DUP7 as usize] = wrap_sync!(stack::dup::<7, _, _>);
    table[DUP8 as usize] = wrap_sync!(stack::dup::<8, _, _>);
    table[DUP9 as usize] = wrap_sync!(stack::dup::<9, _, _>);
    table[DUP10 as usize] = wrap_sync!(stack::dup::<10, _, _>);
    table[DUP11 as usize] = wrap_sync!(stack::dup::<11, _, _>);
    table[DUP12 as usize] = wrap_sync!(stack::dup::<12, _, _>);
    table[DUP13 as usize] = wrap_sync!(stack::dup::<13, _, _>);
    table[DUP14 as usize] = wrap_sync!(stack::dup::<14, _, _>);
    table[DUP15 as usize] = wrap_sync!(stack::dup::<15, _, _>);
    table[DUP16 as usize] = wrap_sync!(stack::dup::<16, _, _>);

    table[SWAP1 as usize] = wrap_sync!(stack::swap::<1, _, _>);
    table[SWAP2 as usize] = wrap_sync!(stack::swap::<2, _, _>);
    table[SWAP3 as usize] = wrap_sync!(stack::swap::<3, _, _>);
    table[SWAP4 as usize] = wrap_sync!(stack::swap::<4, _, _>);
    table[SWAP5 as usize] = wrap_sync!(stack::swap::<5, _, _>);
    table[SWAP6 as usize] = wrap_sync!(stack::swap::<6, _, _>);
    table[SWAP7 as usize] = wrap_sync!(stack::swap::<7, _, _>);
    table[SWAP8 as usize] = wrap_sync!(stack::swap::<8, _, _>);
    table[SWAP9 as usize] = wrap_sync!(stack::swap::<9, _, _>);
    table[SWAP10 as usize] = wrap_sync!(stack::swap::<10, _, _>);
    table[SWAP11 as usize] = wrap_sync!(stack::swap::<11, _, _>);
    table[SWAP12 as usize] = wrap_sync!(stack::swap::<12, _, _>);
    table[SWAP13 as usize] = wrap_sync!(stack::swap::<13, _, _>);
    table[SWAP14 as usize] = wrap_sync!(stack::swap::<14, _, _>);
    table[SWAP15 as usize] = wrap_sync!(stack::swap::<15, _, _>);
    table[SWAP16 as usize] = wrap_sync!(stack::swap::<16, _, _>);

    table[LOG0 as usize] = wrap_sync!(host::log::<0, _>);
    table[LOG1 as usize] = wrap_sync!(host::log::<1, _>);
    table[LOG2 as usize] = wrap_sync!(host::log::<2, _>);
    table[LOG3 as usize] = wrap_sync!(host::log::<3, _>);
    table[LOG4 as usize] = wrap_sync!(host::log::<4, _>);

    table[DATALOAD as usize] = wrap_sync!(data::data_load::<_, _>);
    table[DATALOADN as usize] = wrap_sync!(data::data_loadn::<_, _>);
    table[DATASIZE as usize] = wrap_sync!(data::data_size::<_, _>);
    table[DATACOPY as usize] = wrap_sync!(data::data_copy::<_, _>);

    table[RJUMP as usize] = wrap_sync!(control::rjump::<_, _>);
    table[RJUMPI as usize] = wrap_sync!(control::rjumpi::<_, _>);
    table[RJUMPV as usize] = wrap_sync!(control::rjumpv::<_, _>);
    table[CALLF as usize] = wrap_sync!(control::callf::<_, _>);
    table[RETF as usize] = wrap_sync!(control::retf::<_, _>);
    table[JUMPF as usize] = wrap_sync!(control::jumpf::<_, _>);
    table[DUPN as usize] = wrap_sync!(stack::dupn::<_, _>);
    table[SWAPN as usize] = wrap_sync!(stack::swapn::<_, _>);
    table[EXCHANGE as usize] = wrap_sync!(stack::exchange::<_, _>);

    table[EOFCREATE as usize] = wrap_sync!(contract::eofcreate::<_, _>);
    table[TXCREATE as usize] = wrap_sync!(contract::txcreate::<_, _>);
    table[RETURNCONTRACT as usize] = wrap_sync!(contract::return_contract);

    table[CREATE as usize] = wrap_sync!(contract::create::<_, false, _>);
    table[CALL as usize] = wrap_async!(contract::call::<_, _>);
    table[CALLCODE as usize] = wrap_async!(contract::call_code::<_, _>);
    table[RETURN as usize] = wrap_sync!(control::ret::<_, _>);
    table[DELEGATECALL as usize] = wrap_async!(contract::delegate_call::<_, _>);
    table[CREATE2 as usize] = wrap_sync!(contract::create::<_, true, _>);

    table[RETURNDATALOAD as usize] = wrap_sync!(system::returndataload::<_, _>);
    table[EXTCALL as usize] = wrap_async!(contract::extcall::<_, _>);
    table[EXTDELEGATECALL as usize] = wrap_async!(contract::extdelegatecall::<_, _>);
    table[STATICCALL as usize] = wrap_async!(contract::static_call::<_, _>);
    table[EXTSTATICCALL as usize] = wrap_async!(contract::extstaticcall::<_, _>);
    table[REVERT as usize] = wrap_sync!(control::revert::<_, _>);
    table[INVALID as usize] = wrap_sync!(control::invalid::<_, _>);
    table[SELFDESTRUCT as usize] = wrap_async!(host::selfdestruct::<_, _>);

    table
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
                "Opcode 0x{:X?} is not handled",
                i
            );
        }
    }
}
