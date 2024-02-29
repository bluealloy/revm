use std::borrow::Cow;

/// Right-pads the given slice at `offset` with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline(always)]
pub fn right_pad_with_offset<const LEN: usize>(data: &[u8], offset: usize) -> Cow<'_, [u8; LEN]> {
    right_pad(data.get(offset..).unwrap_or_default())
}

/// Right-pads the given slice at `offset` with zeroes until `len`.
///
/// Returns the first `len` bytes if it does not need padding.
#[inline(always)]
pub fn right_pad_with_offset_vec(data: &[u8], offset: usize, len: usize) -> Cow<'_, [u8]> {
    right_pad_vec(data.get(offset..).unwrap_or_default(), len)
}

/// Right-pads the given slice with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline(always)]
pub fn right_pad<const LEN: usize>(data: &[u8]) -> Cow<'_, [u8; LEN]> {
    if let Some(data) = data.get(..LEN) {
        Cow::Borrowed(data.try_into().unwrap())
    } else {
        let mut padded = [0; LEN];
        padded[..data.len()].copy_from_slice(data);
        Cow::Owned(padded)
    }
}

/// Right-pads the given slice with zeroes until `len`.
///
/// Returns the first `len` bytes if it does not need padding.
#[inline(always)]
pub fn right_pad_vec(data: &[u8], len: usize) -> Cow<'_, [u8]> {
    if let Some(data) = data.get(..len) {
        Cow::Borrowed(data)
    } else {
        let mut padded = vec![0; len];
        padded[..data.len()].copy_from_slice(data);
        Cow::Owned(padded)
    }
}

/// Left-pads the given slice with zeroes until `LEN`.
///
/// Returns the first `LEN` bytes if it does not need padding.
#[inline(always)]
pub fn left_pad<const LEN: usize>(data: &[u8]) -> Cow<'_, [u8; LEN]> {
    if let Some(data) = data.get(..LEN) {
        Cow::Borrowed(data.try_into().unwrap())
    } else {
        let mut padded = [0; LEN];
        padded[LEN - data.len()..].copy_from_slice(data);
        Cow::Owned(padded)
    }
}

/// Left-pads the given slice with zeroes until `len`.
///
/// Returns the first `len` bytes if it does not need padding.
#[inline(always)]
pub fn left_pad_vec(data: &[u8], len: usize) -> Cow<'_, [u8]> {
    if let Some(data) = data.get(..len) {
        Cow::Borrowed(data)
    } else {
        let mut padded = vec![0; len];
        padded[len - data.len()..].copy_from_slice(data);
        Cow::Owned(padded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_with_right_padding() {
        let data = [1, 2, 3, 4];
        let padded = right_pad_with_offset::<8>(&data, 4);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [0, 0, 0, 0, 0, 0, 0, 0]);
        let padded = right_pad_with_offset_vec(&data, 4, 8);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [0, 0, 0, 0, 0, 0, 0, 0]);

        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        let padded = right_pad_with_offset::<8>(&data, 0);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);
        let padded = right_pad_with_offset_vec(&data, 0, 8);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);

        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        let padded = right_pad_with_offset::<8>(&data, 4);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [5, 6, 7, 8, 0, 0, 0, 0]);
        let padded = right_pad_with_offset_vec(&data, 4, 8);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [5, 6, 7, 8, 0, 0, 0, 0]);
    }

    #[test]
    fn right_padding() {
        let data = [1, 2, 3, 4];
        let padded = right_pad::<8>(&data);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 0, 0, 0, 0]);
        let padded = right_pad_vec(&data, 8);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 0, 0, 0, 0]);

        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let padded = right_pad::<8>(&data);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);
        let padded = right_pad_vec(&data, 8);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn left_padding() {
        let data = [1, 2, 3, 4];
        let padded = left_pad::<8>(&data);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [0, 0, 0, 0, 1, 2, 3, 4]);
        let padded = left_pad_vec(&data, 8);
        assert!(matches!(padded, Cow::Owned(_)));
        assert_eq!(padded[..], [0, 0, 0, 0, 1, 2, 3, 4]);

        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let padded = left_pad::<8>(&data);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);
        let padded = left_pad_vec(&data, 8);
        assert!(matches!(padded, Cow::Borrowed(_)));
        assert_eq!(padded[..], [1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
