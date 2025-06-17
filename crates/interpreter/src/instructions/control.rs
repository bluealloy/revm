use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{
        EofCodeInfo, Immediates, InterpreterTypes, Jumps, LoopControl, MemoryTr, RuntimeFlag,
        StackTr, SubRoutineStack,
    },
    InstructionResult, InterpreterAction,
};
use primitives::{Bytes, U256};

use crate::InstructionContext;

pub fn rjump<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::BASE);
    let offset = context.interpreter.bytecode.read_i16() as isize;
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    context.interpreter.bytecode.relative_jump(offset + 2);
}

pub fn rjumpi<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::CONDITION_JUMP_GAS);
    popn!([condition], context.interpreter);
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    let mut offset = 2;
    if !condition.is_zero() {
        offset += context.interpreter.bytecode.read_i16() as isize;
    }

    context.interpreter.bytecode.relative_jump(offset);
}

pub fn rjumpv<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::CONDITION_JUMP_GAS);
    popn!([case], context.interpreter);
    let case = as_isize_saturated!(case);

    let max_index = context.interpreter.bytecode.read_u8() as isize;
    // For number of items we are adding 1 to max_index, multiply by 2 as each offset is 2 bytes
    // and add 1 for max_index itself. Note that revm already incremented the instruction pointer
    let mut offset = (max_index + 1) * 2 + 1;

    if case <= max_index {
        offset += context.interpreter.bytecode.read_offset_i16(1 + case * 2) as isize;
    }
    context.interpreter.bytecode.relative_jump(offset);
}

pub fn jump<ITy: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, ITy>) {
    gas!(context.interpreter, gas::MID);
    popn!([target], context.interpreter);
    jump_inner(context.interpreter, target);
}

pub fn jumpi<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::HIGH);
    popn!([target, cond], context.interpreter);

    if !cond.is_zero() {
        jump_inner(context.interpreter, target);
    }
}

#[inline(always)]
fn jump_inner<WIRE: InterpreterTypes>(interpreter: &mut Interpreter<WIRE>, target: U256) {
    let target = as_usize_or_fail!(interpreter, target, InstructionResult::InvalidJump);
    if !interpreter.bytecode.is_valid_legacy_jump(target) {
        interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    interpreter.bytecode.absolute_jump(target);
}

pub fn jumpdest_or_nop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::JUMPDEST);
}

pub fn callf<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::LOW);

    let idx = context.interpreter.bytecode.read_u16() as usize;
    // Get target types
    let Some(types) = context.interpreter.bytecode.code_info(idx) else {
        panic!("Invalid EOF in execution, expecting correct intermediate in callf")
    };

    // Check max stack height for target code section.
    // Safe to subtract as max_stack_height is always more than inputs.
    if context.interpreter.stack.len() + types.max_stack_increase as usize > 1024 {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }

    // Push current idx and PC to the callf stack.
    // PC is incremented by 2 to point to the next instruction after callf.
    if !(context
        .interpreter
        .sub_routine
        .push(context.interpreter.bytecode.pc() + 2, idx))
    {
        context
            .interpreter
            .halt(InstructionResult::SubRoutineStackOverflow);
        return;
    };
    let pc = context
        .interpreter
        .bytecode
        .code_section_pc(idx)
        .expect("Invalid code section index");
    context.interpreter.bytecode.absolute_jump(pc);
}

pub fn retf<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::RETF_GAS);

    let Some(jump) = context.interpreter.sub_routine.pop() else {
        panic!("Expected function frame")
    };

    context.interpreter.bytecode.absolute_jump(jump);
}

pub fn jumpf<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::LOW);

    let idx = context.interpreter.bytecode.read_u16() as usize;

    // Get target types
    let types = context
        .interpreter
        .bytecode
        .code_info(idx)
        .expect("Invalid code section index");

    // Check max stack height for target code section.
    if context.interpreter.stack.len() + types.max_stack_increase as usize > 1024 {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }
    context.interpreter.sub_routine.set_routine_idx(idx);
    let pc = context
        .interpreter
        .bytecode
        .code_section_pc(idx)
        .expect("Invalid code section index");
    context.interpreter.bytecode.absolute_jump(pc);
}

pub fn pc<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.pc() - 1)
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

    interpreter
        .bytecode
        .set_action(InterpreterAction::new_return(
            instruction_result,
            output,
            interpreter.gas,
        ));
}

pub fn ret<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    return_inner(context.interpreter, InstructionResult::Return);
}

/// EIP-140: REVERT instruction
pub fn revert<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, BYZANTIUM);
    return_inner(context.interpreter, InstructionResult::Revert);
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::Stop);
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::InvalidFEOpcode);
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    context.interpreter.halt(InstructionResult::OpcodeNotFound);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::interpreter::SubRoutineReturnFrame;
    use crate::{instruction_table, interpreter::EthInterpreter};
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

        interpreter.step_dummy(&table);
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

        let _ = interpreter.stack.push(U256::from(1));
        let _ = interpreter.stack.push(U256::from(0));

        // Dont jump
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.bytecode.pc(), 3);
        // Jumps to last opcode
        interpreter.step_dummy(&table);
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

        // More then max_index
        let _ = interpreter.stack.push(U256::from(10));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.bytecode.pc(), 6);

        // Cleanup
        interpreter.step_dummy(&table);
        interpreter.step_dummy(&table);
        interpreter.step_dummy(&table);
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.bytecode.pc(), 0);

        // Jump to first index of vtable
        let _ = interpreter.stack.push(U256::from(0));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.bytecode.pc(), 7);

        // Cleanup
        interpreter.step_dummy(&table);
        interpreter.step_dummy(&table);
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.bytecode.pc(), 0);

        // Jump to second index of vtable
        let _ = interpreter.stack.push(U256::from(1));
        interpreter.step_dummy(&table);
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

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01, STOP]);
        let bytes2 = Bytes::from([RETF]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        // CALLF
        interpreter.step_dummy(&table);

        assert_eq!(interpreter.sub_routine.current_code_idx, 1);
        assert_eq!(
            interpreter.sub_routine.return_stack[0],
            SubRoutineReturnFrame::new(0, 3 + base_pc)
        );
        // points to second code section, at RETF opcode
        assert_eq!(interpreter.bytecode.pc() - base_pc, 4);

        // RETF
        interpreter.step_dummy(&table);

        assert_eq!(interpreter.sub_routine.current_code_idx, 0);
        assert_eq!(interpreter.sub_routine.return_stack, Vec::new());
        // we have returned from the second code section and next opcode is STOP
        assert_eq!(interpreter.bytecode.pc() - base_pc, 3);

        // STOP
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_end());
    }

    #[test]
    fn callf_stop() {
        let table = instruction_table();

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        // CALLF
        interpreter.step_dummy(&table);

        assert_eq!(interpreter.sub_routine.current_code_idx, 1);
        assert_eq!(
            interpreter.sub_routine.return_stack[0],
            SubRoutineReturnFrame::new(0, 3 + base_pc)
        );
        // program counter points to STOP of second code section.
        assert_eq!(interpreter.bytecode.pc(), 3 + base_pc);

        // STOP
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_end());
    }

    #[test]
    fn callf_stack_overflow() {
        let table = instruction_table();

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter =
            eof_setup_with_types(bytes1, bytes2.clone(), CodeInfo::new(0, 0, 1023));
        interpreter.runtime_flag.is_eof = true;

        // push two items so we can overflow the CALLF call.
        // overflow happens if max_stack_increase + stack.len is more than 1024
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        // CALLF
        interpreter.step_dummy(&table);

        // Stack overflow
        assert_eq!(
            interpreter.take_next_action(),
            InterpreterAction::new_halt(InstructionResult::StackOverflow, interpreter.gas)
        );
    }

    #[test]
    fn jumpf_stop() {
        let table = instruction_table();

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter = eof_setup(bytes1, bytes2.clone());
        interpreter.runtime_flag.is_eof = true;
        let base_pc = interpreter.bytecode.pc();

        // JUMPF
        interpreter.step_dummy(&table);

        // fails after this line
        assert_eq!(interpreter.sub_routine.current_code_idx, 1);
        assert!(interpreter.sub_routine.return_stack.is_empty());
        // program counter points to STOP of second code section.
        assert_eq!(interpreter.bytecode.pc(), 3 + base_pc);

        // STOP
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_end());
    }

    #[test]
    fn jumpf_stack_overflow() {
        let table = instruction_table();

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01, STOP]);
        let bytes2 = Bytes::from([STOP]);
        let mut interpreter =
            eof_setup_with_types(bytes1, bytes2.clone(), CodeInfo::new(0, 0, 1023));
        interpreter.runtime_flag.is_eof = true;

        // push two items so we can overflow the JUMPF call.
        // overflow happens if max_stack_size + stack.len is more than 1024
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        // JUMPF
        interpreter.step_dummy(&table);

        let gas = interpreter.gas;
        // Stack overflow
        assert_eq!(
            interpreter.take_next_action(),
            InterpreterAction::new_halt(InstructionResult::StackOverflow, gas)
        );
    }
}
