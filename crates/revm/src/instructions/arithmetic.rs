use crate::{gas, Interpreter, Return, Spec};

use super::i256::{i256_div, i256_mod};
use core::{convert::TryInto, ops::Rem};
use primitive_types::{U256, U512};

pub fn div(op1: U256, op2: U256) -> U256 {
    if op2.is_zero() {
        U256::zero()
    } else {
        //op1 / op2
        super::i256::div_u256::div_mod(op1, op2).0
    }
}

pub fn sdiv(op1: U256, op2: U256) -> U256 {
    i256_div(op1, op2)
}

pub fn rem(op1: U256, op2: U256) -> U256 {
    if op2.is_zero() {
        U256::zero()
    } else {
        op1.rem(op2)
    }
}

pub fn smod(op1: U256, op2: U256) -> U256 {
    if op2.is_zero() {
        U256::zero()
    } else {
        i256_mod(op1, op2)
    }
}

pub fn addmod(op1: U256, op2: U256, op3: U256) -> U256 {
    if op3.is_zero() {
        U256::zero()
    } else {
        let op1: U512 = op1.into();
        let op2: U512 = op2.into();
        let op3: U512 = op3.into();
        let v = (op1 + op2) % op3;
        v.try_into()
            .expect("op3 is less than U256::MAX, thus it never overflows; qed")
    }
}

pub fn mulmod(op1: U256, op2: U256, op3: U256) -> U256 {
    if op3.is_zero() {
        U256::zero()
    } else {
        let op1: U512 = op1.into();
        let op2: U512 = op2.into();
        let op3: U512 = op3.into();
        let v = (op1 * op2) % op3;
        v.try_into()
            .expect("op3 is less than U256::MAX, thus it never overflows; qed")
    }
}

pub fn exp(op1: U256, op2: U256) -> U256 {
    let mut op1 = op1;
    let mut op2 = op2;
    let mut r: U256 = 1.into();

    while op2 != 0.into() {
        if op2 & 1.into() != 0.into() {
            r = r.overflowing_mul(op1).0;
        }
        op2 >>= 1;
        op1 = op1.overflowing_mul(op1).0;
    }
    r
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
        let bit_index = (8 * op1.low_u32() + 7) as usize;
        let bit = op2.bit(bit_index);
        let mask = (U256::one() << bit_index) - U256::one();
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
            U256::zero(),
            U256::one(),
            U256::from(8),
            U256::from(10),
            U256::from(65),
            U256::from(100),
            U256::from(128),
            U256::from(11) * (U256::one() << 65),
            U256::from(7) * (U256::one() << 123),
            U256::MAX / 167,
            U256::MAX,
        ];
        for x in 0..64 {
            for y in test_values.iter() {
                compare_old_signextend(x.into(), *y);
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
            let mut ret = U256::zero();
            let len: usize = op1.as_usize();
            let t: usize = 8 * (len + 1) - 1;
            let t_bit_mask = U256::one() << t;
            let t_value = (op2 & t_bit_mask) >> t;
            for i in 0..256 {
                let bit_mask = U256::one() << i;
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
