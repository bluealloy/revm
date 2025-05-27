use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{
        EofCodeInfo, Immediates, InterpreterTypes, Jumps, LoopControl, MemoryTr, RuntimeFlag,
        StackTr, SubRoutineStack,
    },
    Host, InstructionResult, InterpreterAction, InterpreterResult,
};
use primitives::{Bytes, U256};

use super::context::InstructionContext;

pub fn rjump<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::BASE);
    let offset = ctx.interpreter.bytecode.read_i16() as isize;
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    ctx.interpreter.bytecode.relative_jump(offset + 2);
}

pub fn rjumpi<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::CONDITION_JUMP_GAS);
    popn!([condition], ctx.interpreter);
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    let mut offset = 2;
    if !condition.is_zero() {
        offset += ctx.interpreter.bytecode.read_i16() as isize;
    }

    ctx.interpreter.bytecode.relative_jump(offset);
}

pub fn rjumpv<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::CONDITION_JUMP_GAS);
    popn!([case], ctx.interpreter);
    let case = as_isize_saturated!(case);

    let max_index = ctx.interpreter.bytecode.read_u8() as isize;
    // For number of items we are adding 1 to max_index, multiply by 2 as each offset is 2 bytes
    // and add 1 for max_index itself. Note that revm already incremented the instruction pointer
    let mut offset = (max_index + 1) * 2 + 1;

    if case <= max_index {
        offset += ctx.interpreter.bytecode.read_offset_i16(1 + case * 2) as isize;
    }
    ctx.interpreter.bytecode.relative_jump(offset);
}

pub fn jump<ITy: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, ITy>) {
    gas!(ctx.interpreter, gas::MID);
    popn!([target], ctx.interpreter);
    jump_inner(ctx.interpreter, target);
}

pub fn jumpi<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::HIGH);
    popn!([target, cond], ctx.interpreter);

    if !cond.is_zero() {
        jump_inner(ctx.interpreter, target);
    }
}

#[inline]
fn jump_inner<WIRE: InterpreterTypes>(interpreter: &mut Interpreter<WIRE>, target: U256) {
    let target = as_usize_or_fail!(interpreter, target, InstructionResult::InvalidJump);
    if !interpreter.bytecode.is_valid_legacy_jump(target) {
        interpreter
            .control
            .set_instruction_result(InstructionResult::InvalidJump);
        return;
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    interpreter.bytecode.absolute_jump(target);
}

pub fn jumpdest_or_nop<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::JUMPDEST);
}

pub fn callf<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::LOW);

    let idx = ctx.interpreter.bytecode.read_u16() as usize;
    // Get target types
    let Some(types) = ctx.interpreter.bytecode.code_info(idx) else {
        panic!("Invalid EOF in execution, expecting correct intermediate in callf")
    };

    // Check max stack height for target code section.
    // Safe to subtract as max_stack_height is always more than inputs.
    if ctx.interpreter.stack.len() + types.max_stack_increase as usize > 1024 {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::StackOverflow);
        return;
    }

    // Push current idx and PC to the callf stack.
    // PC is incremented by 2 to point to the next instruction after callf.
    if !(ctx
        .interpreter
        .sub_routine
        .push(ctx.interpreter.bytecode.pc() + 2, idx))
    {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::SubRoutineStackOverflow);
        return;
    };
    let pc = ctx
        .interpreter
        .bytecode
        .code_section_pc(idx)
        .expect("Invalid code section index");
    ctx.interpreter.bytecode.absolute_jump(pc);
}

pub fn retf<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::RETF_GAS);

    let Some(jump) = ctx.interpreter.sub_routine.pop() else {
        panic!("Expected function frame")
    };

    ctx.interpreter.bytecode.absolute_jump(jump);
}

pub fn jumpf<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::LOW);

    let idx = ctx.interpreter.bytecode.read_u16() as usize;

    // Get target types
    let types = ctx
        .interpreter
        .bytecode
        .code_info(idx)
        .expect("Invalid code section index");

    // Check max stack height for target code section.
    if ctx.interpreter.stack.len() + types.max_stack_increase as usize > 1024 {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::StackOverflow);
        return;
    }
    ctx.interpreter.sub_routine.set_routine_idx(idx);
    let pc = ctx
        .interpreter
        .bytecode
        .code_section_pc(idx)
        .expect("Invalid code section index");
    ctx.interpreter.bytecode.absolute_jump(pc);
}

pub fn pc<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(
        ctx.interpreter,
        U256::from(ctx.interpreter.bytecode.pc() - 1)
    );
}

#[inline]
fn return_inner(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    instruction_result: InstructionResult,
) {
    // Zero gas cost
    // gas!(interpreter, gas::ZERO)
    popn!([offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    // Important: Offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, offset, len);
        output = interpreter.memory.slice_len(offset, len).to_vec().into()
    }

    let gas = *interpreter.control.gas();
    interpreter.control.set_next_action(
        InterpreterAction::Return {
            result: InterpreterResult {
                output,
                gas,
                result: instruction_result,
            },
        },
        instruction_result,
    );
}

pub fn ret<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    return_inner(ctx.interpreter, InstructionResult::Return);
}

/// EIP-140: REVERT instruction
pub fn revert<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    check!(ctx.interpreter, BYZANTIUM);
    return_inner(ctx.interpreter, InstructionResult::Revert);
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    ctx.interpreter
        .control
        .set_instruction_result(InstructionResult::Stop);
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    ctx.interpreter
        .control
        .set_instruction_result(InstructionResult::InvalidFEOpcode);
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    ctx.interpreter
        .control
        .set_instruction_result(InstructionResult::OpcodeNotFound);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::interpreter::SubRoutineReturnFrame;
    use crate::{host::DummyHost, instruction_table, interpreter::EthInterpreter};
    use bytecode::opcode::{CALLF, JUMPF, NOP, RETF, RJUMP, RJUMPI, RJUMPV, STOP};
    use bytecode::{
        eof::{CodeInfo, Eof},
        Bytecode,
    };
    use primitives::bytes;
    use std::sync::Arc;

    #[test]
    fn rjump() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[RJUMP, 0x00, 0x02, STOP, STOP]));
        let mut interpreter = Interpreter::<EthInterpreter>::default().with_bytecode(bytecode);

        interpreter.runtime_flag.is_eof = true;
        let table = instruction_table();
        let mut host = DummyHost;

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        ctx.step(&table);
        assert_eq!(interpreter.bytecode.pc(), 5)
    }

    #[test]
    fn rjumpi() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[
            RJUMPI, 0x00, 0x03, RJUMPI, 0x00, 0x01, STOP, STOP,
        ]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        interpreter.runtime_flag.is_eof = true;
        let table = instruction_table();
        let mut host = DummyHost;

        let _ = interpreter.stack.push(U256::from(1));
        let _ = interpreter.stack.push(U256::from(0));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // Dont jump
        ctx.step(&table);
        assert_eq!(ctx.interpreter.bytecode.pc(), 3);
        // Jumps to last opcode
        ctx.step(&table);
        assert_eq!(interpreter.bytecode.pc(), 7);
    }

    #[test]
    fn rjumpv() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[
            RJUMPV,
            0x01, // max index, 0 and 1
            0x00, // first x0001
            0x01,
            0x00, // second 0x0002
            0x02,
            NOP,
            NOP,
            NOP,
            RJUMP,
            0xFF,
            (-12i8) as u8,
            STOP,
        ]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        interpreter.runtime_flag.is_eof = true;
        let table = instruction_table();
        let mut host = DummyHost;

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // More then max_index
        let _ = ctx.interpreter.stack.push(U256::from(10));
        ctx.step(&table);
        assert_eq!(ctx.interpreter.bytecode.pc(), 6);

        // Cleanup
        ctx.step(&table);
        ctx.step(&table);
        ctx.step(&table);
        ctx.step(&table);
        assert_eq!(ctx.interpreter.bytecode.pc(), 0);

        // Jump to first index of vtable
        let _ = ctx.interpreter.stack.push(U256::from(0));
        ctx.step(&table);
        assert_eq!(ctx.interpreter.bytecode.pc(), 7);

        // Cleanup
        ctx.step(&table);
        ctx.step(&table);
        ctx.step(&table);
        assert_eq!(ctx.interpreter.bytecode.pc(), 0);

        // Jump to second index of vtable
        let _ = ctx.interpreter.stack.push(U256::from(1));
        ctx.step(&table);
        assert_eq!(interpreter.bytecode.pc(), 8);
    }

    fn dummy_eof() -> Eof {
        let bytes = bytes!("ef00010100040200010001ff00000000800000fe");
        Eof::decode(bytes).unwrap()
    }

    fn eof_setup(bytes1: Bytes, bytes2: Bytes) -> Interpreter {
        eof_setup_with_types(bytes1, bytes2, CodeInfo::default())
    }

    /// Two code section and types section is for last code.
    fn eof_setup_with_types(bytes1: Bytes, bytes2: Bytes, types: CodeInfo) -> Interpreter {
        let mut eof = dummy_eof();

        eof.body.code_section.clear();
        eof.body.code_info.clear();
        eof.header.code_sizes.clear();

        eof.header.code_sizes.push(bytes1.len() as u16);
        eof.body.code_section.push(bytes1.len());
        eof.body.code_info.push(CodeInfo::new(0, 0, 11));

        eof.header.code_sizes.push(bytes2.len() as u16);
        eof.body.code_section.push(bytes2.len() + bytes1.len());
        eof.body.code_info.push(types);

        // added two code infos that are 4 bytes each.
        eof.header.types_size = 2 * 4;

        eof.body.code = Bytes::from([bytes1, bytes2].concat());

        // encoding EOF is done se we can generate a raw bytecode.
        // raw bytecode is used to calculate program counter.
        let encoded = eof.encode_slow();

        let bytecode = Bytecode::Eof(Arc::new(Eof::decode(encoded).unwrap()));

        Interpreter::default().with_bytecode(bytecode)
    }

    #[test]
    fn callf_retf_stop() {
        let table = instruction_table();
        let mut host = DummyHost;

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01, STOP]);
        let bytes2 = Bytes::from([RETF]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // CALLF
        ctx.step(&table);

        assert_eq!(ctx.interpreter.sub_routine.current_code_idx, 1);
        assert_eq!(
            ctx.interpreter.sub_routine.return_stack[0],
            SubRoutineReturnFrame::new(0, 3 + base_pc)
        );
        // points to second code section, at RETF opcode
        assert_eq!(ctx.interpreter.bytecode.pc() - base_pc, 4);

        // RETF
        ctx.step(&table);

        assert_eq!(ctx.interpreter.sub_routine.current_code_idx, 0);
        assert_eq!(ctx.interpreter.sub_routine.return_stack, Vec::new());
        // we have returned from the second code section and next opcode is STOP
        assert_eq!(ctx.interpreter.bytecode.pc() - base_pc, 3);

        // STOP
        ctx.step(&table);
        assert_eq!(
            ctx.interpreter.control.instruction_result,
            InstructionResult::Stop
        );
    }

    #[test]
    fn callf_stop() {
        let table = instruction_table();
        let mut host = DummyHost;

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // CALLF
        ctx.step(&table);

        assert_eq!(ctx.interpreter.sub_routine.current_code_idx, 1);
        assert_eq!(
            ctx.interpreter.sub_routine.return_stack[0],
            SubRoutineReturnFrame::new(0, 3 + base_pc)
        );
        // program counter points to STOP of second code section.
        assert_eq!(ctx.interpreter.bytecode.pc(), 3 + base_pc);

        // STOP
        ctx.step(&table);
        assert_eq!(
            ctx.interpreter.control.instruction_result,
            InstructionResult::Stop
        );
    }

    #[test]
    fn callf_stack_overflow() {
        let table = instruction_table();
        let mut host = DummyHost;

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter =
            eof_setup_with_types(bytes1, bytes2.clone(), CodeInfo::new(0, 0, 1023));
        interpreter.runtime_flag.is_eof = true;

        // push two items so we can overflow the CALLF call.
        // overflow happens if max_stack_increase + stack.len is more than 1024
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // CALLF
        ctx.step(&table);

        // Stack overflow
        assert_eq!(
            ctx.interpreter.control.instruction_result,
            InstructionResult::StackOverflow
        );
    }

    #[test]
    fn jumpf_stop() {
        let table = instruction_table();
        let mut host = DummyHost;

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // JUMPF
        ctx.step(&table);

        // fails after this line
        assert_eq!(ctx.interpreter.sub_routine.current_code_idx, 1);
        assert!(ctx.interpreter.sub_routine.return_stack.is_empty());
        // program counter points to STOP of second code section.
        assert_eq!(ctx.interpreter.bytecode.pc(), 3 + base_pc);

        // STOP
        ctx.step(&table);
        assert_eq!(
            ctx.interpreter.control.instruction_result,
            InstructionResult::Stop
        );
    }

    #[test]
    fn jumpf_stack_overflow() {
        let table = instruction_table();
        let mut host = DummyHost;

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01, STOP]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter =
            eof_setup_with_types(bytes1, bytes2.clone(), CodeInfo::new(0, 0, 1023));
        interpreter.runtime_flag.is_eof = true;

        // push two items so we can overflow the JUMPF call.
        // overflow happens if max_stack_size + stack.len is more than 1024
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        // JUMPF
        ctx.step(&table);

        // Stack overflow
        assert_eq!(
            ctx.interpreter.control.instruction_result,
            InstructionResult::StackOverflow
        );
    }
}
