//! Shared chain-specific account payloads.

use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Deref,
};
use primitives::Bytes;
use std::vec::Vec;
use triomphe::ThinArc;

/// An immutable account payload with a one-pointer inline representation.
///
/// Empty payloads allocate nothing. Nonempty payloads store their length and bytes
/// in one reference-counted allocation; cloning shares that allocation.
#[derive(Clone, Debug, Default)]
pub struct AccountExtension(Option<ThinArc<(), u8>>);

impl AccountExtension {
    /// Creates an empty payload without allocating.
    pub const fn new() -> Self {
        Self(None)
    }

    /// Copies bytes into a single shared allocation.
    pub fn copy_from_slice(bytes: &[u8]) -> Self {
        Self((!bytes.is_empty()).then(|| ThinArc::from_header_and_slice((), bytes)))
    }

    /// Allocates a zero-filled payload and initializes it in place.
    ///
    /// Encoders can write directly into the final allocation, avoiding an intermediate Vec.
    pub fn new_with(len: usize, write: impl FnOnce(&mut [u8])) -> Self {
        if len == 0 {
            write(&mut []);
            return Self::new();
        }
        let mut arc = ThinArc::from_header_and_iter((), core::iter::repeat_n(0, len));
        arc.with_arc_mut(|arc| {
            let unique = triomphe::Arc::get_mut(arc).expect("new allocation is unique");
            write(unique.slice_mut());
        });
        Self(Some(arc))
    }

    /// Returns whether the payload is empty.
    pub const fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl AsRef<[u8]> for AccountExtension {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().map_or(&[], |arc| &arc.slice)
    }
}

impl Deref for AccountExtension {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl From<Bytes> for AccountExtension {
    fn from(bytes: Bytes) -> Self {
        Self::copy_from_slice(&bytes)
    }
}

impl From<Vec<u8>> for AccountExtension {
    fn from(bytes: Vec<u8>) -> Self {
        Self::copy_from_slice(&bytes)
    }
}

impl PartialEq for AccountExtension {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for AccountExtension {}

impl PartialOrd for AccountExtension {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccountExtension {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order payload bytes, not ThinArc's length header.
        self.as_ref().cmp(other.as_ref())
    }
}

impl Hash for AccountExtension {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AccountExtension {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            primitives::hex::serialize(self.as_ref(), serializer)
        } else {
            serializer.serialize_bytes(self.as_ref())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AccountExtension {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Bytes::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_payload_and_in_place_encoding() {
        assert_eq!(size_of::<AccountExtension>(), size_of::<usize>());
        let payload = AccountExtension::new_with(32, |out| out.fill(42));
        let cloned = payload.clone();
        assert_eq!(payload.as_ref(), &[42; 32]);
        assert_eq!(payload.as_ptr(), cloned.as_ptr());
        assert_eq!(payload, AccountExtension::copy_from_slice(&[42; 32]));
        assert!(AccountExtension::new_with(0, |out| assert!(out.is_empty())).is_empty());
        assert!(
            AccountExtension::copy_from_slice(&[0, 255]) < AccountExtension::copy_from_slice(&[1])
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn byte_wire_format() {
        for payload in [&[][..], &[42; 32][..]] {
            let bytes = Bytes::copy_from_slice(payload);
            let extension = AccountExtension::from(bytes.clone());
            let json = serde_json::to_vec(&bytes).unwrap();
            assert_eq!(serde_json::to_vec(&extension).unwrap(), json);
            assert_eq!(
                serde_json::from_slice::<AccountExtension>(&json).unwrap(),
                extension
            );
            let binary = postcard::to_allocvec(&bytes).unwrap();
            assert_eq!(postcard::to_allocvec(&extension).unwrap(), binary);
            assert_eq!(
                postcard::from_bytes::<AccountExtension>(&binary).unwrap(),
                extension
            );
        }
    }
}
