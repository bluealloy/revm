use crate::Evm;
use alloc::boxed::Box;
use revm_interpreter::{
    opcode::{BoxedInstruction, Instruction},
    primitives::{db::Database, Spec},
    InstructionResult, Interpreter,
};

/// Outer closure that calls Inspector for every instruction.
pub fn inspector_instruction<'a, SPEC: Spec + 'static, EXT: 'a, DB: Database>(
    instruction: Instruction<Evm<'a, SPEC, EXT, DB>>,
) -> BoxedInstruction<'a, Evm<'a, SPEC, EXT, DB>> {
    Box::new(
        move |interpreter: &mut Interpreter, host: &mut Evm<'a, SPEC, EXT, DB>| {
            if let Some(inspector) = host.inspector.as_mut() {
                // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
                // old Inspector behavior.
                interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.sub(1) };

                inspector.step(interpreter, &mut host.context.evm);
                if interpreter.instruction_result != InstructionResult::Continue {
                    return;
                }

                interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(1) };
            }

            // execute instruction.
            instruction(interpreter, host);

            // step ends
            if let Some(inspector) = host.inspector.as_mut() {
                inspector.step_end(interpreter, &mut host.context.evm);
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::EmptyDB, interpreter::opcode::*, primitives::BerlinSpec, Evm};

    #[test]
    fn test_make_boxed_instruction_table() {
        // test that this pattern builds.
        let inst: InstructionTable<Evm<'_, BerlinSpec, (), EmptyDB>> =
            make_instruction_table::<Evm<'_, BerlinSpec, _, _>, BerlinSpec>();
        let _test: BoxedInstructionTable<'_, Evm<'_, BerlinSpec, _, _>> =
            make_boxed_instruction_table::<'_, Evm<'_, BerlinSpec, _, EmptyDB>, BerlinSpec, _>(
                inst,
                inspector_instruction,
            );
    }
}
