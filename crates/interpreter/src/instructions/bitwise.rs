use super::i256::{i256_cmp, i256_sign, two_compl, Sign};
use crate::{
    gas,
    primitives::SpecId::CONSTANTINOPLE,
    primitives::{Spec, U256},
    Host, InstructionResult, Interpreter,
};
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, BitXor};

pub fn lt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = if op1.lt(op2) {
        U256::from(1)
    } else {
        U256::ZERO
    };
}

pub fn gt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = if op1.gt(op2) {
        U256::from(1)
    } else {
        U256::ZERO
    };
}

pub fn slt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = if i256_cmp(op1, *op2) == Ordering::Less {
        U256::from(1)
    } else {
        U256::ZERO
    }
}

pub fn sgt(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = if i256_cmp(op1, *op2) == Ordering::Greater {
        U256::from(1)
    } else {
        U256::ZERO
    };
}

pub fn eq(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = if op1.eq(op2) {
        U256::from(1)
    } else {
        U256::ZERO
    };
}

pub fn iszero(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = if *op1 == U256::ZERO {
        U256::from(1)
    } else {
        U256::ZERO
    };
}
pub fn bitand(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.bitand(*op2);
}
pub fn bitor(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.bitor(*op2);
}
pub fn bitxor(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1.bitxor(*op2);
}

pub fn not(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = !*op1;
}

pub fn byte(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    let mut ret = U256::ZERO;

    for i in 0..256 {
        if i < 8 && op1 < U256::from(32) {
            let o = as_usize_saturated!(op1);
            let t = 255 - (7 - i + 8 * o);
            let bit_mask = U256::from(1) << t;
            let value = (*op2 & bit_mask) >> t;
            ret = ret.overflowing_add(value << i).0;
        }
    }

    *op2 = ret;
}

pub fn shl<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // EIP-145: Bitwise shifting instructions in EVM
    check!(interpreter, SPEC::enabled(CONSTANTINOPLE));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 <<= as_usize_saturated!(op1);
}

pub fn shr<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // EIP-145: Bitwise shifting instructions in EVM
    check!(interpreter, SPEC::enabled(CONSTANTINOPLE));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 >>= as_usize_saturated!(op1);
}

pub fn sar<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // EIP-145: Bitwise shifting instructions in EVM
    check!(interpreter, SPEC::enabled(CONSTANTINOPLE));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let value_sign = i256_sign::<true>(op2);

    *op2 = if *op2 == U256::ZERO || op1 >= U256::from(256) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::ZERO,
            // value is <0, pushing -1
            Sign::Minus => two_compl(U256::from(1)),
        }
    } else {
        let shift = usize::try_from(op1).unwrap();

        match value_sign {
            Sign::Plus | Sign::Zero => *op2 >> shift,
            Sign::Minus => {
                let shifted = ((op2.overflowing_sub(U256::from(1)).0) >> shift)
                    .overflowing_add(U256::from(1))
                    .0;
                two_compl(shifted)
            }
        }
    };
}
