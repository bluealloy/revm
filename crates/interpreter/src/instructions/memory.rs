use core::ops::Index;

use crate::{gas, interpreter::Interpreter, primitives::U256, Host, InstructionResult};
use core::cmp::max;

use revm_primitives::{Spec, SpecId::CANCUN};

pub fn mload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 32);
    push!(
        interpreter,
        U256::from_be_bytes::<{ U256::BYTES }>(
            interpreter.memory.get_slice(index, 32).try_into().unwrap()
        )
    );
}

pub fn mstore(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 32);
    interpreter.memory.set_u256(index, value);
}

pub fn mstore8(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 1);
    let value = value.as_le_bytes()[0];
    // Safety: we resized our memory two lines above.
    unsafe { interpreter.memory.set_byte(index, value) }
}

pub fn msize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.effective_len()));
}

// From EIP-5656 MCOPY
pub fn mcopy<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // Opcode enabled in Cancun.
    // EIP-5656: MCOPY - Memory copying instruction
    check!(interpreter, SPEC::enabled(CANCUN));
    // get src and dest and length from stack
    pop!(interpreter, dest, src, len);

    // into usize or fail
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    // deduce gas
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }

    let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidOperandOOG);
    let src = as_usize_or_fail!(interpreter, src, InstructionResult::InvalidOperandOOG);
    // resize memory
    memory_resize!(interpreter, max(dest, src), len);
    // copy memory in place
    interpreter.memory.copy(dest, src, len);
}

/// see https://eips.ethereum.org/EIPS/eip-4844
pub fn blob_hash<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, SPEC::enabled(CANCUN));
    // We add an instruction BLOBHASH (with opcode HASH_OPCODE_BYTE) which reads index from the top of the stack as big-endian uint256,
    // and replaces it on the stack with tx.blob_versioned_hashes[index] if index < len(tx.blob_versioned_hashes),
    // and otherwise with a zeroed bytes32 value. The opcode has a gas cost of HASH_OPCODE_GAS.
    gas!(interpreter, gas::HASH_OPCODE_GAS);
    pop!(interpreter, index);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    if index < host.env().tx.blob_versioned_hashes.len() {
        // Replace the top of the stack with the versioned hash here
        push!(
            interpreter,
            *host.env().tx.blob_versioned_hashes.index(index)
        );
    } else {
        // else write out a zerod out 32 bytes
        let bytes: [u8; 32] = [0; 32];
        push!(interpreter, U256::from_be_bytes(bytes));
    }
}
