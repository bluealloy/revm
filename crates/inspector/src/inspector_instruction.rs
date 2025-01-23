use core::mem::MaybeUninit;
use revm::{
    bytecode::opcode::OpCode,
    context_interface::JournalGetter,
    handler::instructions::InstructionExecutor,
    interpreter::{
        instructions::host::{log, selfdestruct},
        interpreter_types::{Jumps, LoopControl},
        table::{make_instruction_table, CustomInstruction, InstructionTable},
        Host, Instruction, InstructionResult, Interpreter, InterpreterAction, InterpreterTypes,
    },
    JournalEntry,
};
use std::rc::Rc;

use crate::{
    journal::{JournalExt, JournalExtGetter},
    InspectorCtx,
};

#[derive(Clone)]
pub struct InspectorInstruction<IT: InterpreterTypes, HOST> {
    pub instruction: fn(&mut Interpreter<IT>, &mut HOST),
}

impl<IT: InterpreterTypes, HOST> CustomInstruction for InspectorInstruction<IT, HOST>
where
    HOST: InspectorCtx<IT = IT>,
{
    type Wire = IT;
    type Host = HOST;

    fn exec(&self, interpreter: &mut Interpreter<Self::Wire>, host: &mut Self::Host) {
        (self.instruction)(interpreter, host);
    }

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self {
        Self { instruction }
    }
}

pub struct InspectorInstructionExecutor<WIRE: InterpreterTypes, HOST> {
    instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
}

impl<WIRE, HOST> Clone for InspectorInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

impl<WIRE, HOST> InspectorInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    pub fn new(base_table: InstructionTable<WIRE, HOST>) -> Self {
        let mut table: [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, element) in table.iter_mut().enumerate() {
            let function: InspectorInstruction<WIRE, HOST> = InspectorInstruction {
                instruction: base_table[i],
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
                selfdestruct::<WIRE, HOST>(interp, context);
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
}

impl<IT, CTX> InstructionExecutor for InspectorInstructionExecutor<IT, CTX>
where
    IT: InterpreterTypes,
    CTX: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = IT>,
{
    type InterpreterTypes = IT;
    type CTX = CTX;
    type Output = InterpreterAction;

    fn run(
        &mut self,
        context: &mut Self::CTX,
        interpreter: &mut Interpreter<Self::InterpreterTypes>,
    ) -> Self::Output {
        interpreter.reset_control();

        // Main loop
        while interpreter.control.instruction_result().is_continue() {
            // Get current opcode.
            let opcode = interpreter.bytecode.opcode();

            // Call Inspector step.
            context.step(interpreter);
            if interpreter.control.instruction_result() != InstructionResult::Continue {
                break;
            }

            // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
            // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
            // it will do noop and just stop execution of this contract
            interpreter.bytecode.relative_jump(1);

            // Execute instruction.
            self.instruction_table[opcode as usize].exec(interpreter, context);

            // Call step_end.
            context.step_end(interpreter);
        }

        interpreter.take_next_action()
    }
}

impl<WIRE, HOST> Default for InspectorInstructionExecutor<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    fn default() -> Self {
        Self::new(make_instruction_table())
    }
}
