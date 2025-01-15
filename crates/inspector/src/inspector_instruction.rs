use core::mem::MaybeUninit;
use revm::{
    bytecode::opcode::OpCode,
    context_interface::JournalGetter,
    interpreter::{
        instructions::host::{log, selfdestruct},
        interpreter::InstructionProvider,
        interpreter_types::{Jumps, LoopControl},
        table::{self, CustomInstruction},
        Host, Instruction, InstructionResult, Interpreter, InterpreterTypes,
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
        // SAFETY: As the PC was already incremented we need to subtract 1 to preserve the
        // old Inspector behavior.
        interpreter.bytecode.relative_jump(-1);

        // Call step.
        host.step(interpreter);
        if interpreter.control.instruction_result() != InstructionResult::Continue {
            return;
        }

        // Reset PC to previous value.
        interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        (self.instruction)(interpreter, host);

        // Call step_end.
        host.step_end(interpreter);
    }

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self {
        Self { instruction }
    }
}

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

impl<WIRE, HOST> InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    pub fn new() -> Self {
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

impl<WIRE, HOST> InstructionProvider for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    type WIRE = WIRE;
    type Host = HOST;

    fn table(&mut self) -> &[impl CustomInstruction<Wire = Self::WIRE, Host = Self::Host>; 256] {
        self.instruction_table.as_ref()
    }
}
