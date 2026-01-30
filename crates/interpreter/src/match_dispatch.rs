//! Match-based dispatch for the EVM interpreter.
//!
//! This module provides an alternative dispatch mechanism that uses a giant `match`
//! statement instead of indirect function pointer lookups. This is inspired by
//! evmone's computed goto dispatch and can improve branch prediction on some CPUs.
//!
//! Enable with the `match-dispatch` feature flag.

use crate::{
    instruction_context::InstructionContext,
    instructions::{
        arithmetic, bitwise, block_info, contract, control, gas, host, memory, stack, system,
        tx_info,
    },
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, Jumps, LoopControl},
    Host, InterpreterAction,
};
use bytecode::opcode::*;

const fn static_gas(opcode: u8) -> u64 {
    match opcode {
        STOP => 0,
        ADD => 3,
        MUL => 5,
        SUB => 3,
        DIV => 5,
        SDIV => 5,
        MOD => 5,
        SMOD => 5,
        ADDMOD => 8,
        MULMOD => 8,
        EXP => gas::EXP,
        SIGNEXTEND => 5,

        LT => 3,
        GT => 3,
        SLT => 3,
        SGT => 3,
        EQ => 3,
        ISZERO => 3,
        AND => 3,
        OR => 3,
        XOR => 3,
        NOT => 3,
        BYTE => 3,
        SHL => 3,
        SHR => 3,
        SAR => 3,
        CLZ => 5,

        KECCAK256 => gas::KECCAK256,

        ADDRESS => 2,
        BALANCE => 20,
        ORIGIN => 2,
        CALLER => 2,
        CALLVALUE => 2,
        CALLDATALOAD => 3,
        CALLDATASIZE => 2,
        CALLDATACOPY => 3,
        CODESIZE => 2,
        CODECOPY => 3,

        GASPRICE => 2,
        EXTCODESIZE => 20,
        EXTCODECOPY => 20,
        RETURNDATASIZE => 2,
        RETURNDATACOPY => 3,
        EXTCODEHASH => 400,
        BLOCKHASH => 20,
        COINBASE => 2,
        TIMESTAMP => 2,
        NUMBER => 2,
        DIFFICULTY => 2,
        GASLIMIT => 2,
        CHAINID => 2,
        SELFBALANCE => 5,
        BASEFEE => 2,
        BLOBHASH => 3,
        BLOBBASEFEE => 2,
        SLOTNUM => 2,

        POP => 2,
        MLOAD => 3,
        MSTORE => 3,
        MSTORE8 => 3,
        SLOAD => 50,
        SSTORE => 0,
        JUMP => 8,
        JUMPI => 10,
        PC => 2,
        MSIZE => 2,
        GAS => 2,
        JUMPDEST => 1,
        TLOAD => 100,
        TSTORE => 100,
        MCOPY => 3,

        PUSH0 => 2,
        PUSH1..=PUSH32 => 3,

        DUP1..=DUP16 => 3,
        SWAP1..=SWAP16 => 3,

        DUPN => 3,
        SWAPN => 3,
        EXCHANGE => 3,

        LOG0..=LOG4 => gas::LOG,

        CREATE => 0,
        CALL => 40,
        CALLCODE => 40,
        RETURN => 0,
        DELEGATECALL => 40,
        CREATE2 => 0,
        STATICCALL => 40,
        REVERT => 0,
        INVALID => 0,
        SELFDESTRUCT => 0,

        _ => 0,
    }
}

macro_rules! step_match {
    ($interpreter:expr, $host:expr, $opcode:expr) => {
        match $opcode {
            STOP => control::stop(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            ADD => arithmetic::add(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MUL => arithmetic::mul(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SUB => arithmetic::sub(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DIV => arithmetic::div(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SDIV => arithmetic::sdiv(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MOD => arithmetic::rem(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SMOD => arithmetic::smod(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            ADDMOD => arithmetic::addmod(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MULMOD => arithmetic::mulmod(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EXP => arithmetic::exp(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SIGNEXTEND => arithmetic::signextend(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            LT => bitwise::lt(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            GT => bitwise::gt(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SLT => bitwise::slt(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SGT => bitwise::sgt(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EQ => bitwise::eq(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            ISZERO => bitwise::iszero(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            AND => bitwise::bitand(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            OR => bitwise::bitor(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            XOR => bitwise::bitxor(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            NOT => bitwise::not(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BYTE => bitwise::byte(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SHL => bitwise::shl(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SHR => bitwise::shr(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SAR => bitwise::sar(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CLZ => bitwise::clz(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            KECCAK256 => system::keccak256(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            ADDRESS => system::address(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BALANCE => host::balance(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            ORIGIN => tx_info::origin(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CALLER => system::caller(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CALLVALUE => system::callvalue(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CALLDATALOAD => system::calldataload(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CALLDATASIZE => system::calldatasize(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CALLDATACOPY => system::calldatacopy(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CODESIZE => system::codesize(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CODECOPY => system::codecopy(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            GASPRICE => tx_info::gasprice(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EXTCODESIZE => host::extcodesize(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EXTCODECOPY => host::extcodecopy(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            RETURNDATASIZE => system::returndatasize(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            RETURNDATACOPY => system::returndatacopy(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EXTCODEHASH => host::extcodehash(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BLOCKHASH => host::blockhash(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            COINBASE => block_info::coinbase(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            TIMESTAMP => block_info::timestamp(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            NUMBER => block_info::block_number(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DIFFICULTY => block_info::difficulty(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            GASLIMIT => block_info::gaslimit(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            CHAINID => block_info::chainid(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SELFBALANCE => host::selfbalance(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BASEFEE => block_info::basefee(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BLOBHASH => tx_info::blob_hash(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            BLOBBASEFEE => block_info::blob_basefee(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SLOTNUM => block_info::slot_num(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            POP => stack::pop(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MLOAD => memory::mload(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MSTORE => memory::mstore(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MSTORE8 => memory::mstore8(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SLOAD => host::sload(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SSTORE => host::sstore(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            JUMP => control::jump(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            JUMPI => control::jumpi(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PC => control::pc(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MSIZE => memory::msize(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            GAS => system::gas(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            JUMPDEST => control::jumpdest(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            TLOAD => host::tload(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            TSTORE => host::tstore(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            MCOPY => memory::mcopy(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            PUSH0 => stack::push0(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH1 => stack::push::<1, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH2 => stack::push::<2, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH3 => stack::push::<3, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH4 => stack::push::<4, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH5 => stack::push::<5, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH6 => stack::push::<6, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH7 => stack::push::<7, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH8 => stack::push::<8, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH9 => stack::push::<9, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH10 => stack::push::<10, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH11 => stack::push::<11, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH12 => stack::push::<12, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH13 => stack::push::<13, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH14 => stack::push::<14, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH15 => stack::push::<15, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH16 => stack::push::<16, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH17 => stack::push::<17, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH18 => stack::push::<18, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH19 => stack::push::<19, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH20 => stack::push::<20, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH21 => stack::push::<21, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH22 => stack::push::<22, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH23 => stack::push::<23, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH24 => stack::push::<24, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH25 => stack::push::<25, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH26 => stack::push::<26, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH27 => stack::push::<27, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH28 => stack::push::<28, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH29 => stack::push::<29, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH30 => stack::push::<30, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH31 => stack::push::<31, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            PUSH32 => stack::push::<32, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            DUP1 => stack::dup::<1, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP2 => stack::dup::<2, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP3 => stack::dup::<3, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP4 => stack::dup::<4, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP5 => stack::dup::<5, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP6 => stack::dup::<6, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP7 => stack::dup::<7, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP8 => stack::dup::<8, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP9 => stack::dup::<9, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP10 => stack::dup::<10, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP11 => stack::dup::<11, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP12 => stack::dup::<12, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP13 => stack::dup::<13, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP14 => stack::dup::<14, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP15 => stack::dup::<15, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DUP16 => stack::dup::<16, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            SWAP1 => stack::swap::<1, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP2 => stack::swap::<2, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP3 => stack::swap::<3, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP4 => stack::swap::<4, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP5 => stack::swap::<5, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP6 => stack::swap::<6, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP7 => stack::swap::<7, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP8 => stack::swap::<8, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP9 => stack::swap::<9, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP10 => stack::swap::<10, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP11 => stack::swap::<11, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP12 => stack::swap::<12, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP13 => stack::swap::<13, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP14 => stack::swap::<14, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP15 => stack::swap::<15, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAP16 => stack::swap::<16, _, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            DUPN => stack::dupn(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SWAPN => stack::swapn(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            EXCHANGE => stack::exchange(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            LOG0 => host::log::<0, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            LOG1 => host::log::<1, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            LOG2 => host::log::<2, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            LOG3 => host::log::<3, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            LOG4 => host::log::<4, _>(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),

            CREATE => {
                #[cold]
                #[inline(never)]
                fn cold_create<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::create::<_, false, _>(InstructionContext { interpreter, host })
                }
                cold_create($interpreter, $host)
            }
            CALL => {
                #[cold]
                #[inline(never)]
                fn cold_call<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::call(InstructionContext { interpreter, host })
                }
                cold_call($interpreter, $host)
            }
            CALLCODE => {
                #[cold]
                #[inline(never)]
                fn cold_callcode<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::call_code(InstructionContext { interpreter, host })
                }
                cold_callcode($interpreter, $host)
            }
            RETURN => control::ret(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            DELEGATECALL => {
                #[cold]
                #[inline(never)]
                fn cold_delegatecall<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::delegate_call(InstructionContext { interpreter, host })
                }
                cold_delegatecall($interpreter, $host)
            }
            CREATE2 => {
                #[cold]
                #[inline(never)]
                fn cold_create2<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::create::<_, true, _>(InstructionContext { interpreter, host })
                }
                cold_create2($interpreter, $host)
            }
            STATICCALL => {
                #[cold]
                #[inline(never)]
                fn cold_staticcall<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    contract::static_call(InstructionContext { interpreter, host })
                }
                cold_staticcall($interpreter, $host)
            }
            REVERT => control::revert(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            INVALID => control::invalid(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
            SELFDESTRUCT => {
                #[cold]
                #[inline(never)]
                fn cold_selfdestruct<IW: InterpreterTypes, H: Host + ?Sized>(
                    interpreter: &mut Interpreter<IW>,
                    host: &mut H,
                ) {
                    host::selfdestruct(InstructionContext { interpreter, host })
                }
                cold_selfdestruct($interpreter, $host)
            }

            _ => control::unknown(InstructionContext {
                interpreter: $interpreter,
                host: $host,
            }),
        }
    };
}

impl<IW: InterpreterTypes> Interpreter<IW> {
    /// Executes a single instruction using match-based dispatch.
    ///
    /// This uses a giant `match` statement instead of function pointer lookups,
    /// which can improve branch prediction on some CPUs.
    #[inline]
    pub fn step_match<H: Host + ?Sized>(&mut self, host: &mut H) {
        let opcode = self.bytecode.opcode();
        self.bytecode.relative_jump(1);

        let gas_cost = static_gas(opcode);
        if gas_cost != 0 && self.gas.record_cost_unsafe(gas_cost) {
            return self.halt_oog();
        }

        step_match!(self, host, opcode);
    }

    /// Executes the interpreter until it returns or stops, using match-based dispatch.
    ///
    /// This is the match-dispatch equivalent of [`run_plain`](Interpreter::run_plain),
    /// potentially offering better performance on some CPUs.
    #[inline]
    pub fn run_match<H: Host + ?Sized>(&mut self, host: &mut H) -> InterpreterAction {
        while self.bytecode.is_not_end() {
            self.step_match(host);
        }
        self.take_next_action()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        host::DummyHost,
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
    };
    use bytecode::Bytecode;
    use primitives::{hardfork::SpecId, Bytes};

    #[test]
    fn test_match_dispatch_simple() {
        let code = Bytes::from(
            &[
                0x60, 0x01, // PUSH1 1
                0x60, 0x02, // PUSH1 2
                0x01, // ADD
                0x00, // STOP
            ][..],
        );
        let bytecode = Bytecode::new_raw(code);

        let mut interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::default(),
            100000,
        );

        let mut host = DummyHost::default();
        let action = interpreter.run_match(&mut host);

        assert!(action.is_return());
    }
}
