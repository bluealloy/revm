//! Experimental storage wrapper, installed only in disposable comparison worktrees.
use core::{cmp::Ordering, hash::{Hash, Hasher}, ops::Deref};
use primitives::Bytes;
use std::vec::Vec;

type Storage = STORAGE_TYPE;

/// Whole account-extension payload; empty inputs are normalized by construction.
#[derive(Clone, Debug)]
pub struct AccountExtension(Storage);

impl AccountExtension {
    /// Creates an allocation-free empty extension.
    pub const fn new() -> Self { Self(EMPTY_STORAGE) }

    /// Copies a payload into its final storage.
    pub fn copy_from_slice(bytes: &[u8]) -> Self { COPY_STORAGE }

    /// Tests the empty representation without accessing an absent allocation.
    #[inline]
    #[allow(clippy::missing_const_for_fn)] // Keep one API across const and non-const backends.
    pub fn is_empty(&self) -> bool { EMPTY_TEST }
}

impl Default for AccountExtension {
    fn default() -> Self { Self::new() }
}

impl AsRef<[u8]> for AccountExtension {
    #[inline]
    fn as_ref(&self) -> &[u8] { SLICE_ACCESS }
}

impl Deref for AccountExtension {
    type Target = [u8];
    fn deref(&self) -> &[u8] { self.as_ref() }
}

impl From<Bytes> for AccountExtension {
    fn from(bytes: Bytes) -> Self { FROM_BYTES }
}

impl From<Vec<u8>> for AccountExtension {
    fn from(bytes: Vec<u8>) -> Self { FROM_VEC }
}

impl PartialEq for AccountExtension {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.is_empty() { return other.is_empty(); }
        if other.is_empty() { return false; }
        self.0 == other.0
    }
}
impl Eq for AccountExtension {}

impl PartialOrd for AccountExtension {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for AccountExtension {
    fn cmp(&self, other: &Self) -> Ordering { self.as_ref().cmp(other.as_ref()) }
}
impl Hash for AccountExtension {
    fn hash<H: Hasher>(&self, h: &mut H) { self.as_ref().hash(h); }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AccountExtension {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() { primitives::hex::serialize(self.as_ref(), s) }
        else { s.serialize_bytes(self.as_ref()) }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AccountExtension {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Match the current wire format. Conversion costs are measured, not hidden.
        Bytes::deserialize(d).map(Self::from)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn byte_semantics_and_wire_format() {
        for payload in [&[][..], &[1][..], &[0, 255][..], &[255; 32][..]] {
            let bytes = Bytes::copy_from_slice(payload);
            let ext = AccountExtension::from(bytes.clone());
            assert_eq!(ext.as_ref(), payload);
            assert_eq!(ext.is_empty(), payload.is_empty());
            assert_eq!(ext, ext.clone());
            let json = serde_json::to_vec(&bytes).unwrap();
            assert_eq!(serde_json::to_vec(&ext).unwrap(), json);
            assert_eq!(serde_json::from_slice::<AccountExtension>(&json).unwrap(), ext);
            let binary = postcard::to_allocvec(&bytes).unwrap();
            assert_eq!(postcard::to_allocvec(&ext).unwrap(), binary);
            assert_eq!(postcard::from_bytes::<AccountExtension>(&binary).unwrap(), ext);
        }
        // ThinArc's header order must not become the payload's lexicographic order.
        assert!(AccountExtension::copy_from_slice(&[0, 255]) < AccountExtension::copy_from_slice(&[1]));
    }
}
