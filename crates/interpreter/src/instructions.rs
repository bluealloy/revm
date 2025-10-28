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

use primitives::hardfork::SpecId;

use crate::{gas, interpreter_types::InterpreterTypes, Host, InstructionContext};

/// EVM opcode function signature.
#[derive(Debug)]
pub struct Instruction<W: InterpreterTypes, H: ?Sized> {
    fn_: fn(InstructionContext<'_, H, W>),
    static_gas: u64,
}

impl<W: InterpreterTypes, H: Host + ?Sized> Instruction<W, H> {
    /// Creates a new instruction with the given function and static gas cost.
    #[inline]
    pub const fn new(fn_: fn(InstructionContext<'_, H, W>), static_gas: u64) -> Self {
        Self { fn_, static_gas }
    }

    /// Creates an unknown/invalid instruction.
    #[inline]
    pub const fn unknown() -> Self {
        Self {
            fn_: control::unknown,
            static_gas: 0,
        }
    }

    /// Executes the instruction with the given context.
    #[inline(always)]
    pub fn execute(self, ctx: InstructionContext<'_, H, W>) {
        (self.fn_)(ctx)
    }

    /// Returns the static gas cost of this instruction.
    #[inline(always)]
    pub const fn static_gas(&self) -> u64 {
        self.static_gas
    }
}

impl<W: InterpreterTypes, H: Host + ?Sized> Copy for Instruction<W, H> {}
impl<W: InterpreterTypes, H: Host + ?Sized> Clone for Instruction<W, H> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Instruction table is list of instruction function pointers mapped to 256 EVM opcodes.
pub type InstructionTable<W, H> = [Instruction<W, H>; 256];

/// Returns the default instruction table for the given interpreter types and host.
#[inline]
pub const fn instruction_table<WIRE: InterpreterTypes, H: Host>() -> [Instruction<WIRE, H>; 256] {
    const { instruction_table_impl::<WIRE, H>() }
}

/// Create a instruction table with applied spec changes to static gas cost.
#[inline]
pub fn instruction_table_gas_changes_spec<WIRE: InterpreterTypes, H: Host>(
    spec: SpecId,
) -> [Instruction<WIRE, H>; 256] {
    use bytecode::opcode::*;
    use SpecId::*;
    let mut table = instruction_table();

    table[EXTCODESIZE as usize].static_gas = 20;
    table[EXTCODEHASH as usize].static_gas = 400;
    table[EXTCODECOPY as usize].static_gas = 20;
    table[SLOAD as usize].static_gas = 50;
    table[BALANCE as usize].static_gas = 20;
    table[CALL as usize].static_gas = 20;
    table[CALLCODE as usize].static_gas = 20;
    table[DELEGATECALL as usize].static_gas = 20;
    table[STATICCALL as usize].static_gas = 20;
    // SSTORE static gas can be found in GasParams as check for minimal stipend
    // needs to be done before deduction of static gas.

    if spec.is_enabled_in(TANGERINE) {
        // EIP-150: Gas cost changes for IO-heavy operations
        table[SLOAD as usize].static_gas = 200;
        table[BALANCE as usize].static_gas = 400;
        table[EXTCODESIZE as usize].static_gas = 700;
        table[EXTCODECOPY as usize].static_gas = 700;
        table[CALL as usize].static_gas = 700;
        table[CALLCODE as usize].static_gas = 700;
        table[DELEGATECALL as usize].static_gas = 700;
        table[STATICCALL as usize].static_gas = 700;
        // EIP-150: Gas cost changes for IO-heavy operations
        //
        table[SELFDESTRUCT as usize].static_gas = 5000;
    }

    if spec.is_enabled_in(ISTANBUL) {
        // EIP-1884: Repricing for trie-size-dependent opcodes
        table[SLOAD as usize].static_gas = gas::ISTANBUL_SLOAD_GAS;
        table[BALANCE as usize].static_gas = 400;
        table[EXTCODEHASH as usize].static_gas = 700;
    }

    if spec.is_enabled_in(BERLIN) {
        // warm account cost is base gas that is spend. Additional gas depends if account is cold loaded.
        table[SLOAD as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[BALANCE as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[EXTCODESIZE as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[EXTCODEHASH as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[EXTCODECOPY as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[CALL as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[CALLCODE as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[DELEGATECALL as usize].static_gas = gas::WARM_STORAGE_READ_COST;
        table[STATICCALL as usize].static_gas = gas::WARM_STORAGE_READ_COST;
    }

    println!(
        "INSTRUCTION CALL TABLE: {:?}",
        table[CALL as usize].static_gas
    );

    table
}

const fn instruction_table_impl<WIRE: InterpreterTypes, H: Host>() -> [Instruction<WIRE, H>; 256] {
    use bytecode::opcode::*;
    let mut table = [Instruction::unknown(); 256];

    table[STOP as usize] = Instruction::new(control::stop, 0);
    table[ADD as usize] = Instruction::new(arithmetic::add, 3);
    table[MUL as usize] = Instruction::new(arithmetic::mul, 5);
    table[SUB as usize] = Instruction::new(arithmetic::sub, 3);
    table[DIV as usize] = Instruction::new(arithmetic::div, 5);
    table[SDIV as usize] = Instruction::new(arithmetic::sdiv, 5);
    table[MOD as usize] = Instruction::new(arithmetic::rem, 5);
    table[SMOD as usize] = Instruction::new(arithmetic::smod, 5);
    table[ADDMOD as usize] = Instruction::new(arithmetic::addmod, 8);
    table[MULMOD as usize] = Instruction::new(arithmetic::mulmod, 8);
    table[EXP as usize] = Instruction::new(arithmetic::exp, gas::EXP); // base
    table[SIGNEXTEND as usize] = Instruction::new(arithmetic::signextend, 5);

    table[LT as usize] = Instruction::new(bitwise::lt, 3);
    table[GT as usize] = Instruction::new(bitwise::gt, 3);
    table[SLT as usize] = Instruction::new(bitwise::slt, 3);
    table[SGT as usize] = Instruction::new(bitwise::sgt, 3);
    table[EQ as usize] = Instruction::new(bitwise::eq, 3);
    table[ISZERO as usize] = Instruction::new(bitwise::iszero, 3);
    table[AND as usize] = Instruction::new(bitwise::bitand, 3);
    table[OR as usize] = Instruction::new(bitwise::bitor, 3);
    table[XOR as usize] = Instruction::new(bitwise::bitxor, 3);
    table[NOT as usize] = Instruction::new(bitwise::not, 3);
    table[BYTE as usize] = Instruction::new(bitwise::byte, 3);
    table[SHL as usize] = Instruction::new(bitwise::shl, 3);
    table[SHR as usize] = Instruction::new(bitwise::shr, 3);
    table[SAR as usize] = Instruction::new(bitwise::sar, 3);
    table[CLZ as usize] = Instruction::new(bitwise::clz, 5);

    table[KECCAK256 as usize] = Instruction::new(system::keccak256, 30); // dynamic

    table[ADDRESS as usize] = Instruction::new(system::address, 2);
    table[BALANCE as usize] = Instruction::new(host::balance, 0); // dynamic
    table[ORIGIN as usize] = Instruction::new(tx_info::origin, 2);
    table[CALLER as usize] = Instruction::new(system::caller, 2);
    table[CALLVALUE as usize] = Instruction::new(system::callvalue, 2);
    table[CALLDATALOAD as usize] = Instruction::new(system::calldataload, 3);
    table[CALLDATASIZE as usize] = Instruction::new(system::calldatasize, 2);
    table[CALLDATACOPY as usize] = Instruction::new(system::calldatacopy, 3);
    table[CODESIZE as usize] = Instruction::new(system::codesize, 2);
    table[CODECOPY as usize] = Instruction::new(system::codecopy, 3);

    table[GASPRICE as usize] = Instruction::new(tx_info::gasprice, 2);
    table[EXTCODESIZE as usize] = Instruction::new(host::extcodesize, 0); // dynamic
    table[EXTCODECOPY as usize] = Instruction::new(host::extcodecopy, 0); // dynamic
    table[RETURNDATASIZE as usize] = Instruction::new(system::returndatasize, 2);
    table[RETURNDATACOPY as usize] = Instruction::new(system::returndatacopy, 3);
    table[EXTCODEHASH as usize] = Instruction::new(host::extcodehash, 0); // dynamic
    table[BLOCKHASH as usize] = Instruction::new(host::blockhash, 20);
    table[COINBASE as usize] = Instruction::new(block_info::coinbase, 2);
    table[TIMESTAMP as usize] = Instruction::new(block_info::timestamp, 2);
    table[NUMBER as usize] = Instruction::new(block_info::block_number, 2);
    table[DIFFICULTY as usize] = Instruction::new(block_info::difficulty, 2);
    table[GASLIMIT as usize] = Instruction::new(block_info::gaslimit, 2);
    table[CHAINID as usize] = Instruction::new(block_info::chainid, 2);
    table[SELFBALANCE as usize] = Instruction::new(host::selfbalance, 5);
    table[BASEFEE as usize] = Instruction::new(block_info::basefee, 2);
    table[BLOBHASH as usize] = Instruction::new(tx_info::blob_hash, 3);
    table[BLOBBASEFEE as usize] = Instruction::new(block_info::blob_basefee, 2);

    table[POP as usize] = Instruction::new(stack::pop, 2);
    table[MLOAD as usize] = Instruction::new(memory::mload, 3);
    table[MSTORE as usize] = Instruction::new(memory::mstore, 3);
    table[MSTORE8 as usize] = Instruction::new(memory::mstore8, 3);
    table[SLOAD as usize] = Instruction::new(host::sload, 0); // dynamic
    table[SSTORE as usize] = Instruction::new(host::sstore, 0); // dynamic
    table[JUMP as usize] = Instruction::new(control::jump, 8);
    table[JUMPI as usize] = Instruction::new(control::jumpi, 10);
    table[PC as usize] = Instruction::new(control::pc, 2);
    table[MSIZE as usize] = Instruction::new(memory::msize, 2);
    table[GAS as usize] = Instruction::new(system::gas, 2);
    table[JUMPDEST as usize] = Instruction::new(control::jumpdest, 1);
    table[TLOAD as usize] = Instruction::new(host::tload, 100);
    table[TSTORE as usize] = Instruction::new(host::tstore, 100);
    table[MCOPY as usize] = Instruction::new(memory::mcopy, 3); // static 2, mostly dynamic

    table[PUSH0 as usize] = Instruction::new(stack::push0, 2);
    table[PUSH1 as usize] = Instruction::new(stack::push::<1, _, _>, 3);
    table[PUSH2 as usize] = Instruction::new(stack::push::<2, _, _>, 3);
    table[PUSH3 as usize] = Instruction::new(stack::push::<3, _, _>, 3);
    table[PUSH4 as usize] = Instruction::new(stack::push::<4, _, _>, 3);
    table[PUSH5 as usize] = Instruction::new(stack::push::<5, _, _>, 3);
    table[PUSH6 as usize] = Instruction::new(stack::push::<6, _, _>, 3);
    table[PUSH7 as usize] = Instruction::new(stack::push::<7, _, _>, 3);
    table[PUSH8 as usize] = Instruction::new(stack::push::<8, _, _>, 3);
    table[PUSH9 as usize] = Instruction::new(stack::push::<9, _, _>, 3);
    table[PUSH10 as usize] = Instruction::new(stack::push::<10, _, _>, 3);
    table[PUSH11 as usize] = Instruction::new(stack::push::<11, _, _>, 3);
    table[PUSH12 as usize] = Instruction::new(stack::push::<12, _, _>, 3);
    table[PUSH13 as usize] = Instruction::new(stack::push::<13, _, _>, 3);
    table[PUSH14 as usize] = Instruction::new(stack::push::<14, _, _>, 3);
    table[PUSH15 as usize] = Instruction::new(stack::push::<15, _, _>, 3);
    table[PUSH16 as usize] = Instruction::new(stack::push::<16, _, _>, 3);
    table[PUSH17 as usize] = Instruction::new(stack::push::<17, _, _>, 3);
    table[PUSH18 as usize] = Instruction::new(stack::push::<18, _, _>, 3);
    table[PUSH19 as usize] = Instruction::new(stack::push::<19, _, _>, 3);
    table[PUSH20 as usize] = Instruction::new(stack::push::<20, _, _>, 3);
    table[PUSH21 as usize] = Instruction::new(stack::push::<21, _, _>, 3);
    table[PUSH22 as usize] = Instruction::new(stack::push::<22, _, _>, 3);
    table[PUSH23 as usize] = Instruction::new(stack::push::<23, _, _>, 3);
    table[PUSH24 as usize] = Instruction::new(stack::push::<24, _, _>, 3);
    table[PUSH25 as usize] = Instruction::new(stack::push::<25, _, _>, 3);
    table[PUSH26 as usize] = Instruction::new(stack::push::<26, _, _>, 3);
    table[PUSH27 as usize] = Instruction::new(stack::push::<27, _, _>, 3);
    table[PUSH28 as usize] = Instruction::new(stack::push::<28, _, _>, 3);
    table[PUSH29 as usize] = Instruction::new(stack::push::<29, _, _>, 3);
    table[PUSH30 as usize] = Instruction::new(stack::push::<30, _, _>, 3);
    table[PUSH31 as usize] = Instruction::new(stack::push::<31, _, _>, 3);
    table[PUSH32 as usize] = Instruction::new(stack::push::<32, _, _>, 3);

    table[DUP1 as usize] = Instruction::new(stack::dup::<1, _, _>, 3);
    table[DUP2 as usize] = Instruction::new(stack::dup::<2, _, _>, 3);
    table[DUP3 as usize] = Instruction::new(stack::dup::<3, _, _>, 3);
    table[DUP4 as usize] = Instruction::new(stack::dup::<4, _, _>, 3);
    table[DUP5 as usize] = Instruction::new(stack::dup::<5, _, _>, 3);
    table[DUP6 as usize] = Instruction::new(stack::dup::<6, _, _>, 3);
    table[DUP7 as usize] = Instruction::new(stack::dup::<7, _, _>, 3);
    table[DUP8 as usize] = Instruction::new(stack::dup::<8, _, _>, 3);
    table[DUP9 as usize] = Instruction::new(stack::dup::<9, _, _>, 3);
    table[DUP10 as usize] = Instruction::new(stack::dup::<10, _, _>, 3);
    table[DUP11 as usize] = Instruction::new(stack::dup::<11, _, _>, 3);
    table[DUP12 as usize] = Instruction::new(stack::dup::<12, _, _>, 3);
    table[DUP13 as usize] = Instruction::new(stack::dup::<13, _, _>, 3);
    table[DUP14 as usize] = Instruction::new(stack::dup::<14, _, _>, 3);
    table[DUP15 as usize] = Instruction::new(stack::dup::<15, _, _>, 3);
    table[DUP16 as usize] = Instruction::new(stack::dup::<16, _, _>, 3);

    table[SWAP1 as usize] = Instruction::new(stack::swap::<1, _, _>, 3);
    table[SWAP2 as usize] = Instruction::new(stack::swap::<2, _, _>, 3);
    table[SWAP3 as usize] = Instruction::new(stack::swap::<3, _, _>, 3);
    table[SWAP4 as usize] = Instruction::new(stack::swap::<4, _, _>, 3);
    table[SWAP5 as usize] = Instruction::new(stack::swap::<5, _, _>, 3);
    table[SWAP6 as usize] = Instruction::new(stack::swap::<6, _, _>, 3);
    table[SWAP7 as usize] = Instruction::new(stack::swap::<7, _, _>, 3);
    table[SWAP8 as usize] = Instruction::new(stack::swap::<8, _, _>, 3);
    table[SWAP9 as usize] = Instruction::new(stack::swap::<9, _, _>, 3);
    table[SWAP10 as usize] = Instruction::new(stack::swap::<10, _, _>, 3);
    table[SWAP11 as usize] = Instruction::new(stack::swap::<11, _, _>, 3);
    table[SWAP12 as usize] = Instruction::new(stack::swap::<12, _, _>, 3);
    table[SWAP13 as usize] = Instruction::new(stack::swap::<13, _, _>, 3);
    table[SWAP14 as usize] = Instruction::new(stack::swap::<14, _, _>, 3);
    table[SWAP15 as usize] = Instruction::new(stack::swap::<15, _, _>, 3);
    table[SWAP16 as usize] = Instruction::new(stack::swap::<16, _, _>, 3);

    table[LOG0 as usize] = Instruction::new(host::log::<0, _>, gas::LOG); // dynamic
    table[LOG1 as usize] = Instruction::new(host::log::<1, _>, gas::LOG); // dynamic
    table[LOG2 as usize] = Instruction::new(host::log::<2, _>, gas::LOG); // dynamic
    table[LOG3 as usize] = Instruction::new(host::log::<3, _>, gas::LOG); // dynamic
    table[LOG4 as usize] = Instruction::new(host::log::<4, _>, gas::LOG); // dynamic

    table[CREATE as usize] = Instruction::new(contract::create::<_, false, _>, 0); // dynamic
    table[CALL as usize] = Instruction::new(contract::call, 0); // dynamic
    table[CALLCODE as usize] = Instruction::new(contract::call_code, 0); // dynamic
    table[RETURN as usize] = Instruction::new(control::ret, 0);
    table[DELEGATECALL as usize] = Instruction::new(contract::delegate_call, 0); // dynamic
    table[CREATE2 as usize] = Instruction::new(contract::create::<_, true, _>, 0); // dynamic

    table[STATICCALL as usize] = Instruction::new(contract::static_call, 0); // dynamic
    table[REVERT as usize] = Instruction::new(control::revert, 0);
    table[INVALID as usize] = Instruction::new(control::invalid, 0);
    table[SELFDESTRUCT as usize] = Instruction::new(host::selfdestruct, 0); // dynamic
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
            let is_instr_unknown = std::ptr::fn_addr_eq(instr.fn_, unknown_istr.fn_);
            assert_eq!(
                is_instr_unknown, is_opcode_unknown,
                "Opcode 0x{i:X?} is not handled",
            );
        }
    }
}
