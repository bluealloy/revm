use core::{
    cmp::Ordering,
    ops::{Div, Rem},
};
use primitive_types::U256;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
    Plus,
    Minus,
    Zero,
}

const SIGN_BIT_MASK: U256 = U256([
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
    0x7fffffffffffffff,
]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

impl I256 {
    /// Zero value of I256.
    pub fn zero() -> I256 {
        I256(Sign::Zero, U256::zero())
    }
    /// Minimum value of I256.
    pub fn min_value() -> I256 {
        I256(Sign::Minus, (U256::MAX & SIGN_BIT_MASK) + U256::from(1u64))
    }
}

impl Ord for I256 {
    fn cmp(&self, other: &I256) -> Ordering {
        match (self.0, other.0) {
            (Sign::Zero, Sign::Zero) => Ordering::Equal,
            (Sign::Zero, Sign::Plus) => Ordering::Less,
            (Sign::Zero, Sign::Minus) => Ordering::Greater,
            (Sign::Minus, Sign::Zero) => Ordering::Less,
            (Sign::Minus, Sign::Plus) => Ordering::Less,
            (Sign::Minus, Sign::Minus) => self.1.cmp(&other.1).reverse(),
            (Sign::Plus, Sign::Minus) => Ordering::Greater,
            (Sign::Plus, Sign::Zero) => Ordering::Greater,
            (Sign::Plus, Sign::Plus) => self.1.cmp(&other.1),
        }
    }
}

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &I256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for I256 {
    fn default() -> I256 {
        I256::zero()
    }
}

impl From<U256> for I256 {
    fn from(val: U256) -> I256 {
        if val == U256::zero() {
            I256::zero()
        } else if val & SIGN_BIT_MASK == val {
            I256(Sign::Plus, val)
        } else {
            I256(Sign::Minus, !val + U256::from(1u64))
        }
    }
}

impl From<I256> for U256 {
    fn from(value: I256) -> U256 {
        let sign = value.0;
        if sign == Sign::Zero {
            U256::zero()
        } else if sign == Sign::Plus {
            value.1
        } else {
            !value.1 + U256::from(1u64)
        }
    }
}

impl Div for I256 {
    type Output = I256;

    fn div(self, other: I256) -> I256 {
        if other == I256::zero() {
            return I256::zero();
        }

        if self == I256::min_value() && other.1 == U256::from(1u64) {
            return I256::min_value();
        }

        let d = (self.1 / other.1) & SIGN_BIT_MASK;

        if d == U256::zero() {
            return I256::zero();
        }

        match (self.0, other.0) {
            (Sign::Zero, Sign::Plus)
            | (Sign::Plus, Sign::Zero)
            | (Sign::Zero, Sign::Zero)
            | (Sign::Plus, Sign::Plus)
            | (Sign::Minus, Sign::Minus) => I256(Sign::Plus, d),
            (Sign::Zero, Sign::Minus)
            | (Sign::Plus, Sign::Minus)
            | (Sign::Minus, Sign::Zero)
            | (Sign::Minus, Sign::Plus) => I256(Sign::Minus, d),
        }
    }
}

impl Rem for I256 {
    type Output = I256;

    fn rem(self, other: I256) -> I256 {
        let r = (self.1 % other.1) & SIGN_BIT_MASK;

        if r == U256::zero() {
            return I256::zero();
        }

        I256(self.0, r)
    }
}

#[cfg(test)]
mod tests {
    use super::{Sign, I256};
    use core::num::Wrapping;
    use primitive_types::U256;

    #[test]
    fn div_i256() {
        // Sanity checks based on i8. Notice that we need to use `Wrapping` here because
        // Rust will prevent the overflow by default whereas the EVM does not.
        assert_eq!(Wrapping(i8::MIN) / Wrapping(-1), Wrapping(i8::MIN));
        assert_eq!(i8::MAX / -1, -i8::MAX);

        assert_eq!(100i8 / -1, -100i8);
        assert_eq!(100i8 / 2, 50i8);

        // Now the same calculations based on i256
        let one = I256(Sign::Zero, U256::from(1));
        let one_hundred = I256(Sign::Zero, U256::from(100));
        let fifty = I256(Sign::Plus, U256::from(50));
        let two = I256(Sign::Zero, U256::from(2));
        let neg_one_hundred = I256(Sign::Minus, U256::from(100));
        let minus_one = I256(Sign::Minus, U256::from(1));
        let max_value = I256(Sign::Plus, U256::from(2).pow(U256::from(255)) - 1);
        let neg_max_value = I256(Sign::Minus, U256::from(2).pow(U256::from(255)) - 1);

        assert_eq!(I256::min_value() / minus_one, I256::min_value());
        assert_eq!(I256::min_value() / one, I256::min_value());
        assert_eq!(max_value / one, max_value);
        assert_eq!(max_value / minus_one, neg_max_value);

        assert_eq!(one_hundred / minus_one, neg_one_hundred);
        assert_eq!(one_hundred / two, fifty);
    }
}
