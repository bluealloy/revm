use core::cmp::Ordering;

use super::i256::{i256_cmp, i256_sign, two_compl, Sign};
use primitive_types::U256;

pub fn slt(op1: U256, op2: U256) -> U256 {
    if i256_cmp(op1, op2) == Ordering::Less {
        U256::one()
    } else {
        U256::zero()
    }
}

pub fn sgt(op1: U256, op2: U256) -> U256 {
    if i256_cmp(op1, op2) == Ordering::Greater {
        U256::one()
    } else {
        U256::zero()
    }
}

pub fn iszero(op1: U256) -> U256 {
    if op1.is_zero() {
        U256::one()
    } else {
        U256::zero()
    }
}

pub fn not(op1: U256) -> U256 {
    !op1
}

pub fn byte(op1: U256, op2: U256) -> U256 {
    let mut ret = U256::zero();

    for i in 0..256 {
        if i < 8 && op1 < 32.into() {
            let o: usize = op1.as_usize();
            let t = 255 - (7 - i + 8 * o);
            let bit_mask = U256::one() << t;
            let value = (op2 & bit_mask) >> t;
            ret = ret.overflowing_add(value << i).0;
        }
    }

    ret
}

pub fn shl(shift: U256, value: U256) -> U256 {
    if value.is_zero() || shift >= U256::from(256) {
        U256::zero()
    } else {
        let shift: u64 = shift.as_u64();
        value << shift as usize
    }
}

pub fn shr(shift: U256, value: U256) -> U256 {
    if value.is_zero() || shift >= U256::from(256) {
        U256::zero()
    } else {
        let shift: u64 = shift.as_u64();
        value >> shift as usize
    }
}

pub fn sar(shift: U256, mut value: U256) -> U256 {
    let value_sign = i256_sign::<true>(&mut value);

    if value.is_zero() || shift >= U256::from(256) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::zero(),
            // value is <0, pushing -1
            Sign::Minus => two_compl(U256::one()),
        }
    } else {
        let shift: u64 = shift.as_u64();

        match value_sign {
            Sign::Plus | Sign::Zero => value >> shift as usize,
            Sign::Minus => {
                let shifted = ((value.overflowing_sub(U256::one()).0) >> shift as usize)
                    .overflowing_add(U256::one())
                    .0;
                two_compl(shifted)
            }
        }
    }
}
