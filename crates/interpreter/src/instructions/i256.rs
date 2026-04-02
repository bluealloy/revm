use core::cmp::Ordering;
use primitives::U256;

/// Represents the sign of a 256-bit signed integer value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum Sign {
    // Same as `cmp::Ordering`
    /// Negative value sign
    Minus = -1,
    /// Zero value sign  
    Zero = 0,
    #[allow(dead_code)] // "constructed" with `mem::transmute` in `i256_sign` below
    /// Positive value sign
    Plus = 1,
}

/// The maximum positive value for a 256-bit signed integer.
pub const MAX_POSITIVE_VALUE: U256 = U256::from_limbs([
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
    0x7fffffffffffffff,
]);

/// The minimum negative value for a 256-bit signed integer.
pub const MIN_NEGATIVE_VALUE: U256 = U256::from_limbs([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x8000000000000000,
]);

const FLIPH_BITMASK_U64: u64 = 0x7FFF_FFFF_FFFF_FFFF;

/// Determines the sign of a 256-bit signed integer.
#[inline]
pub fn i256_sign(val: &U256) -> Sign {
    if val.bit(U256::BITS - 1) {
        Sign::Minus
    } else {
        // SAFETY: false == 0 == Zero, true == 1 == Plus
        unsafe { core::mem::transmute::<bool, Sign>(!val.is_zero()) }
    }
}

/// Determines the sign of a 256-bit signed integer and converts it to its absolute value.
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

/// Computes the two's complement of a U256 value in place.
#[inline]
pub fn two_compl_mut(op: &mut U256) {
    *op = two_compl(*op);
}

/// Computes the two's complement of a U256 value.
#[inline]
pub fn two_compl(op: U256) -> U256 {
    op.wrapping_neg()
}

/// Compares two 256-bit signed integers.
#[inline]
pub fn i256_cmp(first: &U256, second: &U256) -> Ordering {
    let first_sign = i256_sign(first);
    let second_sign = i256_sign(second);
    match first_sign.cmp(&second_sign) {
        // Note: Adding `if first_sign != Sign::Zero` to short circuit zero comparisons performs
        // slower on average, as of #582
        Ordering::Equal => first.cmp(second),
        o => o,
    }
}

/// Performs signed division of two 256-bit integers.
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

    // Necessary overflow checks are done above, perform the division
    let mut d = first / second;

    // Set sign bit to zero
    u256_remove_sign(&mut d);

    // Two's complement only if the signs are different
    // Note: This condition has better codegen than an exhaustive match, as of #582
    if (first_sign == Sign::Minus && second_sign != Sign::Minus)
        || (second_sign == Sign::Minus && first_sign != Sign::Minus)
    {
        two_compl(d)
    } else {
        d
    }
}

/// Performs signed modulo of two 256-bit integers.
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

    // Set sign bit to zero
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

    // Signed integer helpers (two's complement, matching EVM semantics).
    const NEG1: U256 = U256::MAX; // -1 in two's complement
    const NEG2: U256 = U256::from_limbs([u64::MAX - 1, u64::MAX, u64::MAX, u64::MAX]);
    const NEG3: U256 = U256::from_limbs([u64::MAX - 2, u64::MAX, u64::MAX, u64::MAX]);
    const NEG100: U256 = U256::from_limbs([u64::MAX - 99, u64::MAX, u64::MAX, u64::MAX]);

    #[test]
    fn div_i256() {
        // Sanity checks based on i8. Notice that we need to use `Wrapping` here because
        // Rust will prevent the overflow by default whereas the EVM does not.
        assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));
        assert_eq!(i8::MAX / -1, -i8::MAX);

        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, NEG1), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, U256::ONE), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(MAX_POSITIVE_VALUE, U256::ONE), MAX_POSITIVE_VALUE);
        assert_eq!(
            i256_div(MAX_POSITIVE_VALUE, NEG1),
            NEG1 * MAX_POSITIVE_VALUE
        );
        assert_eq!(i256_div(U256::from(100u64), NEG1), NEG100);
        assert_eq!(
            i256_div(U256::from(100u64), U256::from(2u64)),
            U256::from(50u64)
        );
    }

    #[test]
    fn test_i256_sign() {
        assert_eq!(i256_sign(&U256::ZERO), Sign::Zero);
        assert_eq!(i256_sign(&U256::ONE), Sign::Plus);
        assert_eq!(i256_sign(&NEG1), Sign::Minus);
        assert_eq!(i256_sign(&MIN_NEGATIVE_VALUE), Sign::Minus);
        assert_eq!(i256_sign(&MAX_POSITIVE_VALUE), Sign::Plus);
    }

    #[test]
    fn test_i256_sign_compl() {
        let mut zero = U256::ZERO;
        let mut positive = U256::ONE;
        let mut negative = NEG1;
        assert_eq!(i256_sign_compl(&mut zero), Sign::Zero);
        assert_eq!(i256_sign_compl(&mut positive), Sign::Plus);
        assert_eq!(i256_sign_compl(&mut negative), Sign::Minus);
    }

    #[test]
    fn test_two_compl() {
        assert_eq!(two_compl(U256::ZERO), U256::ZERO);
        assert_eq!(two_compl(U256::ONE), NEG1);
        assert_eq!(two_compl(NEG1), U256::ONE);
        assert_eq!(two_compl(U256::from(2u64)), NEG2);
        assert_eq!(two_compl(NEG2), U256::from(2u64));
        // Two's complement of the min value is itself.
        assert_eq!(two_compl(MIN_NEGATIVE_VALUE), MIN_NEGATIVE_VALUE);
    }

    #[test]
    fn test_two_compl_mut() {
        let mut value = U256::ONE;
        two_compl_mut(&mut value);
        assert_eq!(value, NEG1);
    }

    #[test]
    fn test_i256_cmp() {
        assert_eq!(i256_cmp(&U256::ONE, &U256::from(2u64)), Ordering::Less);
        assert_eq!(
            i256_cmp(&U256::from(2u64), &U256::from(2u64)),
            Ordering::Equal
        );
        assert_eq!(
            i256_cmp(&U256::from(3u64), &U256::from(2u64)),
            Ordering::Greater
        );
        assert_eq!(i256_cmp(&NEG1, &NEG1), Ordering::Equal);
        assert_eq!(i256_cmp(&NEG1, &NEG2), Ordering::Greater);
        assert_eq!(i256_cmp(&NEG1, &U256::ZERO), Ordering::Less);
        assert_eq!(i256_cmp(&NEG2, &U256::from(2u64)), Ordering::Less);
    }

    #[test]
    fn test_i256_div() {
        assert_eq!(i256_div(U256::ONE, U256::ZERO), U256::ZERO);
        assert_eq!(i256_div(U256::ZERO, U256::ONE), U256::ZERO);
        assert_eq!(i256_div(U256::ZERO, NEG1), U256::ZERO);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, U256::ONE), MIN_NEGATIVE_VALUE);
        assert_eq!(
            i256_div(U256::from(4u64), U256::from(2u64)),
            U256::from(2u64)
        );
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, MIN_NEGATIVE_VALUE), U256::ONE);
        assert_eq!(i256_div(U256::from(2u64), NEG1), NEG2);
        assert_eq!(i256_div(NEG2, NEG1), U256::from(2u64));
    }

    #[test]
    fn test_i256_mod() {
        assert_eq!(i256_mod(U256::ZERO, U256::ONE), U256::ZERO);
        assert_eq!(i256_mod(U256::ONE, U256::ZERO), U256::ZERO);
        assert_eq!(i256_mod(U256::from(4u64), U256::from(2u64)), U256::ZERO);
        assert_eq!(i256_mod(U256::from(3u64), U256::from(2u64)), U256::ONE);
        assert_eq!(i256_mod(MIN_NEGATIVE_VALUE, U256::ONE), U256::ZERO);
        assert_eq!(i256_mod(U256::from(2u64), U256::from(2u64)), U256::ZERO);
        assert_eq!(
            i256_mod(U256::from(2u64), U256::from(3u64)),
            U256::from(2u64)
        );
        assert_eq!(i256_mod(NEG2, U256::from(3u64)), NEG2);
        assert_eq!(i256_mod(U256::from(2u64), NEG3), U256::from(2u64));
        assert_eq!(i256_mod(NEG2, NEG3), NEG2);
    }
}
