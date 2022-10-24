use crate::{gas, Interpreter, Return, Spec};

use super::i256::{i256_div, i256_mod};
use core::ops::Rem;
use ruint::aliases::U256;
use std::ops::{Div, Mul};

pub fn div(op1: U256, op2: U256) -> U256 {
    if op2 == U256::ZERO {
        U256::ZERO
    } else {
        //op1 / op2
        op1.div(op2)
    }
}

pub fn sdiv(op1: U256, op2: U256) -> U256 {
    i256_div(op1, op2)
}

pub fn rem(op1: U256, op2: U256) -> U256 {
    if op2 == U256::ZERO {
        U256::ZERO
    } else {
        op1.rem(op2)
    }
}

pub fn smod(op1: U256, op2: U256) -> U256 {
    if op2 == U256::ZERO {
        U256::ZERO
    } else {
        i256_mod(op1, op2)
    }
}

pub fn addmod(op1: U256, op2: U256, op3: U256) -> U256 {
    op1.add_mod(op2, op3)
}

pub fn mulmod(op1: U256, op2: U256, op3: U256) -> U256 {
    op1.mul_mod(op2, op3)
}

pub fn exp(op1: U256, op2: U256) -> U256 {
    let mut result = U256::from(1);
    let mut op1 = op1;
    let mut op2 = op2;
    // Exponentiation by squaring
    while op2 > U256::ZERO {
        // Multiply by base
        if op2.as_limbs()[0] & 1 == 1 {
            result = result.mul(op1);
        }

        // Square base
        op1 = op1.mul(op1);
        op2 >>= 1;
    }
    result
}

pub fn eval_exp<SPEC: Spec>(interp: &mut Interpreter) -> Return {
    pop!(interp, op1, op2);
    gas_or_fail!(interp, gas::exp_cost::<SPEC>(op2));
    let ret = exp(op1, op2);
    push!(interp, ret);

    Return::Continue
}

/// In the yellow paper `SIGNEXTEND` is defined to take two inputs, we will call them
/// `x` and `y`, and produce one output. The first `t` bits of the output (numbering from the
/// left, starting from 0) are equal to the `t`-th bit of `y`, where `t` is equal to
/// `256 - 8(x + 1)`. The remaining bits of the output are equal to the corresponding bits of `y`.
/// Note: if `x >= 32` then the output is equal to `y` since `t <= 0`. To efficiently implement
/// this algorithm in the case `x < 32` we do the following. Let `b` be equal to the `t`-th bit
/// of `y` and let `s = 255 - t = 8x + 7` (this is effectively the same index as `t`, but
/// numbering the bits from the right instead of the left). We can create a bit mask which is all
/// zeros up to and including the `t`-th bit, and all ones afterwards by computing the quantity
/// `2^s - 1`. We can use this mask to compute the output depending on the value of `b`.
/// If `b == 1` then the yellow paper says the output should be all ones up to
/// and including the `t`-th bit, followed by the remaining bits of `y`; this is equal to
/// `y | !mask` where `|` is the bitwise `OR` and `!` is bitwise negation. Similarly, if
/// `b == 0` then the yellow paper says the output should start with all zeros, then end with
/// bits from `b`; this is equal to `y & mask` where `&` is bitwise `AND`.

pub fn signextend(op1: U256, op2: U256) -> U256 {
    if op1 < U256::from(32) {
        // `low_u32` works since op1 < 32
        let bit_index = (8 * op1.as_limbs()[0] + 7) as usize;
        let bit = op2.bit(bit_index);
        let mask = (U256::from(1) << bit_index) - U256::from(1);
        if bit {
            op2 | !mask
        } else {
            op2 & mask
        }
    } else {
        op2
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::{signextend, U256};

    /// Test to ensure new (optimized) `signextend` implementation is equivalent to the previous
    /// implementation.
    #[test]
    fn test_signextend() {
        let test_values = vec![
            U256::ZERO,
            U256::from(1),
            U256::from(8),
            U256::from(10),
            U256::from(65),
            U256::from(100),
            U256::from(128),
            U256::from(11) * (U256::from(1) << 65),
            U256::from(7) * (U256::from(1) << 123),
            U256::MAX / U256::from(167),
            U256::MAX,
        ];
        for x in 0..64 {
            for y in test_values.iter() {
                compare_old_signextend(U256::from(x), *y);
            }
        }
    }

    fn compare_old_signextend(x: U256, y: U256) {
        let old = old_signextend(x, y);
        let new = signextend(x, y);

        assert_eq!(old, new);
    }

    fn old_signextend(op1: U256, op2: U256) -> U256 {
        if op1 > U256::from(32) {
            op2
        } else {
            let mut ret = U256::ZERO;
            let len = u128::try_from(op1).unwrap() as usize;
            let t: usize = 8 * (len + 1) - 1;
            let t_bit_mask = U256::from(1) << t;
            let t_value = (op2 & t_bit_mask) >> t;
            for i in 0..256 {
                let bit_mask = U256::from(1) << i;
                let i_value = (op2 & bit_mask) >> i;
                if i <= t {
                    ret = ret.overflowing_add(i_value << i).0;
                } else {
                    ret = ret.overflowing_add(t_value << i).0;
                }
            }
            ret
        }
    }
}
