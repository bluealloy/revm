use derive_more::{AsRef, Deref};
use fixed_hash::{construct_fixed_hash, impl_fixed_hash_conversions};

#[cfg(any(test, feature = "arbitrary"))]
use arbitrary::Arbitrary;
#[cfg(any(test, feature = "arbitrary"))]
use proptest_derive::Arbitrary as PropTestArbitrary;

construct_fixed_hash! {
    /// revm 256 bits type.
    #[cfg_attr(any(test, feature = "arbitrary"), derive(Arbitrary, PropTestArbitrary))]
    #[derive(AsRef,Deref)]
    pub struct B256(32);
}

construct_fixed_hash! {
    /// revm 160 bits type.
    #[cfg_attr(any(test, feature = "arbitrary"), derive(Arbitrary, PropTestArbitrary))]
    #[derive(AsRef,Deref)]
    pub struct B160(20);
}

impl From<u64> for B160 {
    fn from(fr: u64) -> Self {
        let x_bytes = fr.to_be_bytes();
        B160([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3],
            x_bytes[4], x_bytes[5], x_bytes[6], x_bytes[7],
        ])
    }
}

impl_fixed_hash_conversions!(B256, B160);

#[cfg(feature = "serde")]
impl serde::Serialize for B256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut slice = [0u8; 2 + 2 * 32];
        serialize::serialize_raw(&mut slice, &self.0, serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for B256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes = [0u8; 32];
        serialize::deserialize_check_len(deserializer, serialize::ExpectedLen::Exact(&mut bytes))?;
        Ok(B256(bytes))
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for B160 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut slice = [0u8; 2 + 2 * 20];
        serialize::serialize_raw(&mut slice, &self.0, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for B160 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut bytes = [0u8; 20];
        serialize::deserialize_check_len(deserializer, serialize::ExpectedLen::Exact(&mut bytes))?;
        Ok(B160(bytes))
    }
}

// code optained from: https://docs.rs/impl-serde/0.4.0/impl_serde/
#[cfg(feature = "serde")]
mod serialize {

    use alloc::{string::String, vec::Vec};
    use core::{fmt, result::Result};
    use serde::{de, Deserializer, Serializer};

    static CHARS: &[u8] = b"0123456789abcdef";

    /// Serialize given bytes to a 0x-prefixed hex string.
    ///
    /// If `skip_leading_zero` initial 0s will not be printed out,
    /// unless the byte string is empty, in which case `0x0` will be returned.
    /// The results are consistent with `serialize_uint` output if the flag is
    /// on and `serialize_raw` if the flag is off.
    pub fn to_hex(bytes: &[u8], skip_leading_zero: bool) -> String {
        let bytes = if skip_leading_zero {
            let non_zero = bytes.iter().take_while(|b| **b == 0).count();
            let bytes = &bytes[non_zero..];
            if bytes.is_empty() {
                return "0x0".into();
            } else {
                bytes
            }
        } else if bytes.is_empty() {
            return "0x".into();
        } else {
            bytes
        };

        let mut slice = vec![0u8; (bytes.len() + 1) * 2];
        to_hex_raw(&mut slice, bytes, skip_leading_zero).into()
    }

    fn to_hex_raw<'a>(v: &'a mut [u8], bytes: &[u8], skip_leading_zero: bool) -> &'a str {
        assert!(v.len() > 1 + bytes.len() * 2);

        v[0] = b'0';
        v[1] = b'x';

        let mut idx = 2;
        let first_nibble = bytes[0] >> 4;
        if first_nibble != 0 || !skip_leading_zero {
            v[idx] = CHARS[first_nibble as usize];
            idx += 1;
        }
        v[idx] = CHARS[(bytes[0] & 0xf) as usize];
        idx += 1;

        for &byte in bytes.iter().skip(1) {
            v[idx] = CHARS[(byte >> 4) as usize];
            v[idx + 1] = CHARS[(byte & 0xf) as usize];
            idx += 2;
        }

        // SAFETY: all characters come either from CHARS or "0x", therefore valid UTF8
        unsafe { core::str::from_utf8_unchecked(&v[0..idx]) }
    }

    /// Decoding bytes from hex string error.
    #[derive(Debug, PartialEq, Eq)]
    pub enum FromHexError {
        /// Invalid (non-hex) character encountered.
        InvalidHex {
            /// The unexpected character.
            character: char,
            /// Index of that occurrence.
            index: usize,
        },
    }

    #[cfg(feature = "std")]
    impl std::error::Error for FromHexError {}

    impl fmt::Display for FromHexError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Self::InvalidHex { character, index } => {
                    write!(fmt, "invalid hex character: {character}, at {index}")
                }
            }
        }
    }

    /// Decode given (both 0x-prefixed or not) hex string into a vector of bytes.
    ///
    /// Returns an error if non-hex characters are present.
    pub fn from_hex(v: &str) -> Result<Vec<u8>, FromHexError> {
        let (v, stripped) = v.strip_prefix("0x").map_or((v, false), |v| (v, true));

        let mut bytes = vec![0u8; (v.len() + 1) / 2];
        from_hex_raw(v, &mut bytes, stripped)?;
        Ok(bytes)
    }

    /// Decode given 0x-prefix-stripped hex string into provided slice.
    /// Used internally by `from_hex` and `deserialize_check_len`.
    ///
    /// The method will panic if `bytes` have incorrect length (make sure to allocate enough beforehand).
    fn from_hex_raw(v: &str, bytes: &mut [u8], stripped: bool) -> Result<usize, FromHexError> {
        let bytes_len = v.len();
        let mut modulus = bytes_len % 2;
        let mut buf = 0;
        let mut pos = 0;
        for (index, byte) in v.bytes().enumerate() {
            buf <<= 4;

            match byte {
                b'A'..=b'F' => buf |= byte - b'A' + 10,
                b'a'..=b'f' => buf |= byte - b'a' + 10,
                b'0'..=b'9' => buf |= byte - b'0',
                b' ' | b'\r' | b'\n' | b'\t' => {
                    buf >>= 4;
                    continue;
                }
                b => {
                    let character = char::from(b);
                    return Err(FromHexError::InvalidHex {
                        character,
                        index: index + if stripped { 2 } else { 0 },
                    });
                }
            }

            modulus += 1;
            if modulus == 2 {
                modulus = 0;
                bytes[pos] = buf;
                pos += 1;
            }
        }

        Ok(pos)
    }

    /// Serializes a slice of bytes.
    pub fn serialize_raw<S>(
        slice: &mut [u8],
        bytes: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if bytes.is_empty() {
            serializer.serialize_str("0x")
        } else {
            serializer.serialize_str(to_hex_raw(slice, bytes, false))
        }
    }

    /// Expected length of bytes vector.
    #[derive(Debug, PartialEq, Eq)]
    pub enum ExpectedLen<'a> {
        /// Exact length in bytes.
        Exact(&'a mut [u8]),
        /// A bytes length between (min; slice.len()].
        Between(usize, &'a mut [u8]),
    }

    impl<'a> fmt::Display for ExpectedLen<'a> {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ExpectedLen::Exact(ref v) => write!(fmt, "length of {}", v.len() * 2),
                ExpectedLen::Between(min, ref v) => {
                    write!(fmt, "length between ({}; {}]", min * 2, v.len() * 2)
                }
            }
        }
    }

    /// Deserialize into vector of bytes with additional size check.
    /// Returns number of bytes written.
    pub fn deserialize_check_len<'a, 'de, D>(
        deserializer: D,
        len: ExpectedLen<'a>,
    ) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<'a> {
            len: ExpectedLen<'a>,
        }

        impl<'a, 'b> de::Visitor<'b> for Visitor<'a> {
            type Value = usize;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a (both 0x-prefixed or not) hex string with {}",
                    self.len
                )
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let (v, stripped) = v.strip_prefix("0x").map_or((v, false), |v| (v, true));

                let len = v.len();
                let is_len_valid = match self.len {
                    ExpectedLen::Exact(ref slice) => len == 2 * slice.len(),
                    ExpectedLen::Between(min, ref slice) => len <= 2 * slice.len() && len > 2 * min,
                };

                if !is_len_valid {
                    return Err(E::invalid_length(v.len(), &self));
                }

                let bytes = match self.len {
                    ExpectedLen::Exact(slice) => slice,
                    ExpectedLen::Between(_, slice) => slice,
                };

                from_hex_raw(v, bytes, stripped).map_err(E::custom)
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(Visitor { len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary() {
        proptest::proptest!(|(_v1: B160, _v2: B256)| {});
    }
}
