use super::i256::{i256_cmp, i256_sign, two_compl, Sign};
use super::prelude::*;

pub(super) fn lt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 < *op2);
}

pub(super) fn gt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 > *op2);
}

pub(super) fn slt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(op1, *op2) == Ordering::Less);
}

pub(super) fn sgt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(op1, *op2) == Ordering::Greater);
}

pub(super) fn eq(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 == *op2);
}

pub(super) fn iszero(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = U256::from(*op1 == U256::ZERO);
}

pub(super) fn bitand(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 & *op2;
}

pub(super) fn bitor(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 | *op2;
}

pub(super) fn bitxor(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 ^ *op2;
}

pub(super) fn not(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = !*op1;
}

pub(super) fn byte(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let o1 = as_usize_saturated!(op1);
    *op2 = if o1 < 32 {
        // TODO: Remove once this optimization is in `Uint::byte`
        // https://github.com/recmo/uint/pull/273

        // `31 - o1` because `byte` returns LE, while we want BE
        #[cfg(target_endian = "little")]
        let byte = op2.as_le_slice()[31 - o1];
        #[cfg(target_endian = "big")]
        let byte = op2.byte(31 - o1);
        U256::from(byte)
    } else {
        U256::ZERO
    };
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn shl<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 <<= as_usize_saturated!(op1);
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn shr<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 >>= as_usize_saturated!(op1);
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn sar<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let value_sign = i256_sign::<true>(op2);

    *op2 = if *op2 == U256::ZERO || op1 >= U256::from(256) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::ZERO,
            // value is <0, pushing -1
            Sign::Minus => U256::MAX,
        }
    } else {
        const ONE: U256 = U256::from_limbs([1, 0, 0, 0]);
        let shift = usize::try_from(op1).unwrap();
        match value_sign {
            Sign::Plus | Sign::Zero => op2.wrapping_shr(shift),
            Sign::Minus => two_compl(op2.wrapping_sub(ONE).wrapping_shr(shift).wrapping_add(ONE)),
        }
    };
}
