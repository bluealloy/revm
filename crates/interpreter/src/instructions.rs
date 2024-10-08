//! EVM opcode implementations.

#[macro_use]
pub mod macros;
pub mod arithmetic;
pub mod bitwise;
pub mod contract;
pub mod control;
pub mod data;
pub mod host;
pub mod host_env;
pub mod i256;
pub mod memory;
pub mod stack;
pub mod system;
pub mod utility;

use crate::Host;
use specification::hardfork::Spec;

/// Returns the instruction function for the given opcode and spec.
pub const fn instruction<H: Host + ?Sized, SPEC: Spec>(opcode: u8) -> crate::table::Instruction<H> {
    let table = instruction_table::<H, SPEC>();
    table[opcode as usize]
}

pub const fn instruction_table<H: Host + ?Sized, SPEC: Spec>() -> [crate::table::Instruction<H>; 256]
{
    use bytecode::opcode::*;
    let mut table = [control::unknown as crate::table::Instruction<H>; 256];

    table[STOP as usize] = control::stop;
    table[ADD as usize] = arithmetic::add;
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
    table[EXP as usize] = arithmetic::exp::<H, SPEC>;
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
    table[SHL as usize] = bitwise::shl::<H, SPEC>;
    table[SHR as usize] = bitwise::shr::<H, SPEC>;
    table[SAR as usize] = bitwise::sar::<H, SPEC>;

    table[KECCAK256 as usize] = system::keccak256;

    table[ADDRESS as usize] = system::address;
    table[BALANCE as usize] = host::balance::<H, SPEC>;
    table[ORIGIN as usize] = host_env::origin;
    table[CALLER as usize] = system::caller;
    table[CALLVALUE as usize] = system::callvalue;
    table[CALLDATALOAD as usize] = system::calldataload;
    table[CALLDATASIZE as usize] = system::calldatasize;
    table[CALLDATACOPY as usize] = system::calldatacopy;
    table[CODESIZE as usize] = system::codesize;
    table[CODECOPY as usize] = system::codecopy;

    table[GASPRICE as usize] = host_env::gasprice;
    table[EXTCODESIZE as usize] = host::extcodesize::<H, SPEC>;
    table[EXTCODECOPY as usize] = host::extcodecopy::<H, SPEC>;
    table[RETURNDATASIZE as usize] = system::returndatasize::<H, SPEC>;
    table[RETURNDATACOPY as usize] = system::returndatacopy::<H, SPEC>;
    table[EXTCODEHASH as usize] = host::extcodehash::<H, SPEC>;
    table[BLOCKHASH as usize] = host::blockhash::<H, SPEC>;
    table[COINBASE as usize] = host_env::coinbase;
    table[TIMESTAMP as usize] = host_env::timestamp;
    table[NUMBER as usize] = host_env::block_number;
    table[DIFFICULTY as usize] = host_env::difficulty::<H, SPEC>;
    table[GASLIMIT as usize] = host_env::gaslimit;
    table[CHAINID as usize] = host_env::chainid::<H, SPEC>;
    table[SELFBALANCE as usize] = host::selfbalance::<H, SPEC>;
    table[BASEFEE as usize] = host_env::basefee::<H, SPEC>;
    table[BLOBHASH as usize] = host_env::blob_hash::<H, SPEC>;
    table[BLOBBASEFEE as usize] = host_env::blob_basefee::<H, SPEC>;

    table[POP as usize] = stack::pop;
    table[MLOAD as usize] = memory::mload;
    table[MSTORE as usize] = memory::mstore;
    table[MSTORE8 as usize] = memory::mstore8;
    table[SLOAD as usize] = host::sload::<H, SPEC>;
    table[SSTORE as usize] = host::sstore::<H, SPEC>;
    table[JUMP as usize] = control::jump;
    table[JUMPI as usize] = control::jumpi;
    table[PC as usize] = control::pc;
    table[MSIZE as usize] = memory::msize;
    table[GAS as usize] = system::gas;
    table[JUMPDEST as usize] = control::jumpdest_or_nop;
    table[TLOAD as usize] = host::tload::<H, SPEC>;
    table[TSTORE as usize] = host::tstore::<H, SPEC>;
    table[MCOPY as usize] = memory::mcopy::<H, SPEC>;

    table[PUSH0 as usize] = stack::push0::<H, SPEC>;
    table[PUSH1 as usize] = stack::push::<1, H>;
    table[PUSH2 as usize] = stack::push::<2, H>;
    table[PUSH3 as usize] = stack::push::<3, H>;
    table[PUSH4 as usize] = stack::push::<4, H>;
    table[PUSH5 as usize] = stack::push::<5, H>;
    table[PUSH6 as usize] = stack::push::<6, H>;
    table[PUSH7 as usize] = stack::push::<7, H>;
    table[PUSH8 as usize] = stack::push::<8, H>;
    table[PUSH9 as usize] = stack::push::<9, H>;
    table[PUSH10 as usize] = stack::push::<10, H>;
    table[PUSH11 as usize] = stack::push::<11, H>;
    table[PUSH12 as usize] = stack::push::<12, H>;
    table[PUSH13 as usize] = stack::push::<13, H>;
    table[PUSH14 as usize] = stack::push::<14, H>;
    table[PUSH15 as usize] = stack::push::<15, H>;
    table[PUSH16 as usize] = stack::push::<16, H>;
    table[PUSH17 as usize] = stack::push::<17, H>;
    table[PUSH18 as usize] = stack::push::<18, H>;
    table[PUSH19 as usize] = stack::push::<19, H>;
    table[PUSH20 as usize] = stack::push::<20, H>;
    table[PUSH21 as usize] = stack::push::<21, H>;
    table[PUSH22 as usize] = stack::push::<22, H>;
    table[PUSH23 as usize] = stack::push::<23, H>;
    table[PUSH24 as usize] = stack::push::<24, H>;
    table[PUSH25 as usize] = stack::push::<25, H>;
    table[PUSH26 as usize] = stack::push::<26, H>;
    table[PUSH27 as usize] = stack::push::<27, H>;
    table[PUSH28 as usize] = stack::push::<28, H>;
    table[PUSH29 as usize] = stack::push::<29, H>;
    table[PUSH30 as usize] = stack::push::<30, H>;
    table[PUSH31 as usize] = stack::push::<31, H>;
    table[PUSH32 as usize] = stack::push::<32, H>;

    table[DUP1 as usize] = stack::dup::<1, H>;
    table[DUP2 as usize] = stack::dup::<2, H>;
    table[DUP3 as usize] = stack::dup::<3, H>;
    table[DUP4 as usize] = stack::dup::<4, H>;
    table[DUP5 as usize] = stack::dup::<5, H>;
    table[DUP6 as usize] = stack::dup::<6, H>;
    table[DUP7 as usize] = stack::dup::<7, H>;
    table[DUP8 as usize] = stack::dup::<8, H>;
    table[DUP9 as usize] = stack::dup::<9, H>;
    table[DUP10 as usize] = stack::dup::<10, H>;
    table[DUP11 as usize] = stack::dup::<11, H>;
    table[DUP12 as usize] = stack::dup::<12, H>;
    table[DUP13 as usize] = stack::dup::<13, H>;
    table[DUP14 as usize] = stack::dup::<14, H>;
    table[DUP15 as usize] = stack::dup::<15, H>;
    table[DUP16 as usize] = stack::dup::<16, H>;

    table[SWAP1 as usize] = stack::swap::<1, H>;
    table[SWAP2 as usize] = stack::swap::<2, H>;
    table[SWAP3 as usize] = stack::swap::<3, H>;
    table[SWAP4 as usize] = stack::swap::<4, H>;
    table[SWAP5 as usize] = stack::swap::<5, H>;
    table[SWAP6 as usize] = stack::swap::<6, H>;
    table[SWAP7 as usize] = stack::swap::<7, H>;
    table[SWAP8 as usize] = stack::swap::<8, H>;
    table[SWAP9 as usize] = stack::swap::<9, H>;
    table[SWAP10 as usize] = stack::swap::<10, H>;
    table[SWAP11 as usize] = stack::swap::<11, H>;
    table[SWAP12 as usize] = stack::swap::<12, H>;
    table[SWAP13 as usize] = stack::swap::<13, H>;
    table[SWAP14 as usize] = stack::swap::<14, H>;
    table[SWAP15 as usize] = stack::swap::<15, H>;
    table[SWAP16 as usize] = stack::swap::<16, H>;

    table[LOG0 as usize] = host::log::<0, H>;
    table[LOG1 as usize] = host::log::<1, H>;
    table[LOG2 as usize] = host::log::<2, H>;
    table[LOG3 as usize] = host::log::<3, H>;
    table[LOG4 as usize] = host::log::<4, H>;

    table[DATALOAD as usize] = data::data_load;
    table[DATALOADN as usize] = data::data_loadn;
    table[DATASIZE as usize] = data::data_size;
    table[DATACOPY as usize] = data::data_copy;

    table[RJUMP as usize] = control::rjump;
    table[RJUMPI as usize] = control::rjumpi;
    table[RJUMPV as usize] = control::rjumpv;
    table[CALLF as usize] = control::callf;
    table[RETF as usize] = control::retf;
    table[JUMPF as usize] = control::jumpf;
    table[DUPN as usize] = stack::dupn;
    table[SWAPN as usize] = stack::swapn;
    table[EXCHANGE as usize] = stack::exchange;

    table[EOFCREATE as usize] = contract::eofcreate;

    table[RETURNCONTRACT as usize] = contract::return_contract;

    table[CREATE as usize] = contract::create::<false, H, SPEC>;
    table[CALL as usize] = contract::call::<H, SPEC>;
    table[CALLCODE as usize] = contract::call_code::<H, SPEC>;
    table[RETURN as usize] = control::ret;
    table[DELEGATECALL as usize] = contract::delegate_call::<H, SPEC>;
    table[CREATE2 as usize] = contract::create::<true, H, SPEC>;

    table[RETURNDATALOAD as usize] = system::returndataload;
    table[EXTCALL as usize] = contract::extcall::<H, SPEC>;
    table[EXTDELEGATECALL as usize] = contract::extdelegatecall::<H, SPEC>;
    table[STATICCALL as usize] = contract::static_call::<H, SPEC>;
    table[EXTSTATICCALL as usize] = contract::extstaticcall;
    table[REVERT as usize] = control::revert::<H, SPEC>;
    table[INVALID as usize] = control::invalid;
    table[SELFDESTRUCT as usize] = host::selfdestruct::<H, SPEC>;
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DummyHost;
    use bytecode::opcode::*;
    use specification::hardfork::LatestSpec;
    use wiring::DefaultEthereumWiring;

    #[test]
    fn all_instructions_and_opcodes_used() {
        // known unknown instruction we compare it with other instructions from table.
        let unknown_instruction = 0x0C_usize;
        let instr_table = instruction_table::<DummyHost<DefaultEthereumWiring>, LatestSpec>();

        let unknown_istr = instr_table[unknown_instruction];
        for (i, instr) in instr_table.iter().enumerate() {
            let is_opcode_unknown = OpCode::new(i as u8).is_none();
            let is_instr_unknown = *instr == unknown_istr;
            assert_eq!(
                is_instr_unknown, is_opcode_unknown,
                "Opcode 0x{:X?} is not handled",
                i
            );
        }
    }
}
