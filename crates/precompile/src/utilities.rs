use alloc::vec::Vec;
use core::cmp::min;

/// Get an array from the data, if data does not contain `start` to `len` bytes,
/// add right padding with zeroes.
#[inline(always)]
pub fn get_right_padded<const LEN: usize>(data: &[u8], offset: usize) -> [u8; LEN] {
    let mut padded = [0; LEN];
    let start = min(offset, data.len());
    let end = min(start.saturating_add(LEN), data.len());
    padded[..end - start].copy_from_slice(&data[start..end]);
    padded
}

/// Get a vector of the data, if data does not contain the slice of `start` to `len`,
/// right pad missing part with zeroes.
#[inline(always)]
pub fn get_right_padded_vec(data: &[u8], offset: usize, len: usize) -> Vec<u8> {
    let mut padded = vec![0; len];
    let start = min(offset, data.len());
    let end = min(start.saturating_add(len), data.len());
    padded[..end - start].copy_from_slice(&data[start..end]);
    padded
}

/// Left padding until `len`. If data is more then len, truncate the right most bytes.
#[inline(always)]
pub fn left_padding<const LEN: usize>(data: &[u8]) -> [u8; LEN] {
    let mut padded = [0; LEN];
    let end = min(LEN, data.len());
    padded[LEN - end..].copy_from_slice(&data[..end]);
    padded
}

/// Left padding until `len`. If data is more then len, truncate the right most bytes.
#[inline(always)]
pub fn left_padding_vec(data: &[u8], len: usize) -> Vec<u8> {
    let mut padded = vec![0; len];
    let end = min(len, data.len());
    padded[len - end..].copy_from_slice(&data[..end]);
    padded
}
