use core::cmp::Ordering;
use primitive_types::U256;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
    Plus,
    Minus,
    Zero,
}

pub const SIGN_BIT_MASK: U256 = U256([
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
    FLIPH_BITMASK_U64,
]);

pub const MIN_NEGATIVE_VALUE: U256 = U256([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x8000000000000000,
]);

const SIGN_BITMASK_U64: u64 = 0x8000000000000000;
const FLIPH_BITMASK_U64: u64 = 0x7FFFFFFFFFFFFFFF;
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub Sign, pub U256);

#[inline(always)]
pub fn i256_sign<const DO_TWO_COMPL: bool>(val: &mut U256) -> Sign {
    if unsafe { val.0.get_unchecked(3) } & SIGN_BITMASK_U64 == 0 {
        if val.is_zero() {
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
        *val.0.get_unchecked_mut(3) = val.0.get_unchecked(3) & FLIPH_BITMASK_U64;
    }
}

#[inline(always)]
pub fn two_compl_mut(op: &mut U256) {
    *op = two_compl(*op);
}

pub fn two_compl(op: U256) -> U256 {
    !op + U256::one()
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

/*
pub mod inner_zkp_u256 {
    use core::convert::TryFrom;
    use zkp_u256::U256;

    #[inline]
    const fn val_2(lo: u64, hi: u64) -> u128 {
        ((hi as u128) << 64) | (lo as u128)
    }

    #[inline]
    const fn mul_2(a: u64, b: u64) -> u128 {
        (a as u128) * (b as u128)
    }

    /// Compute <hi, lo> / d, returning the quotient and the remainder.
    // TODO: Make sure it uses divq on x86_64.
    // See http://lists.llvm.org/pipermail/llvm-dev/2017-October/118323.html
    // (Note that we require d > hi for this)
    // TODO: If divq is not supported, use a fast software implementation:
    // See https://gmplib.org/~tege/division-paper.pdf
    #[inline]
    pub fn divrem_2by1(lo: u64, hi: u64, d: u64) -> (u64, u64) {
        debug_assert!(d > 0);
        debug_assert!(d > hi);
        let d = u128::from(d);
        let n = val_2(lo, hi);
        let q = n / d;
        let r = n % d;
        debug_assert!(q < val_2(0, 1));
        debug_assert!(
            mul_2(u64::try_from(q).unwrap(), u64::try_from(d).unwrap())
                + val_2(u64::try_from(r).unwrap(), 0)
                == val_2(lo, hi)
        );
        debug_assert!(r < d);
        // There should not be any truncation.
        #[allow(clippy::cast_possible_truncation)]
        (q as u64, r as u64)
    }

    #[inline]
    pub fn div_rem(lhs: &[u64; 4], rhs: &[u64; 4]) -> [u64; 4] {
        let mut numerator = [lhs[0], lhs[1], lhs[2], lhs[3], 0];
        if rhs[3] > 0 {
            // divrem_nby4
            divrem_nbym(&mut numerator, &mut [rhs[0], rhs[1], rhs[2], rhs[3]]);
            [numerator[0], numerator[1], numerator[2], numerator[3]]
        } else if rhs[2] > 0 {
            // divrem_nby3
            divrem_nbym(&mut numerator, &mut [rhs[0], rhs[1], rhs[2]]);
            [numerator[0], numerator[1], numerator[2], 0]
        } else if rhs[1] > 0 {
            // divrem_nby2
            divrem_nbym(&mut numerator, &mut [rhs[0], rhs[1]]);
            [numerator[2], numerator[3], numerator[4], 0]
        } else {
            //if rhs[0] > 0
            divrem_nby1(&mut numerator, rhs[0]);
            [numerator[0], numerator[1], numerator[2], numerator[3]]
        }
    }

    pub(crate) fn divrem_nby1(numerator: &mut [u64], divisor: u64) -> u64 {
        debug_assert!(divisor > 0);
        let mut remainder = 0;
        for i in (0..numerator.len()).rev() {
            let (ni, ri) = divrem_2by1(numerator[i], remainder, divisor);
            numerator[i] = ni;
            remainder = ri;
        }
        remainder
    }

    //      |  n2 n1 n0  |
    //  q = |  --------  |
    //      |_    d1 d0 _|
    fn div_3by2(n: &[u64; 3], d: &[u64; 2]) -> u64 {
        // The highest bit of d needs to be set
        debug_assert!(d[1] >> 63 == 1);

        // The quotient needs to fit u64. For this we need [n2 n1] < [d1 d0]
        debug_assert!(val_2(n[1], n[2]) < val_2(d[0], d[1]));

        if n[2] == d[1] {
            // From [n2 n1] < [d1 d0] and n2 = d1 it follows that n[1] < d[0].
            debug_assert!(n[1] < d[0]);
            // We start by subtracting 2^64 times the divisor, resulting in a
            // negative remainder. Depending on the result, we need to add back
            // in one or two times the divisor to make the remainder positive.
            // (It can not be more since the divisor is > 2^127 and the negated
            // remainder is < 2^128.)
            let neg_remainder = val_2(0, d[0]) - val_2(n[0], n[1]);
            if neg_remainder > val_2(d[0], d[1]) {
                0xffff_ffff_ffff_fffe_u64
            } else {
                0xffff_ffff_ffff_ffff_u64
            }
        } else {
            // Compute quotient and remainder
            let (mut q, mut r) = divrem_2by1(n[1], n[2], d[1]);

            if mul_2(q, d[0]) > val_2(n[0], r) {
                q -= 1;
                r = r.wrapping_add(d[1]);
                let overflow = r < d[1];
                if !overflow && mul_2(q, d[0]) > val_2(n[0], r) {
                    q -= 1;
                    // UNUSED: r += d[1];
                }
            }
            q
        }
    }

    pub(crate) fn divrem_nbym(numerator: &mut [u64], divisor: &mut [u64]) {
        debug_assert!(divisor.len() >= 2);
        debug_assert!(numerator.len() > divisor.len());
        debug_assert!(*divisor.last().unwrap() > 0);
        debug_assert!(*numerator.last().unwrap() == 0);
        // OPT: Once const generics are in, unroll for lengths.
        // OPT: We can use macro generated specializations till then.
        let n = divisor.len();
        let m = numerator.len() - n - 1;

        // D1. Normalize.
        let shift = divisor[n - 1].leading_zeros();
        if shift > 0 {
            numerator[n + m] = numerator[n + m - 1] >> (64 - shift);
            for i in (1..n + m).rev() {
                numerator[i] <<= shift;
                numerator[i] |= numerator[i - 1] >> (64 - shift);
            }
            numerator[0] <<= shift;
            for i in (1..n).rev() {
                divisor[i] <<= shift;
                divisor[i] |= divisor[i - 1] >> (64 - shift);
            }
            divisor[0] <<= shift;
        }

        // D2. Loop over quotient digits
        for j in (0..=m).rev() {
            // D3. Calculate approximate quotient word
            let mut qhat = div_3by2(
                &[numerator[j + n - 2], numerator[j + n - 1], numerator[j + n]],
                &[divisor[n - 2], divisor[n - 1]],
            );

            // D4. Multiply and subtract.
            let mut borrow = 0;
            for i in 0..n {
                let (a, b) = msb(numerator[j + i], qhat, divisor[i], borrow);
                numerator[j + i] = a;
                borrow = b;
            }

            // D5. Test remainder for negative result.
            if numerator[j + n] < borrow {
                // D6. Add back. (happens rarely)
                let mut carry = 0;
                for i in 0..n {
                    let (a, b) = adc(numerator[j + i], divisor[i], carry);
                    numerator[j + i] = a;
                    carry = b;
                }
                qhat -= 1;
                // The updated value of numerator[j + n] would be 0. But since we're going to
                // overwrite it below, we only check that the result would be 0.
                debug_assert_eq!(numerator[j + n].wrapping_sub(borrow).wrapping_add(carry), 0);
            } else {
                // This the would be the updated value when the remainder is non-negative.
                debug_assert_eq!(numerator[j + n].wrapping_sub(borrow), 0);
            }

            // Store remainder in the unused bits of numerator
            numerator[j + n] = qhat;
        }

        // D8. Unnormalize.
        if shift > 0 {
            // Make sure to only normalize the remainder part, the quotient
            // is already normalized.
            for i in 0..(n - 1) {
                numerator[i] >>= shift;
                numerator[i] |= numerator[i + 1] << (64 - shift);
            }
            numerator[n - 1] >>= shift;
        }
    }

    /// Compute a + b + carry, returning the result and the new carry over.
    #[inline(always)]
    pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
        let ret = (a as u128) + (b as u128) + (carry as u128);
        // We want truncation here
        #[allow(clippy::cast_possible_truncation)]
        (ret as u64, (ret >> 64) as u64)
    }

    /// Compute a - (b * c + borrow), returning the result and the new borrow.
    #[inline(always)]
    pub const fn msb(a: u64, b: u64, c: u64, borrow: u64) -> (u64, u64) {
        let ret = (a as u128).wrapping_sub((b as u128) * (c as u128) + (borrow as u128));
        // TODO: Why is this wrapping_sub required?
        // We want truncation here
        #[allow(clippy::cast_possible_truncation)]
        (ret as u64, 0_u64.wrapping_sub((ret >> 64) as u64))
    }
} */

pub mod div_u256 {
    use super::*;

    const WORD_BITS: usize = 64;
    /// Returns a pair `(self / other, self % other)`.
    ///
    /// # Panics
    ///
    /// Panics if `other` is zero.
    #[inline(always)]
    pub fn div_mod(me: U256, other: U256) -> (U256, U256) {
        let my_bits = me.bits();
        let your_bits = other.bits();

        assert!(your_bits != 0, "division by zero");

        // Early return in case we are dividing by a larger number than us
        if my_bits < your_bits {
            return (U256::zero(), me);
        }

        if your_bits <= WORD_BITS {
            return div_mod_small(me, other.low_u64());
        }

        let (n, m) = {
            let my_words = words(my_bits);
            let your_words = words(your_bits);
            (your_words, my_words - your_words)
        };

        div_mod_knuth(me, other, n, m)
    }

    #[inline(always)]
    fn div_mod_small(mut me: U256, other: u64) -> (U256, U256) {
        let mut rem = 0u64;
        for d in me.0.iter_mut().rev() {
            let (q, r) = div_mod_word(rem, *d, other);
            *d = q;
            rem = r;
        }
        (me, rem.into())
    }

    // Whether this fits u64.
    #[inline(always)]
    fn fits_word(me: &U256) -> bool {
        let U256(ref arr) = me;
        for i in arr.iter().take(4).skip(1) {
            if *i != 0 {
                return false;
            }
        }
        true
    }

    // See Knuth, TAOCP, Volume 2, section 4.3.1, Algorithm D.
    #[inline(always)]
    fn div_mod_knuth(me: U256, mut v: U256, n: usize, m: usize) -> (U256, U256) {
        debug_assert!(me.bits() >= v.bits() && !fits_word(&v));
        debug_assert!(n + m <= 4);
        // D1.
        // Make sure 64th bit in v's highest word is set.
        // If we shift both self and v, it won't affect the quotient
        // and the remainder will only need to be shifted back.
        let shift = v.0[n - 1].leading_zeros();
        v <<= shift;
        // u will store the remainder (shifted)
        let mut u = full_shl(me, shift);

        // quotient
        let mut q = U256::zero();
        let v_n_1 = v.0[n - 1];
        let v_n_2 = v.0[n - 2];

        // D2. D7.
        // iterate from m downto 0
        for j in (0..=m).rev() {
            let u_jn = u[j + n];

            // D3.
            // q_hat is our guess for the j-th quotient digit
            // q_hat = min(b - 1, (u_{j+n} * b + u_{j+n-1}) / v_{n-1})
            // b = 1 << WORD_BITS
            // Theorem B: q_hat >= q_j >= q_hat - 2
            let mut q_hat = if u_jn < v_n_1 {
                let (mut q_hat, mut r_hat) = div_mod_word(u_jn, u[j + n - 1], v_n_1);
                // this loop takes at most 2 iterations
                loop {
                    // check if q_hat * v_{n-2} > b * r_hat + u_{j+n-2}
                    let (hi, lo) = split_u128(u128::from(q_hat) * u128::from(v_n_2));
                    if (hi, lo) <= (r_hat, u[j + n - 2]) {
                        break;
                    }
                    // then iterate till it doesn't hold
                    q_hat -= 1;
                    let (new_r_hat, overflow) = r_hat.overflowing_add(v_n_1);
                    r_hat = new_r_hat;
                    // if r_hat overflowed, we're done
                    if overflow {
                        break;
                    }
                }
                q_hat
            } else {
                // here q_hat >= q_j >= q_hat - 1
                u64::max_value()
            };

            // ex. 20:
            // since q_hat * v_{n-2} <= b * r_hat + u_{j+n-2},
            // either q_hat == q_j, or q_hat == q_j + 1

            // D4.
            // let's assume optimistically q_hat == q_j
            // subtract (q_hat * v) from u[j..]
            let q_hat_v = full_mul_u64(v, q_hat);
            // u[j..] -= q_hat_v;
            let c = sub_slice(&mut u[j..], &q_hat_v[..n + 1]);

            // D6.
            // actually, q_hat == q_j + 1 and u[j..] has overflowed
            // highly unlikely ~ (1 / 2^63)
            if c {
                q_hat -= 1;
                // add v to u[j..]
                let c = add_slice(&mut u[j..], &v.0[..n]);
                u[j + n] = u[j + n].wrapping_add(u64::from(c));
            }

            // D5.
            q.0[j] = q_hat;
        }

        // D8.
        let remainder = full_shr(u, shift);

        (q, remainder)
    }

    #[inline(always)]
    fn add_slice(a: &mut [u64], b: &[u64]) -> bool {
        binop_slice(a, b, u64::overflowing_add)
    }

    #[inline(always)]
    fn sub_slice(a: &mut [u64], b: &[u64]) -> bool {
        binop_slice(a, b, u64::overflowing_sub)
    }

    #[inline(always)]
    fn binop_slice(
        a: &mut [u64],
        b: &[u64],
        binop: impl Fn(u64, u64) -> (u64, bool) + Copy,
    ) -> bool {
        let mut c = false;
        a.iter_mut().zip(b.iter()).for_each(|(x, y)| {
            let (res, carry) = binop_carry(*x, *y, c, binop);
            *x = res;
            c = carry;
        });
        c
    }

    #[inline(always)]
    fn binop_carry(
        a: u64,
        b: u64,
        c: bool,
        binop: impl Fn(u64, u64) -> (u64, bool),
    ) -> (u64, bool) {
        let (res1, overflow1) = b.overflowing_add(u64::from(c));
        let (res2, overflow2) = binop(a, res1);
        (res2, overflow1 || overflow2)
    }

    #[inline(always)]
    fn full_shl(me: U256, shift: u32) -> [u64; 4 + 1] {
        debug_assert!(shift < WORD_BITS as u32);
        let mut u = [0u64; 4 + 1];
        let u_lo = me.0[0] << shift;
        let u_hi = me >> (WORD_BITS as u32 - shift);
        u[0] = u_lo;
        u[1..].copy_from_slice(&u_hi.0[..]);
        u
    }

    #[inline(always)]
    fn full_shr(u: [u64; 4 + 1], shift: u32) -> U256 {
        debug_assert!(shift < WORD_BITS as u32);
        let mut res = U256::zero();
        for (i, item) in u.iter().enumerate().take(4) {
            res.0[i] = item >> shift;
        }
        // carry
        if shift > 0 {
            for (i, item) in u.iter().enumerate().skip(1) {
                res.0[i - 1] |= item << (WORD_BITS as u32 - shift);
            }
        }
        res
    }

    #[inline(always)]
    fn full_mul_u64(me: U256, by: u64) -> [u64; 4 + 1] {
        let (prod, carry) = overflowing_mul_u64(me, by);
        let mut res = [0u64; 4 + 1];
        res[..4].copy_from_slice(&prod.0[..]);
        res[4] = carry;
        res
    }

    /// Overflowing multiplication by u64.
    /// Returns the result and carry.
    #[inline(always)]
    fn overflowing_mul_u64(mut me: U256, other: u64) -> (U256, u64) {
        let mut carry = 0u64;

        for d in me.0.iter_mut() {
            let (res, c) = mul_u64(*d, other, carry);
            *d = res;
            carry = c;
        }

        (me, carry)
    }

    #[inline(always)]
    // Returns the least number of words needed to represent the nonzero number
    fn words(bits: usize) -> usize {
        debug_assert!(bits > 0);
        1 + (bits - 1) / WORD_BITS
    }

    #[inline(always)]
    fn mul_u64(a: u64, b: u64, carry: u64) -> (u64, u64) {
        let (hi, lo) = split_u128(a as u128 * b as u128 + carry as u128);
        (lo, hi)
    }

    #[inline(always)]
    const fn split(a: u64) -> (u64, u64) {
        (a >> 32, a & 0xFFFF_FFFF)
    }

    #[inline(always)]
    const fn split_u128(a: u128) -> (u64, u64) {
        ((a >> 64) as _, (a & 0xFFFFFFFFFFFFFFFF) as _)
    }

    #[inline(always)]
    fn div_mod_word(hi: u64, lo: u64, y: u64) -> (u64, u64) {
        debug_assert!(hi < y);
        let x = (u128::from(hi) << 64) + u128::from(lo);
        let d = u128::from(y);
        ((x / d) as u64, (x % d) as u64)
        /*
        // TODO: look at https://gmplib.org/~tege/division-paper.pdf
        const TWO32: u64 = 1 << 32;
        let s = y.leading_zeros();
        let y = y << s;
        let (yn1, yn0) = split(y);
        let un32 = (hi << s) | lo.checked_shr(64 - s).unwrap_or(0);
        let un10 = lo << s;
        let (un1, un0) = split(un10);
        let mut q1 = un32 / yn1;
        let mut rhat = un32 - q1 * yn1;

        while q1 >= TWO32 || q1 * yn0 > TWO32 * rhat + un1 {
            q1 -= 1;
            rhat += yn1;
            if rhat >= TWO32 {
                break;
            }
        }

        let un21 = un32
            .wrapping_mul(TWO32)
            .wrapping_add(un1)
            .wrapping_sub(q1.wrapping_mul(y));
        let mut q0 = un21 / yn1;
        rhat = un21.wrapping_sub(q0.wrapping_mul(yn1));

        while q0 >= TWO32 || q0 * yn0 > TWO32 * rhat + un0 {
            q0 -= 1;
            rhat += yn1;
            if rhat >= TWO32 {
                break;
            }
        }

        let rem = un21
            .wrapping_mul(TWO32)
            .wrapping_add(un0)
            .wrapping_sub(y.wrapping_mul(q0));
        (q1 * TWO32 + q0, rem >> s)
        */
    }
}

#[inline(always)]
pub fn i256_div(mut first: U256, mut second: U256) -> U256 {
    let second_sign = i256_sign::<true>(&mut second);
    if second_sign == Sign::Zero {
        return U256::zero();
    }
    let first_sign = i256_sign::<true>(&mut first);
    if first_sign == Sign::Minus && first == MIN_NEGATIVE_VALUE && second == U256::one() {
        return two_compl(MIN_NEGATIVE_VALUE);
    }

    //use crypto_bigint::U256 as fastU256;
    // let ff = fastU256::from(first.0);
    //let sf = fastU256::from(second.0);

    //let d = ff.checked_div(&sf).unwrap();
    //let mut d: U256 = U256(d.to_uint_array());

    //let mut d = first/second;
    let mut d = div_u256::div_mod(first, second).0;
    //let mut d = U256(inner_zkp_u256::div_rem(&first.0, &second.0));

    //let first = zkp_u256::U256::from_limbs(first.0);
    //let second = zkp_u256::U256::from_limbs(second.0);
    //let mut d = U256(*(first / second).as_limbs());

    u256_remove_sign(&mut d);
    //set sign bit to zero

    if d.is_zero() {
        return U256::zero();
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
        return U256::zero();
    }

    let _ = i256_sign::<true>(&mut second);
    let mut r = first % second;
    u256_remove_sign(&mut r);
    if r.is_zero() {
        return U256::zero();
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
    use core::num::Wrapping;
    use primitive_types::U256;

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
        let max_value = U256::from(2).pow(U256::from(255)) - 1;
        let neg_max_value = U256::from(2).pow(U256::from(255)) - 1;

        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, minus_one), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(MIN_NEGATIVE_VALUE, one), MIN_NEGATIVE_VALUE);
        assert_eq!(i256_div(max_value, one), max_value);
        assert_eq!(i256_div(max_value, minus_one), neg_max_value);
        assert_eq!(i256_div(one_hundred, minus_one), neg_one_hundred);
        assert_eq!(i256_div(one_hundred, two), fifty);
    }

    #[test]
    fn benchmark_div() {
        use super::*;

        let mut f = U256([1, 100, 1, 1]);
        let mut s = U256([0, 0, 10, 0]);

        let time = std::time::Instant::now();
        for i in 0..1_000_000 {
            f.0[1] = i;
            s.0[3] = div_u256::div_mod(f, s).0 .0[3];
        }
        println!("TIME:{:?}", time.elapsed());
    }
}
