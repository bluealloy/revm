// U256 newtype wrapper backed by alloy_primitives::U256 (ruint).
//
// Currently the only backend is alloy-primitives (ruint::Uint<256, 4>).
// When a second backend is added, exactly one feature flag must be active
// and each backend will live in its own
// `#[cfg(feature = "...")]` block inside `mod backend`.
//
// While only one backend exists there is nothing to gate, so `uint-alloy` does
// not guard any code today. All interop conversions between `U256` and
// `alloy_primitives::U256` / `FixedBytes<32>` are therefore unconditionally
// available.

// ---------------------------------------------------------------------------
// Backend (always alloy for now)
// ---------------------------------------------------------------------------

mod backend {
    /// Inner U256 type (backed by ruint via alloy-primitives).
    pub(super) use alloy_primitives::U256 as Inner;
}

// ---------------------------------------------------------------------------
// U256
// ---------------------------------------------------------------------------

/// 256-bit unsigned integer backed by `alloy_primitives::U256` (`ruint::Uint<256, 4>`).
#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct U256(backend::Inner);

// ---- constants & constructors ----

impl U256 {
    /// The number of bits in this integer type.
    pub const BITS: usize = backend::Inner::BITS;

    /// The additive identity (zero).
    pub const ZERO: Self = Self::from_limbs([0; 4]);

    /// One.
    pub const ONE: Self = Self::from_limbs([1, 0, 0, 0]);

    /// The multiplicative maximum (all bits set).
    pub const MAX: Self = Self::from_limbs([u64::MAX; 4]);

    /// Construct from four `u64` limbs in little-endian word order
    /// (limbs[0] is least significant). This is a `const fn`.
    #[inline]
    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(backend::Inner::from_limbs(limbs))
    }

    /// Reference to internal limbs in little-endian word order.
    #[inline]
    pub const fn as_limbs(&self) -> &[u64; 4] {
        self.0.as_limbs()
    }

    /// Mutable reference to internal limbs.
    ///
    /// # Safety
    ///
    /// The caller must not violate the invariants of the underlying integer
    /// representation (e.g. the most-significant unused bits must remain zero).
    #[inline]
    pub unsafe fn as_limbs_mut(&mut self) -> &mut [u64; 4] {
        // SAFETY: forwarded to the backend; callers must uphold the same contract.
        unsafe { self.0.as_limbs_mut() }
    }
}

impl core::ops::Not for U256 {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl From<backend::Inner> for U256 {
    #[inline]
    fn from(v: backend::Inner) -> Self {
        Self(v)
    }
}

// ---- byte conversion ----

impl U256 {
    /// Construct from a big-endian fixed-size byte array.
    ///
    /// Panics if `BYTES != 32`.
    #[inline]
    pub const fn from_be_bytes<const BYTES: usize>(bytes: [u8; BYTES]) -> Self {
        Self(backend::Inner::from_be_bytes(bytes))
    }

    /// Construct from a big-endian byte slice.
    ///
    /// Panics if the slice length is not exactly 32.
    #[inline]
    pub const fn from_be_slice(bytes: &[u8]) -> Self {
        Self(backend::Inner::from_be_slice(bytes))
    }

    /// Construct from a big-endian byte slice, returning `None` if the length
    /// is not exactly 32.
    #[inline]
    pub const fn try_from_be_slice(bytes: &[u8]) -> Option<Self> {
        match backend::Inner::try_from_be_slice(bytes) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    /// Convert to a big-endian byte array of exactly `BYTES` bytes.
    ///
    /// Panics if `BYTES != 32`.
    #[inline]
    pub const fn to_be_bytes<const BYTES: usize>(&self) -> [u8; BYTES] {
        self.0.to_be_bytes::<BYTES>()
    }

    /// Convert to a big-endian byte vector (32 bytes).
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_be_bytes_vec(&self) -> Vec<u8> {
        self.to_be_bytes::<32>().to_vec()
    }

    /// Convert to a big-endian byte vector with leading zero bytes stripped.
    #[cfg(feature = "std")]
    #[inline]
    pub fn to_be_bytes_trimmed_vec(&self) -> Vec<u8> {
        let bytes = self.to_be_bytes::<32>();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(32);
        bytes[start..].to_vec()
    }
}

// ---- arithmetic methods ----

impl U256 {
    /// Wrapping (modular) addition.
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    /// Wrapping (modular) subtraction.
    #[inline]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    /// Wrapping (modular) multiplication.
    #[inline]
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    /// Wrapping division. Panics if `rhs` is zero.
    #[inline]
    pub fn wrapping_div(self, rhs: Self) -> Self {
        Self(self.0.wrapping_div(rhs.0))
    }

    /// Wrapping remainder. Panics if `rhs` is zero.
    #[inline]
    pub fn wrapping_rem(self, rhs: Self) -> Self {
        Self(self.0.wrapping_rem(rhs.0))
    }

    /// Saturating addition. Saturates at `U256::MAX` on overflow.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction. Saturates at zero on underflow.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Saturating multiplication. Saturates at `U256::MAX` on overflow.
    #[inline]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        Self(self.0.saturating_mul(rhs.0))
    }

    /// Checked addition. Returns `None` on overflow.
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    /// Checked subtraction. Returns `None` on underflow.
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        match self.0.checked_sub(rhs.0) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }

    /// Checked multiplication. Returns `None` on overflow.
    #[inline]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        self.0.checked_mul(rhs.0).map(Self)
    }

    /// Checked division. Returns `None` if `rhs` is zero.
    #[inline]
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        self.0.checked_div(rhs.0).map(Self)
    }

    /// Overflowing addition. Returns the result and a flag indicating overflow.
    #[inline]
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (v, o) = self.0.overflowing_add(rhs.0);
        (Self(v), o)
    }

    /// Overflowing multiplication. Returns the result and a flag indicating overflow.
    #[inline]
    pub fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let (v, o) = self.0.overflowing_mul(rhs.0);
        (Self(v), o)
    }

    /// Exponentiation: `self ^ exp`.
    #[inline]
    pub fn pow(self, exp: Self) -> Self {
        Self(self.0.pow(exp.0))
    }

    /// Modular addition: `(self + rhs) % modulus`.
    #[inline]
    pub fn add_mod(self, rhs: Self, modulus: Self) -> Self {
        Self(self.0.add_mod(rhs.0, modulus.0))
    }

    /// Modular multiplication: `(self * rhs) % modulus`.
    #[inline]
    pub fn mul_mod(self, rhs: Self, modulus: Self) -> Self {
        Self(self.0.mul_mod(rhs.0, modulus.0))
    }

    /// Arithmetic (signed) right shift by `shift` bits.
    /// The sign bit (bit 255) is replicated into vacated positions.
    /// `shift` must be < 256.
    #[inline]
    pub fn arithmetic_shr(self, shift: usize) -> Self {
        Self(self.0.arithmetic_shr(shift))
    }

    /// Wrapping (two's-complement) negation.
    #[inline]
    pub fn wrapping_neg(self) -> Self {
        Self(self.0.wrapping_neg())
    }

    /// Saturating cast to a primitive type.
    /// Returns the maximum value of `T` if `self` exceeds it, otherwise truncates.
    #[inline]
    pub fn saturating_to<T>(&self) -> T
    where
        backend::Inner: ruint::UintTryTo<T>,
    {
        self.0.saturating_to()
    }

    /// Lossless cast to a primitive type. Panics if the value doesn't fit.
    #[inline]
    pub fn to<T>(&self) -> T
    where
        backend::Inner: ruint::UintTryTo<T>,
        T: core::fmt::Debug,
    {
        self.0.to()
    }
}

// ---- bitwise inspection ----

impl U256 {
    /// Returns `true` if the value is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Const-compatible zero check.
    #[inline]
    pub const fn const_is_zero(&self) -> bool {
        self.0.const_is_zero()
    }

    /// Const-compatible equality check.
    #[inline]
    pub const fn const_eq(&self, other: &Self) -> bool {
        self.0.const_eq(&other.0)
    }

    /// Returns the bit at `index` (0 = least significant bit).
    #[inline]
    pub const fn bit(&self, index: usize) -> bool {
        self.0.bit(index)
    }

    /// Returns the byte at `index` (0 = least significant byte, little-endian).
    #[inline]
    pub const fn byte(&self, index: usize) -> u8 {
        self.0.byte(index)
    }

    /// Number of leading zero bits.
    #[inline]
    pub const fn leading_zeros(&self) -> usize {
        self.0.leading_zeros()
    }

    /// Number of bits needed to represent this value (bit length).
    /// Returns 0 for zero.
    #[inline]
    pub const fn bit_len(&self) -> usize {
        self.0.bit_len()
    }
}

// ---------------------------------------------------------------------------
// Operator implementations (manual — both sides unwrapped)
// ---------------------------------------------------------------------------

macro_rules! impl_bin_op {
    ($trait:ident, $method:ident) => {
        impl core::ops::$trait for U256 {
            type Output = Self;
            #[inline]
            fn $method(self, rhs: Self) -> Self {
                Self(core::ops::$trait::$method(self.0, rhs.0))
            }
        }
    };
}

macro_rules! impl_bin_op_assign {
    ($trait:ident, $method:ident) => {
        impl core::ops::$trait for U256 {
            #[inline]
            fn $method(&mut self, rhs: Self) {
                core::ops::$trait::$method(&mut self.0, rhs.0)
            }
        }
    };
}

// Additive / bitwise (both sides same type)
impl_bin_op!(Add, add);
impl_bin_op!(Sub, sub);
impl_bin_op!(Mul, mul);
impl_bin_op!(Div, div);
impl_bin_op!(Rem, rem);
impl_bin_op!(BitAnd, bitand);
impl_bin_op!(BitOr, bitor);
impl_bin_op!(BitXor, bitxor);

impl_bin_op_assign!(AddAssign, add_assign);
impl_bin_op_assign!(SubAssign, sub_assign);
impl_bin_op_assign!(MulAssign, mul_assign);
impl_bin_op_assign!(DivAssign, div_assign);
impl_bin_op_assign!(RemAssign, rem_assign);
impl_bin_op_assign!(BitAndAssign, bitand_assign);
impl_bin_op_assign!(BitOrAssign, bitor_assign);
impl_bin_op_assign!(BitXorAssign, bitxor_assign);

// Shifts: RHS is usize (ruint uses usize for shifts).
impl core::ops::Shl<usize> for U256 {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: usize) -> Self {
        Self(self.0 << rhs)
    }
}
impl core::ops::Shr<usize> for U256 {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: usize) -> Self {
        Self(self.0 >> rhs)
    }
}
impl core::ops::ShlAssign<usize> for U256 {
    #[inline]
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
    }
}
impl core::ops::ShrAssign<usize> for U256 {
    #[inline]
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}

// ---------------------------------------------------------------------------
// From impls for primitive types
// ---------------------------------------------------------------------------

macro_rules! impl_from_primitive {
    ($($t:ty),+) => {
        $(
            impl From<$t> for U256 {
                #[inline]
                fn from(v: $t) -> Self {
                    Self(backend::Inner::from(v))
                }
            }
        )+
    };
}

impl_from_primitive!(u8, u16, u32, u64, u128, usize, bool);

// Signed integer conversions use two's complement representation.
// Negative values are sign-extended to 256 bits (EVM semantics):
// for v = -n, the 256-bit result is MAX - (n-1) = MAX - !(v as $ut).
macro_rules! impl_from_signed {
    ($($t:ty as $ut:ty),+) => {
        $(
            impl From<$t> for U256 {
                #[inline]
                fn from(v: $t) -> Self {
                    if v >= 0 {
                        Self::from(v as $ut)
                    } else {
                        Self::MAX - Self::from(!(v as $ut))
                    }
                }
            }
        )+
    };
}

impl_from_signed!(i8 as u8, i16 as u16, i32 as u32, i64 as u64);

// ---- TryFrom impls (narrowing conversions) ----

impl TryFrom<U256> for u64 {
    type Error = &'static str;
    #[inline]
    fn try_from(v: U256) -> Result<Self, Self::Error> {
        u64::try_from(v.0).map_err(|_| "U256 value too large for u64")
    }
}

impl TryFrom<U256> for u128 {
    type Error = &'static str;
    #[inline]
    fn try_from(v: U256) -> Result<Self, Self::Error> {
        u128::try_from(v.0).map_err(|_| "U256 value too large for u128")
    }
}

impl TryFrom<U256> for usize {
    type Error = &'static str;
    #[inline]
    fn try_from(v: U256) -> Result<Self, Self::Error> {
        usize::try_from(v.0).map_err(|_| "U256 value too large for usize")
    }
}

// ---- Into<alloy_primitives::U256> for interop ----

impl From<U256> for alloy_primitives::U256 {
    #[inline]
    fn from(v: U256) -> Self {
        v.0
    }
}

// ---- FixedBytes<32> / B256 interop ----
// alloy_primitives::B256 = FixedBytes<32>; bytes are big-endian.

impl From<alloy_primitives::FixedBytes<32>> for U256 {
    #[inline]
    fn from(v: alloy_primitives::FixedBytes<32>) -> Self {
        Self::from_be_bytes(v.0)
    }
}

impl From<U256> for alloy_primitives::FixedBytes<32> {
    #[inline]
    fn from(v: U256) -> Self {
        alloy_primitives::FixedBytes(v.to_be_bytes::<32>())
    }
}

// ---- FromStr ----

impl core::str::FromStr for U256 {
    type Err = <backend::Inner as core::str::FromStr>::Err;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        backend::Inner::from_str(s).map(Self)
    }
}

// ---- formatting ----

impl core::fmt::Debug for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl core::fmt::Display for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl core::fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl core::fmt::UpperHex for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.0, f)
    }
}
