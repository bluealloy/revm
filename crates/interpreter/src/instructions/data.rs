use crate::{
    gas::{cost_per_word, BASE, DATA_LOAD_GAS, VERYLOW},
    interpreter_types::{
        EofData, Immediates, InterpreterTypes, Jumps, MemoryTr, RuntimeFlag, StackTr,
    },
};
use primitives::{B256, U256};

use crate::InstructionContext;

/// Implements the DATALOAD instruction.
///
/// Loads a 32-byte word from the data section at a dynamic offset (EOF only).
pub fn data_load<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, DATA_LOAD_GAS);
    popn_top!([], offset, context.interpreter);

    let offset_usize = as_usize_saturated!(offset);

    let slice = context.interpreter.bytecode.data_slice(offset_usize, 32);

    let mut word = [0u8; 32];
    word[..slice.len()].copy_from_slice(slice);

    *offset = U256::from_be_bytes(word);
}

/// Implements the DATALOADN instruction.
///
/// Loads a 32-byte word from the data section at an immediate offset (EOF only).
pub fn data_loadn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, VERYLOW);
    let offset = context.interpreter.bytecode.read_u16() as usize;

    let slice = context.interpreter.bytecode.data_slice(offset, 32);

    let mut word = [0u8; 32];
    word[..slice.len()].copy_from_slice(slice);

    push!(context.interpreter, B256::new(word).into());

    // Add +2 to the instruction pointer to skip the offset
    context.interpreter.bytecode.relative_jump(2);
}

/// Implements the DATASIZE instruction.
///
/// Pushes the size of the data section onto the stack (EOF only).
pub fn data_size<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, BASE);

    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.data_size())
    );
}

/// Implements the DATACOPY instruction.
///
/// Copies data from the data section to memory (EOF only).
pub fn data_copy<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, VERYLOW);
    popn!([mem_offset, offset, size], context.interpreter);

    // Sizes more than u64::MAX will spend all the gas in memory resize.
    let size = as_usize_or_fail!(context.interpreter, size);
    // Size of zero should not change the memory
    if size == 0 {
        return;
    }
    // Fail if mem offset is big as it will spend all the gas
    let mem_offset = as_usize_or_fail!(context.interpreter, mem_offset);
    resize_memory!(context.interpreter, mem_offset, size);

    gas_or_fail!(context.interpreter, cost_per_word(size, VERYLOW));

    let offset = as_usize_saturated!(offset);
    let data = context.interpreter.bytecode.data();

    // Set data from the eof to the shared memory. Padded it with zeros.
    context
        .interpreter
        .memory
        .set_data(mem_offset, offset, size, data);
}

#[cfg(test)]
mod test {
    use bytecode::{Bytecode, Eof};
    use primitives::{b256, bytes, Bytes};
    use std::sync::Arc;

    use super::*;
    use crate::{instruction_table, Interpreter};
    use bytecode::opcode::{DATACOPY, DATALOAD, DATALOADN, DATASIZE};

    fn dummy_eof(code_bytes: Bytes) -> Bytecode {
        let bytes = bytes!("ef00010100040200010001ff00000000800000fe");
        let mut eof = Eof::decode(bytes).unwrap();

        eof.body.data_section =
            bytes!("000000000000000000000000000000000000000000000000000000000000000102030405");
        eof.header.data_size = eof.body.data_section.len() as u16;

        eof.header.code_sizes[0] = code_bytes.len() as u16;
        eof.body.code_section[0] = code_bytes.len();
        eof.body.code = code_bytes;
        Bytecode::Eof(Arc::new(eof))
    }

    #[test]
    fn dataload_dataloadn() {
        let table = instruction_table();

        let eof = dummy_eof(Bytes::from([
            DATALOAD, DATALOADN, 0x00, 0x00, DATALOAD, DATALOADN, 0x00, 35, DATALOAD, DATALOADN,
            0x00, 36, DATASIZE,
        ]));

        let mut interpreter = Interpreter::default().with_bytecode(eof);
        interpreter.runtime_flag.is_eof = true;

        // DATALOAD
        let _ = interpreter.stack.push(U256::from(0));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.data(), &vec![U256::from(0x01)]);
        interpreter.stack.pop().unwrap();

        // DATALOADN
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.data(), &vec![U256::from(0x01)]);
        interpreter.stack.pop().unwrap();

        // DATALOAD (padding)
        let _ = interpreter.stack.push(U256::from(35));
        interpreter.step_dummy(&table);

        assert_eq!(
            interpreter.stack.data(),
            &vec![b256!("0500000000000000000000000000000000000000000000000000000000000000").into()]
        );
        interpreter.stack.pop().unwrap();

        // DATALOADN (padding)
        interpreter.step_dummy(&table);
        assert_eq!(
            interpreter.stack.data(),
            &vec![b256!("0500000000000000000000000000000000000000000000000000000000000000").into()]
        );
        interpreter.stack.pop().unwrap();

        // DATALOAD (out of bounds)
        let _ = interpreter.stack.push(U256::from(36));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.data(), &vec![U256::ZERO]);
        interpreter.stack.pop().unwrap();

        // DATALOADN (out of bounds)
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.data(), &vec![U256::ZERO]);
        interpreter.stack.pop().unwrap();

        // DATA SIZE
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.data(), &vec![U256::from(36)]);
    }

    #[test]
    fn data_copy() {
        let table = instruction_table();
        let eof = dummy_eof(Bytes::from([DATACOPY, DATACOPY, DATACOPY, DATACOPY]));

        let mut interpreter = Interpreter::default().with_bytecode(eof);
        interpreter.runtime_flag.is_eof = true;

        // Data copy
        // size, offset mem_offset,
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        interpreter.step_dummy(&table);
        assert_eq!(
            *interpreter.memory.context_memory(),
            bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Padding)
        // size, offset mem_offset,
        let _ = interpreter.stack.push(U256::from(2));
        let _ = interpreter.stack.push(U256::from(35));
        let _ = interpreter.stack.push(U256::from(1));
        interpreter.step_dummy(&table);
        assert_eq!(
            *interpreter.memory.context_memory(),
            bytes!("0005000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Out of bounds)
        // size, offset mem_offset,
        let _ = interpreter.stack.push(U256::from(2));
        let _ = interpreter.stack.push(U256::from(37));
        let _ = interpreter.stack.push(U256::from(1));
        interpreter.step_dummy(&table);
        assert_eq!(
            *interpreter.memory.context_memory(),
            bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Size == 0)
        // mem_offset, offset, size
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(37));
        let _ = interpreter.stack.push(U256::from(1));
        interpreter.step_dummy(&table);
        assert_eq!(
            *interpreter.memory.context_memory(),
            bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );
    }
}
