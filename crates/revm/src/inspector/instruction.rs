use crate::EVMImpl;
use alloc::boxed::Box;
use revm_interpreter::{
    opcode::{BoxedInstruction, Instruction},
    primitives::{db::Database, Spec},
    InstructionResult, Interpreter,
};

/// Outer closure that calls Inspector for every instruction.
pub fn inspector_instruction<'a, SPEC: Spec + 'static, DB: Database>(
    instruction: Instruction<EVMImpl<'a, SPEC, DB>>,
) -> BoxedInstruction<'a, EVMImpl<'a, SPEC, DB>> {
    Box::new(
        move |interpreter: &mut Interpreter, host: &mut EVMImpl<'a, SPEC, DB>| {
            if let Some(inspector) = host.inspector.as_mut() {
                // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
                // old Inspector behavior.
                interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.sub(1) };

                inspector.step(interpreter, &mut host.context);
                if interpreter.instruction_result != InstructionResult::Continue {
                    return;
                }

                interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(1) };
            }

            // execute instruction.
            instruction(interpreter, host);

            // step ends
            if let Some(inspector) = host.inspector.as_mut() {
                inspector.step_end(interpreter, &mut host.context);
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::EmptyDB, interpreter::opcode::*, primitives::BerlinSpec, EVMImpl};

    #[test]
    fn test_make_boxed_instruction_table() {
        // test that this pattern builds.
        let inst: InstructionTable<EVMImpl<'_, BerlinSpec, EmptyDB>> =
            make_instruction_table::<EVMImpl<'_, BerlinSpec, _>, BerlinSpec>();
        let _test: BoxedInstructionTable<'_, EVMImpl<'_, BerlinSpec, _>> =
            make_boxed_instruction_table::<'_, EVMImpl<'_, BerlinSpec, EmptyDB>, BerlinSpec, _>(
                inst,
                inspector_instruction,
            );
    }
}
