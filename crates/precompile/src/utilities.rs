use core::cmp::min;

use alloc::vec::Vec;

#[inline(always)]
pub fn get_right_padded<const S: usize>(data: &[u8], start: usize) -> [u8; S] {
    let mut padded = [0; S];
    let start = min(start, data.len());
    let end = min(start + S, data.len());
    padded[..end - start].copy_from_slice(&data[start..end]);
    padded
}

pub fn get_right_padded_vec(data: &[u8], start: usize, len: usize) -> Vec<u8> {
    let mut padded = vec![0; len];
    let start = min(start, data.len());
    let end = min(start + len, data.len());
    padded[..end - start].copy_from_slice(&data[start..end]);
    padded
}

/// Left padding until `len` if data is more then len, truncate the right most bytes.
pub fn left_padding(data: Vec<u8>, len: usize) -> Vec<u8> {
    let mut padded = vec![0; len];
    let start = min(len, data.len());
    padded[len - start..].copy_from_slice(&data[..start]);
    padded
}