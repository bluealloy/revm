use crate::primitives::U256;
use core::cmp::Ordering;

#[cfg(test)]
use proptest_derive::Arbitrary as PropTestArbitrary;

#[cfg(any(test, feature = "arbitrary"))]
use arbitrary::Arbitrary;

#[cfg_attr(test, derive(PropTestArbitrary))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(Arbitrary))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
    Plus,
    Minus,
    Zero,
}

pub const SIGN_BIT_MASK: U256 = U256::from_limbs([
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
    0x7FFFFFFFFFFFFFFF,
]);
pub const MIN_NEGATIVE_VALUE: U256 = U256::from_limbs([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x8000000000000000,
]);

const FLIPH_BITMASK_U64: u64 = 0x7FFFFFFFFFFFFFFF;

#[cfg_attr(test, derive(PropTestArbitrary))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(Arbitrary))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

#[inline(always)]
pub fn i256_sign<const DO_TWO_COMPL: bool>(val: &mut U256) -> Sign {
    if !val.bit(U256::BITS - 1) {
        if *val == U256::ZERO {
            Sign::Zero
        } else {
            Sign::Plus
        }
    } else {
        if DO_TWO_COMPL {
            two_compl_mut(val);
        }
        Sign::Minus
    }
}

#[inline(always)]
fn u256_remove_sign(val: &mut U256) {
    unsafe {
        val.as_limbs_mut()[3] &= FLIPH_BITMASK_U64;
    }
}

#[inline(always)]
pub fn two_compl_mut(op: &mut U256) {
    *op = two_compl(*op);
}

pub fn two_compl(op: U256) -> U256 {
    op.wrapping_neg()
}

#[inline(always)]
pub fn i256_cmp(mut first: U256, mut second: U256) -> Ordering {
    let first_sign = i256_sign::<false>(&mut first);
    let second_sign = i256_sign::<false>(&mut second);
    match (first_sign, second_sign) {
        (Sign::Zero, Sign::Zero) => Ordering::Equal,
        (Sign::Zero, Sign::Plus) => Ordering::Less,
        (Sign::Zero, Sign::Minus) => Ordering::Greater,
        (Sign::Minus, Sign::Zero) => Ordering::Less,
        (Sign::Minus, Sign::Plus) => Ordering::Less,
        (Sign::Minus, Sign::Minus) => first.cmp(&second),
        (Sign::Plus, Sign::Minus) => Ordering::Greater,
        (Sign::Plus, Sign::Zero) => Ordering::Greater,
        (Sign::Plus, Sign::Plus) => first.cmp(&second),
    }
}

#[inline(always)]
pub fn i256_div(mut first: U256, mut second: U256) -> U256 {
    let second_sign = i256_sign::<true>(&mut second);
    if second_sign == Sign::Zero {
        return U256::ZERO;
    }
    let first_sign = i256_sign::<true>(&mut first);
    if first_sign == Sign::Minus && first == MIN_NEGATIVE_VALUE && second == U256::from(1) {
        return two_compl(MIN_NEGATIVE_VALUE);
    }

    //let mut d = first / second;
    let mut d = first.div_rem(second).0;

    u256_remove_sign(&mut d);
    //set sign bit to zero

    if d == U256::ZERO {
        return U256::ZERO;
    }

    match (first_sign, second_sign) {
        (Sign::Zero, Sign::Plus)
        | (Sign::Plus, Sign::Zero)
        | (Sign::Zero, Sign::Zero)
        | (Sign::Plus, Sign::Plus)
        | (Sign::Minus, Sign::Minus) => d,
        (Sign::Zero, Sign::Minus)
        | (Sign::Plus, Sign::Minus)
        | (Sign::Minus, Sign::Zero)
        | (Sign::Minus, Sign::Plus) => two_compl(d),
    }
}

#[inline(always)]
pub fn i256_mod(mut first: U256, mut second: U256) -> U256 {
    let first_sign = i256_sign::<true>(&mut first);
    if first_sign == Sign::Zero {
        return U256::ZERO;
    }

    let _ = i256_sign::<true>(&mut second);
    let mut r = first % second;
    u256_remove_sign(&mut r);
    if r == U256::ZERO {
        return U256::ZERO;
    }
    if first_sign == Sign::Minus {
        two_compl(r)
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::U256;
    use core::num::Wrapping;

    #[test]
    fn div_i256() {
        // Sanity checks based on i8. Notice that we need to use `Wrapping` here because
        // Rust will prevent the overflow by default whereas the EVM does not.
        assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));
        assert_eq!(i8::MAX / -1, -i8::MAX);

        // Now the same calculations based on i256
        let one = U256::from(1);
        let one_hundred = U256::from(100);
        let fifty = U256::from(50);
        let _fifty_sign = Sign::Plus;
        let two = U256::from(2);
        let neg_one_hundred = U256::from(100);
        let _neg_one_hundred_sign = Sign::Minus;
        let minus_one = U256::from(1);
        let max_value = U256::from(2).pow(U256::from(255)) - U256::from(1);
        let neg_max_value = U256::from(2).pow(U256::from(255)) - U256::from(1);

        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, minus_one), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, one), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(max_value, one), max_value);
        assert_eq!(i256_div(max_value, minus_one), neg_max_value);
        assert_eq!(i256_div(one_hundred, minus_one), neg_one_hundred);
        assert_eq!(i256_div(one_hundred, two), fifty);
    }

    #[test]
    fn arbitrary() {
        proptest::proptest!(|(_value: I256)| { })
    }
}
