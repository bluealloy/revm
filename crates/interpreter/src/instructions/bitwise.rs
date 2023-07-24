use super::i256::{i256_cmp, i256_sign, two_compl, Sign};
use super::prelude::*;

#[inline]
const fn btou256(b: bool) -> U256 {
    U256::from_limbs([b as u64, 0, 0, 0])
}

pub(super) fn lt(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = btou256(op1 < *op2);
}

pub(super) fn gt(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = btou256(op1 > *op2);
}

pub(super) fn slt(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = btou256(i256_cmp(op1, *op2) == Ordering::Less);
}

pub(super) fn sgt(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = btou256(i256_cmp(op1, *op2) == Ordering::Greater);
}

pub(super) fn eq(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = btou256(op1 == *op2);
}

pub(super) fn iszero(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = btou256(*op1 == U256::ZERO);
}

pub(super) fn bitand(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 & *op2;
}

pub(super) fn bitor(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 | *op2;
}

pub(super) fn bitxor(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 ^ *op2;
}

pub(super) fn not(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = !*op1;
}

pub(super) fn byte(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let o1 = as_usize_saturated!(op1);
    *op2 = if o1 < 32 {
        // On little endian targets, `Uint` can be interpreted as `&[u8; BYTES]` in LE
        #[cfg(target_endian = "little")]
        {
            // SAFETY: in range 0..32
            U256::from(unsafe { *op2.as_limbs().as_ptr().cast::<u8>().add(31 - o1) })
        }

        #[cfg(target_endian = "big")]
        {
            (*op2 << (8 * o1)) >> (8 * 31)
        }
    } else {
        U256::ZERO
    };
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn shl(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, CONSTANTINOPLE));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 <<= as_usize_saturated!(op1);
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn shr(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, CONSTANTINOPLE));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 >>= as_usize_saturated!(op1);
}

// EIP-145: Bitwise shifting instructions in EVM
pub(super) fn sar(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, CONSTANTINOPLE));
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
