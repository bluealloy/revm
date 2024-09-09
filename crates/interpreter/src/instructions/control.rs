use super::utility::{read_i16, read_u16};
use crate::{
    gas,
    primitives::{Bytes, Spec, U256},
    Host, InstructionResult, Interpreter, InterpreterResult,
};

pub fn rjump<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::BASE);
    let offset = unsafe { read_i16(interpreter.instruction_pointer) } as isize;
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(offset + 2) };
}

pub fn rjumpi<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::CONDITION_JUMP_GAS);
    pop!(interpreter, condition);
    // In spec it is +3 but pointer is already incremented in
    // `Interpreter::step` so for revm is +2.
    let mut offset = 2;
    if !condition.is_zero() {
        offset += unsafe { read_i16(interpreter.instruction_pointer) } as isize;
    }

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(offset) };
}

pub fn rjumpv<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::CONDITION_JUMP_GAS);
    pop!(interpreter, case);
    let case = as_isize_saturated!(case);

    let max_index = unsafe { *interpreter.instruction_pointer } as isize;
    // for number of items we are adding 1 to max_index, multiply by 2 as each offset is 2 bytes
    // and add 1 for max_index itself. Note that revm already incremented the instruction pointer
    let mut offset = (max_index + 1) * 2 + 1;

    if case <= max_index {
        offset += unsafe {
            read_i16(
                interpreter
                    .instruction_pointer
                    // offset for max_index that is one byte
                    .offset(1 + case * 2),
            )
        } as isize;
    }

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(offset) };
}

pub fn jump<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::MID);
    pop!(interpreter, target);
    jump_inner(interpreter, target);
}

pub fn jumpi<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::HIGH);
    pop!(interpreter, target, cond);
    if !cond.is_zero() {
        jump_inner(interpreter, target);
    }
}

#[inline]
fn jump_inner(interpreter: &mut Interpreter, target: U256) {
    let target = as_usize_or_fail!(interpreter, target, InstructionResult::InvalidJump);
    if !interpreter.contract.is_valid_jump(target) {
        interpreter.instruction_result = InstructionResult::InvalidJump;
        return;
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    interpreter.instruction_pointer = unsafe { interpreter.bytecode.as_ptr().add(target) };
}

pub fn jumpdest_or_nop<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::JUMPDEST);
}

pub fn callf<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::LOW);

    let idx = unsafe { read_u16(interpreter.instruction_pointer) } as usize;

    if interpreter.function_stack.return_stack_len() >= 1024 {
        interpreter.instruction_result = InstructionResult::EOFFunctionStackOverflow;
        return;
    }

    // get target types
    let Some(types) = interpreter.eof().unwrap().body.types_section.get(idx) else {
        panic!("Invalid EOF in execution, expecting correct intermediate in callf")
    };

    // Check max stack height for target code section.
    // safe to subtract as max_stack_height is always more than inputs.
    if interpreter.stack.len() + (types.max_stack_size - types.inputs as u16) as usize > 1024 {
        interpreter.instruction_result = InstructionResult::StackOverflow;
        return;
    }

    // push current idx and PC to the callf stack.
    // PC is incremented by 2 to point to the next instruction after callf.
    interpreter
        .function_stack
        .push(interpreter.program_counter() + 2, idx);

    interpreter.load_eof_code(idx, 0)
}

pub fn retf<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::RETF_GAS);

    let Some(fframe) = interpreter.function_stack.pop() else {
        panic!("Expected function frame")
    };

    interpreter.load_eof_code(fframe.idx, fframe.pc);
}

pub fn jumpf<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::LOW);

    let idx = unsafe { read_u16(interpreter.instruction_pointer) } as usize;

    // get target types
    let Some(types) = interpreter.eof().unwrap().body.types_section.get(idx) else {
        panic!("Invalid EOF in execution, expecting correct intermediate in jumpf")
    };

    // Check max stack height for target code section.
    // safe to subtract as max_stack_height is always more than inputs.
    if interpreter.stack.len() + (types.max_stack_size - types.inputs as u16) as usize > 1024 {
        interpreter.instruction_result = InstructionResult::StackOverflow;
        return;
    }

    interpreter.function_stack.set_current_code_idx(idx);
    interpreter.load_eof_code(idx, 0)
}

pub fn pc<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(interpreter, U256::from(interpreter.program_counter() - 1));
}

#[inline]
fn return_inner(interpreter: &mut Interpreter, instruction_result: InstructionResult) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    // important: offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, offset, len);

        output = interpreter.shared_memory.slice(offset, len).to_vec().into()
    }
    interpreter.instruction_result = instruction_result;
    interpreter.next_action = crate::InterpreterAction::Return {
        result: InterpreterResult {
            output,
            gas: interpreter.gas,
            result: instruction_result,
        },
    };
}

pub fn ret<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    return_inner(interpreter, InstructionResult::Return);
}

/// EIP-140: REVERT instruction
pub fn revert<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, BYZANTIUM);
    return_inner(interpreter, InstructionResult::Revert);
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::Stop;
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::InvalidFEOpcode;
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::OpcodeNotFound;
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use revm_primitives::{
        bytes, eof::TypesSection, Bytecode, DefaultEthereumWiring, Eof, PragueSpec,
    };

    use super::*;
    use crate::{
        opcode::{make_instruction_table, CALLF, JUMPF, NOP, RETF, RJUMP, RJUMPI, RJUMPV, STOP},
        DummyHost, FunctionReturnFrame, Gas, Interpreter,
    };

    #[test]
    fn rjump() {
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([
            RJUMP, 0x00, 0x02, STOP, STOP,
        ])));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);

        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 5);
    }

    #[test]
    fn rjumpi() {
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([
            RJUMPI, 0x00, 0x03, RJUMPI, 0x00, 0x01, STOP, STOP,
        ])));
        interp.is_eof = true;
        interp.stack.push(U256::from(1)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.gas = Gas::new(10000);

        // dont jump
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 3);
        // jumps to last opcode
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 7);
    }

    #[test]
    fn rjumpv() {
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([
            RJUMPV,
            0x01, // max index, 0 and 1
            0x00, // first x0001
            0x01,
            0x00, // second 0x002
            0x02,
            NOP,
            NOP,
            NOP,
            RJUMP,
            0xFF,
            (-12i8) as u8,
            STOP,
        ])));
        interp.is_eof = true;
        interp.gas = Gas::new(1000);

        // more then max_index
        interp.stack.push(U256::from(10)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 6);

        // cleanup
        interp.step(&table, &mut host);
        interp.step(&table, &mut host);
        interp.step(&table, &mut host);
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 0);

        // jump to first index of vtable
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 7);

        // cleanup
        interp.step(&table, &mut host);
        interp.step(&table, &mut host);
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 0);

        // jump to second index of vtable
        interp.stack.push(U256::from(1)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.program_counter(), 8);
    }

    fn dummy_eof() -> Eof {
        let bytes = bytes!("ef000101000402000100010400000000800000fe");
        Eof::decode(bytes).unwrap()
    }

    fn eof_setup(bytes1: Bytes, bytes2: Bytes) -> Interpreter {
        eof_setup_with_types(bytes1, bytes2, TypesSection::default())
    }

    /// Two code section and types section is for last code.
    fn eof_setup_with_types(bytes1: Bytes, bytes2: Bytes, types: TypesSection) -> Interpreter {
        let mut eof = dummy_eof();

        eof.body.code_section.clear();
        eof.body.types_section.clear();
        eof.header.code_sizes.clear();

        eof.header.code_sizes.push(bytes1.len() as u16);
        eof.body.code_section.push(bytes1.clone());
        eof.body.types_section.push(TypesSection::new(0, 0, 11));

        eof.header.code_sizes.push(bytes2.len() as u16);
        eof.body.code_section.push(bytes2.clone());
        eof.body.types_section.push(types);

        let mut interp = Interpreter::new_bytecode(Bytecode::Eof(Arc::new(eof)));
        interp.gas = Gas::new(10000);
        interp
    }

    #[test]
    fn callf_retf_stop() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01, STOP]);
        let bytes2 = Bytes::from([RETF]);
        let mut interp = eof_setup(bytes1, bytes2.clone());

        // CALLF
        interp.step(&table, &mut host);

        assert_eq!(interp.function_stack.current_code_idx, 1);
        assert_eq!(
            interp.function_stack.return_stack[0],
            FunctionReturnFrame::new(0, 3)
        );
        assert_eq!(interp.instruction_pointer, bytes2.as_ptr());

        // RETF
        interp.step(&table, &mut host);

        assert_eq!(interp.function_stack.current_code_idx, 0);
        assert_eq!(interp.function_stack.return_stack, Vec::new());
        assert_eq!(interp.program_counter(), 3);

        // STOP
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Stop);
    }

    #[test]
    fn callf_stop() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interp = eof_setup(bytes1, bytes2.clone());

        // CALLF
        interp.step(&table, &mut host);

        assert_eq!(interp.function_stack.current_code_idx, 1);
        assert_eq!(
            interp.function_stack.return_stack[0],
            FunctionReturnFrame::new(0, 3)
        );
        assert_eq!(interp.instruction_pointer, bytes2.as_ptr());

        // STOP
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Stop);
    }

    #[test]
    fn callf_stack_overflow() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

        let bytes1 = Bytes::from([CALLF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interp =
            eof_setup_with_types(bytes1, bytes2.clone(), TypesSection::new(0, 0, 1025));

        // CALLF
        interp.step(&table, &mut host);

        // stack overflow
        assert_eq!(interp.instruction_result, InstructionResult::StackOverflow);
    }

    #[test]
    fn jumpf_stop() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interp = eof_setup(bytes1, bytes2.clone());

        // JUMPF
        interp.step(&table, &mut host);

        assert_eq!(interp.function_stack.current_code_idx, 1);
        assert!(interp.function_stack.return_stack.is_empty());
        assert_eq!(interp.instruction_pointer, bytes2.as_ptr());

        // STOP
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Stop);
    }

    #[test]
    fn jumpf_stack_overflow() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

        let bytes1 = Bytes::from([JUMPF, 0x00, 0x01]);
        let bytes2 = Bytes::from([STOP]);
        let mut interp =
            eof_setup_with_types(bytes1, bytes2.clone(), TypesSection::new(0, 0, 1025));

        // JUMPF
        interp.step(&table, &mut host);

        // stack overflow
        assert_eq!(interp.instruction_result, InstructionResult::StackOverflow);
    }
}
