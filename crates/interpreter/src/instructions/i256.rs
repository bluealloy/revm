use crate::primitives::U256;
use core::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum Sign {
    // same as `cmp::Ordering`
    Minus = -1,
    Zero = 0,
    #[allow(dead_code)] // "constructed" with `mem::transmute` in `i256_sign` below
    Plus = 1,
}

const MIN_NEGATIVE_VALUE: U256 = U256::from_limbs([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x8000000000000000,
]);

const FLIPH_BITMASK_U64: u64 = 0x7FFFFFFFFFFFFFFF;

#[inline]
pub fn i256_sign(val: &U256) -> Sign {
    if val.bit(U256::BITS - 1) {
        Sign::Minus
    } else {
        // SAFETY: false == 0 == Zero, true == 1 == Plus
        unsafe { core::mem::transmute(*val != U256::ZERO) }
    }
}

#[inline]
pub fn i256_sign_compl(val: &mut U256) -> Sign {
    let sign = i256_sign(val);
    if sign == Sign::Minus {
        two_compl_mut(val);
    }
    sign
}

#[inline]
fn u256_remove_sign(val: &mut U256) {
    // SAFETY: U256 does not have any padding bytes
    unsafe {
        val.as_limbs_mut()[3] &= FLIPH_BITMASK_U64;
    }
}

#[inline]
pub fn two_compl_mut(op: &mut U256) {
    *op = two_compl(*op);
}

#[inline]
pub fn two_compl(op: U256) -> U256 {
    op.wrapping_neg()
}

#[inline]
pub fn i256_cmp(first: &U256, second: &U256) -> Ordering {
    let first_sign = i256_sign(first);
    let second_sign = i256_sign(second);
    match first_sign.cmp(&second_sign) {
        // note: adding `if first_sign != Sign::Zero` to short circuit zero comparisons performs
        // slower on average, as of #582
        Ordering::Equal => first.cmp(second),
        o => o,
    }
}

#[inline]
pub fn i256_div(mut first: U256, mut second: U256) -> U256 {
    let second_sign = i256_sign_compl(&mut second);
    if second_sign == Sign::Zero {
        return U256::ZERO;
    }

    let first_sign = i256_sign_compl(&mut first);
    if first == MIN_NEGATIVE_VALUE && second == U256::from(1) {
        return two_compl(MIN_NEGATIVE_VALUE);
    }

    // necessary overflow checks are done above, perform the division
    let mut d = first / second;

    // set sign bit to zero
    u256_remove_sign(&mut d);

    // two's complement only if the signs are different
    // note: this condition has better codegen than an exhaustive match, as of #582
    if (first_sign == Sign::Minus && second_sign != Sign::Minus)
        || (second_sign == Sign::Minus && first_sign != Sign::Minus)
    {
        two_compl(d)
    } else {
        d
    }
}

#[inline]
pub fn i256_mod(mut first: U256, mut second: U256) -> U256 {
    let first_sign = i256_sign_compl(&mut first);
    if first_sign == Sign::Zero {
        return U256::ZERO;
    }

    let second_sign = i256_sign_compl(&mut second);
    if second_sign == Sign::Zero {
        return U256::ZERO;
    }

    let mut r = first % second;

    // set sign bit to zero
    u256_remove_sign(&mut r);

    if first_sign == Sign::Minus {
        two_compl(r)
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::Wrapping;

    const ZERO: U256 = U256::ZERO;
    const ONE: U256 = U256::from_limbs([
        0x0000000000000001,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]);
    const TWO: U256 = U256::from_limbs([
        0x0000000000000002,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]);
    const THREE: U256 = U256::from_limbs([
        0x0000000000000003,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]);
    const FOUR: U256 = U256::from_limbs([
        0x0000000000000004,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]);

    const NEG_ONE: U256 = U256::from_limbs([
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
    ]);
    const NEG_TWO: U256 = U256::from_limbs([
        0xfffffffffffffffe,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
    ]);
    const NEG_THREE: U256 = U256::from_limbs([
        0xfffffffffffffffd,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
    ]);

    const I256_MAX: U256 = U256::from_limbs([
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x7fffffffffffffff,
    ]);

    #[test]
    fn div_i256() {
        // Sanity checks based on i8. Notice that we need to use `Wrapping` here because
        // Rust will prevent the overflow by default whereas the EVM does not.
        assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));
        assert_eq!(i8::MAX / -1, -i8::MAX);

        // Now the same calculations based on i256
        let fifty = U256::from(50);
        let one_hundred = U256::from(100);

        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, NEG_ONE), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, ONE), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(I256_MAX, ONE), I256_MAX);
        assert_eq!(i256_div(I256_MAX, NEG_ONE), NEG_ONE * I256_MAX);
        assert_eq!(i256_div(one_hundred, NEG_ONE), NEG_ONE * one_hundred);
        assert_eq!(i256_div(one_hundred, TWO), fifty);
    }
    #[test]
    fn test_i256_sign() {
        assert_eq!(i256_sign(&ZERO), Sign::Zero);
        assert_eq!(i256_sign(&ONE), Sign::Plus);
        assert_eq!(i256_sign(&MIN_NEGATIVE_VALUE), Sign::Minus);
    }

    #[test]
    fn test_i256_sign_compl() {
        let mut zero = ZERO;
        let mut positive = ONE;
        let mut negative = MIN_NEGATIVE_VALUE;
        assert_eq!(i256_sign_compl(&mut zero), Sign::Zero);
        assert_eq!(i256_sign_compl(&mut positive), Sign::Plus);
        assert_eq!(i256_sign_compl(&mut negative), Sign::Minus);
    }

    #[test]
    fn test_two_compl() {
        assert_eq!(two_compl(ZERO), ZERO);
        assert_eq!(two_compl(ONE), NEG_ONE);
        assert_eq!(two_compl(NEG_ONE), ONE);
        assert_eq!(two_compl(TWO), NEG_TWO);
        assert_eq!(two_compl(NEG_TWO), TWO);

        // Two's complement of the min value is itself.
        assert_eq!(two_compl(MIN_NEGATIVE_VALUE), MIN_NEGATIVE_VALUE);
    }

    #[test]
    fn test_two_compl_mut() {
        let mut value = ONE;
        two_compl_mut(&mut value);
        assert_eq!(value, NEG_ONE);
    }

    #[test]
    fn test_i256_cmp() {
        assert_eq!(i256_cmp(&ONE, &TWO), Ordering::Less);
        assert_eq!(i256_cmp(&TWO, &TWO), Ordering::Equal);
        assert_eq!(i256_cmp(&THREE, &TWO), Ordering::Greater);
        assert_eq!(i256_cmp(&NEG_ONE, &NEG_ONE), Ordering::Equal);
        assert_eq!(i256_cmp(&NEG_ONE, &NEG_TWO), Ordering::Greater);
        assert_eq!(i256_cmp(&NEG_ONE, &ZERO), Ordering::Less);
        assert_eq!(i256_cmp(&NEG_TWO, &TWO), Ordering::Less);
    }

    #[test]
    fn test_i256_div() {
        assert_eq!(i256_div(ONE, ZERO), ZERO);
        assert_eq!(i256_div(ZERO, ONE), ZERO);
        assert_eq!(i256_div(ZERO, NEG_ONE), ZERO);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, ONE), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(FOUR, TWO), TWO);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, MIN_NEGATIVE_VALUE), ONE);
        assert_eq!(i256_div(TWO, NEG_ONE), NEG_TWO);
        assert_eq!(i256_div(NEG_TWO, NEG_ONE), TWO);
    }

    #[test]
    fn test_i256_mod() {
        assert_eq!(i256_mod(ZERO, ONE), ZERO);
        assert_eq!(i256_mod(ONE, ZERO), ZERO);
        assert_eq!(i256_mod(FOUR, TWO), ZERO);
        assert_eq!(i256_mod(THREE, TWO), ONE);
        assert_eq!(i256_mod(MIN_NEGATIVE_VALUE, ONE), ZERO);
        assert_eq!(i256_mod(TWO, TWO), ZERO);
        assert_eq!(i256_mod(TWO, THREE), TWO);
        assert_eq!(i256_mod(NEG_TWO, THREE), NEG_TWO);
        assert_eq!(i256_mod(TWO, NEG_THREE), TWO);
        assert_eq!(i256_mod(NEG_TWO, NEG_THREE), NEG_TWO);
    }
}
