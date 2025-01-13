use crate::{
    gas::{cost_per_word, BASE, DATA_LOAD_GAS, VERYLOW},
    interpreter::Interpreter,
    interpreter_types::{
        EofData, Immediates, InterpreterTypes, Jumps, LoopControl, MemoryTrait, RuntimeFlag,
        StackTrait,
    },
    Host,
};
use primitives::{B256, U256};

pub fn data_load<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, DATA_LOAD_GAS);
    popn_top!([], offset, interpreter);

    let offset_usize = as_usize_saturated!(offset);

    let slice = interpreter.bytecode.data_slice(offset_usize, 32);

    let mut word = [0u8; 32];
    word[..slice.len()].copy_from_slice(slice);

    *offset = U256::from_be_bytes(word);
}

pub fn data_loadn<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, VERYLOW);
    let offset = interpreter.bytecode.read_u16() as usize;

    let slice = interpreter.bytecode.data_slice(offset, 32);

    let mut word = [0u8; 32];
    word[..slice.len()].copy_from_slice(slice);

    push!(interpreter, B256::new(word).into());

    // Add +2 to the instruction pointer to skip the offset
    interpreter.bytecode.relative_jump(2);
}

pub fn data_size<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, BASE);

    push!(interpreter, U256::from(interpreter.bytecode.data_size()));
}

pub fn data_copy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, VERYLOW);
    popn!([mem_offset, offset, size], interpreter);

    // Sizes more than u64::MAX will spend all the gas in memory resize.
    let size = as_usize_or_fail!(interpreter, size);
    // Size of zero should not change the memory
    if size == 0 {
        return;
    }
    // Fail if mem offset is big as it will spend all the gas
    let mem_offset = as_usize_or_fail!(interpreter, mem_offset);
    resize_memory!(interpreter, mem_offset, size);

    gas_or_fail!(interpreter, cost_per_word(size, VERYLOW));

    let offset = as_usize_saturated!(offset);
    let data = interpreter.bytecode.data();

    // Set data from the eof to the shared memory. Padded it with zeros.
    interpreter.memory.set_data(mem_offset, offset, size, data);
}

// TODO : Test
/*
#[cfg(test)]
mod test {
    use bytecode::{Bytecode, Eof};
    use primitives::{b256, bytes, Bytes};
    use specification::hardfork::SpecId;
    use std::sync::Arc;
    use context_interface::DefaultEthereumWiring;

    use super::*;
    use crate::{table::make_instruction_table, DummyHost, Gas};
    use bytecode::opcode::{DATACOPY, DATALOAD, DATALOADN, DATASIZE};

    fn dummy_eof(code_bytes: Bytes) -> Bytecode {
        let bytes = bytes!("ef000101000402000100010400000000800000fe");
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
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
        let mut host = DummyHost::default();
        let eof = dummy_eof(Bytes::from([
            DATALOAD, DATALOADN, 0x00, 0x00, DATALOAD, DATALOADN, 0x00, 35, DATALOAD, DATALOADN,
            0x00, 36, DATASIZE,
        ]));

        let mut interp = Interpreter::new_bytecode(eof);
        interp.spec_id = SpecId::PRAGUE;
        interp.gas = Gas::new(10000);

        // DATALOAD
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.data(), &vec![U256::from(0x01)]);
        interp.stack.pop().unwrap();

        // DATALOADN
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.data(), &vec![U256::from(0x01)]);
        interp.stack.pop().unwrap();

        // DATALOAD (padding)
        interp.stack.push(U256::from(35)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(
            interp.stack.data(),
            &vec![b256!("0500000000000000000000000000000000000000000000000000000000000000").into()]
        );
        interp.stack.pop().unwrap();

        // DATALOADN (padding)
        interp.step(&table, &mut host);
        assert_eq!(
            interp.stack.data(),
            &vec![b256!("0500000000000000000000000000000000000000000000000000000000000000").into()]
        );
        interp.stack.pop().unwrap();

        // DATALOAD (out of bounds)
        interp.stack.push(U256::from(36)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.data(), &vec![U256::ZERO]);
        interp.stack.pop().unwrap();

        // DATALOADN (out of bounds)
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.data(), &vec![U256::ZERO]);
        interp.stack.pop().unwrap();

        // DATA SIZE
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.data(), &vec![U256::from(36)]);
    }

    #[test]
    fn data_copy() {
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
        let mut host = DummyHost::default();
        let eof = dummy_eof(Bytes::from([DATACOPY, DATACOPY, DATACOPY, DATACOPY]));

        let mut interp = Interpreter::new_bytecode(eof);
        interp.gas = Gas::new(10000);
        interp.spec_id = SpecId::PRAGUE;

        // Data copy
        // size, offset mem_offset,
        interp.stack.push(U256::from(32)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(
            interp.shared_memory.context_memory(),
            &bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Padding)
        // size, offset mem_offset,
        interp.stack.push(U256::from(2)).unwrap();
        interp.stack.push(U256::from(35)).unwrap();
        interp.stack.push(U256::from(1)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(
            interp.shared_memory.context_memory(),
            &bytes!("0005000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Out of bounds)
        // size, offset mem_offset,
        interp.stack.push(U256::from(2)).unwrap();
        interp.stack.push(U256::from(37)).unwrap();
        interp.stack.push(U256::from(1)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(
            interp.shared_memory.context_memory(),
            &bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );

        // Data copy (Size == 0)
        // mem_offset, offset, size
        interp.stack.push(U256::from(0)).unwrap();
        interp.stack.push(U256::from(37)).unwrap();
        interp.stack.push(U256::from(1)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(
            interp.shared_memory.context_memory(),
            &bytes!("0000000000000000000000000000000000000000000000000000000000000001")
        );
    }
}
 */
