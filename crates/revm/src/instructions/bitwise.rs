use core::cmp::Ordering;

use super::i256::{i256_cmp, i256_sign, two_compl, Sign};
use ruint::aliases::U256;

pub fn slt(op1: U256, op2: U256) -> U256 {
    if i256_cmp(op1, op2) == Ordering::Less {
        U256::from(1)
    } else {
        U256::ZERO
    }
}

pub fn sgt(op1: U256, op2: U256) -> U256 {
    if i256_cmp(op1, op2) == Ordering::Greater {
        U256::from(1)
    } else {
        U256::ZERO
    }
}

pub fn iszero(op1: U256) -> U256 {
    if op1 == U256::ZERO {
        U256::from(1)
    } else {
        U256::ZERO
    }
}

pub fn not(op1: U256) -> U256 {
    !op1
}

pub fn byte(op1: U256, op2: U256) -> U256 {
    let mut ret = U256::ZERO;

    for i in 0..256 {
        if i < 8 && op1 < U256::from(32) {
            let o = u128::try_from(op1).unwrap() as usize;
            let t = 255 - (7 - i + 8 * o);
            let bit_mask = U256::from(1) << t;
            let value = (op2 & bit_mask) >> t;
            ret = ret.overflowing_add(value << i).0;
        }
    }

    ret
}

pub fn shl(shift: U256, value: U256) -> U256 {
    value << usize::try_from(shift).unwrap_or(256)
}

pub fn shr(shift: U256, value: U256) -> U256 {
    value >> usize::try_from(shift).unwrap_or(256)
}

pub fn sar(shift: U256, mut value: U256) -> U256 {
    let value_sign = i256_sign::<true>(&mut value);

    if value == U256::ZERO || shift >= U256::from(256) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::ZERO,
            // value is <0, pushing -1
            Sign::Minus => two_compl(U256::from(1)),
        }
    } else {
        let shift = usize::try_from(shift).unwrap();

        match value_sign {
            Sign::Plus | Sign::Zero => value >> shift,
            Sign::Minus => {
                let shifted = ((value.overflowing_sub(U256::from(1)).0) >> shift)
                    .overflowing_add(U256::from(1))
                    .0;
                two_compl(shifted)
            }
        }
    }
}
