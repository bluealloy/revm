
use auto_impl::auto_impl;
use core::mem::MaybeUninit;
use revm::{
    bytecode::opcode::OpCode,
    context::JournaledState,
    context_interface::{
        block::BlockSetter,
        journaled_state::{AccountLoad, Eip7702CodeLoad},
        transaction::TransactionSetter,
        BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalDBError,
        JournalGetter, TransactionGetter,
    },
    database_interface::{Database, EmptyDB},
    handler::{
        EthExecution, EthFrame, EthHandler, EthPostExecution, EthPreExecution,
        EthPrecompileProvider, EthValidation, FrameResult,
    },
    handler_interface::{Frame, FrameOrResultGen, PrecompileProvider},
    interpreter::{
        instructions::host::{log, selfdestruct},
        interpreter::{EthInterpreter, InstructionProvider},
        interpreter_types::{Jumps, LoopControl},
        table::{self, CustomInstruction},
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput, Host,
        Instruction, InstructionResult, Interpreter, InterpreterTypes, SStoreResult,
        SelfDestructResult, StateLoad,
    },
    precompile::PrecompileErrors,
    primitives::{Address, Bytes, Log, B256, U256},
    state::EvmState,
    Context, Error, Evm, JournalEntry,
};
use std::{rc::Rc, vec::Vec};

pub struct InspectorInstructionProvider<WIRE: InterpreterTypes, HOST> {
    instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
}

impl<WIRE, HOST> Clone for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}


impl<WIRE, HOST> InstructionProvider for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    type WIRE = WIRE;
    type Host = HOST;

    fn new(_context: &mut Self::Host) -> Self {
        let main_table = table::make_instruction_table::<WIRE, HOST>();
        let mut table: [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, element) in table.iter_mut().enumerate() {
            let function = InspectorInstruction {
                instruction: main_table[i],
            };
            *element = MaybeUninit::new(function);
        }

        let mut table = unsafe {
            core::mem::transmute::<
                [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256],
                [InspectorInstruction<WIRE, HOST>; 256],
            >(table)
        };

        // Inspector log wrapper
        fn inspector_log<CTX: Host + JournalExtGetter + InspectorCtx>(
            interpreter: &mut Interpreter<<CTX as InspectorCtx>::IT>,
            context: &mut CTX,
            prev: Instruction<<CTX as InspectorCtx>::IT, CTX>,
        ) {
            prev(interpreter, context);

            if interpreter.control.instruction_result() == InstructionResult::Continue {
                let last_log = context.journal_ext().logs().last().unwrap().clone();
                context.inspector_log(interpreter, &last_log);
            }
        }

        /* LOG and Selfdestruct instructions */
        table[OpCode::LOG0.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<0, HOST>);
            },
        };
        table[OpCode::LOG1.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<1, HOST>);
            },
        };
        table[OpCode::LOG2.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<2, HOST>);
            },
        };
        table[OpCode::LOG3.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<3, HOST>);
            },
        };
        table[OpCode::LOG4.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<4, HOST>);
            },
        };

        table[OpCode::SELFDESTRUCT.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                selfdestruct::<Self::WIRE, HOST>(interp, context);
                if interp.control.instruction_result() == InstructionResult::SelfDestruct {
                    match context.journal_ext().last_journal().last() {
                        Some(JournalEntry::AccountDestroyed {
                            address,
                            target,
                            had_balance,
                            ..
                        }) => {
                            context.inspector_selfdestruct(*address, *target, *had_balance);
                        }
                        Some(JournalEntry::BalanceTransfer {
                            from, to, balance, ..
                        }) => {
                            context.inspector_selfdestruct(*from, *to, *balance);
                        }
                        _ => {}
                    }
                }
            },
        };

        Self {
            instruction_table: Rc::new(table),
        }
    }

    fn table(&mut self) -> &[impl CustomInstruction<Wire = Self::WIRE, Host = Self::Host>; 256] {
        self.instruction_table.as_ref()
    }
}
