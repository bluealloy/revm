use crate::{
    handler::{inspector_handle_register, register::GetInspector},
    Evm,
};
use alloc::sync::Arc;
use revm_interpreter::{
    opcode::{BoxedInstruction, Instruction},
    primitives::db::Database,
    InstructionResult, Interpreter,
};

/// Outer closure that calls Inspector for every instruction.
pub fn inspector_instruction<
    'a,
    INSP: GetInspector<'a, DB>,
    DB: Database,
    Instruction: Fn(&mut Interpreter, &mut Evm<'a, INSP, DB>) + 'a,
>(
    instruction: Instruction,
) -> BoxedInstruction<'a, Evm<'a, INSP, DB>> {
    Box::new(
        move |interpreter: &mut Interpreter, host: &mut Evm<'a, INSP, DB>| {
            // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
            // old Inspector behavior.
            interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.sub(1) };

            host.context
                .external
                .get_inspector()
                .step(interpreter, &mut host.context.evm);
            if interpreter.instruction_result != InstructionResult::Continue {
                return;
            }

            // return PC to old value
            interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(1) };

            // execute instruction.
            instruction(interpreter, host);

            host.context
                .external
                .get_inspector()
                .step_end(interpreter, &mut host.context.evm);
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspectors::NoOpInspector;
    use crate::{db::EmptyDB, interpreter::opcode::*, primitives::BerlinSpec, Evm};

    #[test]
    fn test_make_boxed_instruction_table() {
        // test that this pattern builds.
        let inst: InstructionTable<Evm<'_, NoOpInspector, EmptyDB>> =
            make_instruction_table::<Evm<'_, _, _>, BerlinSpec>();
        let _test: BoxedInstructionTable<'_, Evm<'_, _, _>> =
            make_boxed_instruction_table::<'_, Evm<'_, NoOpInspector, EmptyDB>, BerlinSpec, _>(
                inst,
                inspector_instruction,
            );
    }
}
