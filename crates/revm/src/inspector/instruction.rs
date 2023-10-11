use alloc::boxed::Box;
use revm_interpreter::{
    instructions::control,
    opcode::{BoxedInstruction, BoxedInstructionTable, Instruction, InstructionTable},
    primitives::{db::Database, Spec},
    InstructionResult, Interpreter,
};

use crate::EVMImpl;

/// Wrap instruction that would call inspector.
pub fn inspector_instruction<'a, SPEC: Spec + 'static, DB: Database>(
    instruction: Instruction<EVMImpl<'a, SPEC, DB>>,
) -> BoxedInstruction<'a, EVMImpl<'a, SPEC, DB>> {
    let inspector_instruction = Box::new(
        move |interpreter: &mut Interpreter<'_>, host: &mut EVMImpl<'a, SPEC, DB>| {
            // step
            let data = &mut host.data;
            // Safety: as the PC was already incremented we need to subtract 1 to preserve the
            // old Inspector behavior.
            unsafe {
                interpreter.instruction_pointer = interpreter.instruction_pointer.sub(1);
            }
            if let Some(inspector) = host.inspector.as_mut() {
                if inspector.step(interpreter, data) != InstructionResult::Continue {
                    return;
                }
            }

            // Safety: return PC to previous state
            unsafe {
                interpreter.instruction_pointer = interpreter.instruction_pointer.add(1);
            }

            // execute instruction.
            instruction(interpreter, host);

            // step ends
            let data = &mut host.data;
            if let Some(inspector) = host.inspector.as_mut() {
                inspector.step_end(interpreter, data);
            }
        },
    );

    inspector_instruction
}

/// make inspector table
pub fn make_inspector_instruction_table<'a, SPEC: Spec + 'static, DB: Database>(
    table: InstructionTable<EVMImpl<'a, SPEC, DB>>,
) -> BoxedInstructionTable<'a, EVMImpl<'a, SPEC, DB>> {
    let mut inspector_table: BoxedInstructionTable<'a, EVMImpl<'a, SPEC, DB>> =
        core::array::from_fn(|_| inspector_instruction(control::not_found));

    for (i, instruction) in table.iter().enumerate() {
        inspector_table[i] = inspector_instruction(*instruction);
    }
    inspector_table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::EmptyDB, interpreter::opcode::*, primitives::BerlinSpec, EVMImpl};

    #[test]
    fn test() {
        // test that this pattern builds.
        let inst: InstructionTable<EVMImpl<'_, BerlinSpec, EmptyDB>> =
            make_instruction_table::<BerlinSpec, EVMImpl<'_, BerlinSpec, _>>();
        let _test: BoxedInstructionTable<'_, EVMImpl<'_, BerlinSpec, _>> =
            make_inspector_instruction_table(inst);
    }
}
