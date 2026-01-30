//! Optimized PUSH immediate data loading.
//!
//! This module provides specialized loading functions for PUSH instructions,
//! inspired by evmone's approach of using type-specific reads for different sizes.
//!
//! # Safety
//!
//! These functions use unchecked pointer reads and require that the bytecode
//! is padded with at least 33 bytes (STOP + 32 zeros) after the last instruction
//! to ensure PUSH32 at any position can safely read without bounds checking.

use primitives::U256;

/// Load N bytes from a pointer as a big-endian U256.
///
/// This function uses specialized loading for different sizes to avoid
/// unnecessary memory operations and byte shuffling.
///
/// # Safety
///
/// Caller must ensure `ptr` has at least `N` readable bytes.
/// This is guaranteed if bytecode is padded with at least 33 bytes.
#[inline(always)]
pub const unsafe fn load_push_data<const N: usize>(ptr: *const u8) -> U256 {
    debug_assert!(N <= 32, "PUSH data cannot exceed 32 bytes");

    match N {
        0 => U256::ZERO,
        1 => U256::from_limbs([*ptr as u64, 0, 0, 0]),
        2 => {
            let val = u16::from_be_bytes(core::ptr::read(ptr as *const [u8; 2]));
            U256::from_limbs([val as u64, 0, 0, 0])
        }
        3 => {
            let mut bytes = [0u8; 4];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(1), 3);
            let val = u32::from_be_bytes(bytes);
            U256::from_limbs([val as u64, 0, 0, 0])
        }
        4 => {
            let val = u32::from_be_bytes(core::ptr::read(ptr as *const [u8; 4]));
            U256::from_limbs([val as u64, 0, 0, 0])
        }
        5 => {
            let mut bytes = [0u8; 8];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(3), 5);
            let val = u64::from_be_bytes(bytes);
            U256::from_limbs([val, 0, 0, 0])
        }
        6 => {
            let mut bytes = [0u8; 8];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(2), 6);
            let val = u64::from_be_bytes(bytes);
            U256::from_limbs([val, 0, 0, 0])
        }
        7 => {
            let mut bytes = [0u8; 8];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(1), 7);
            let val = u64::from_be_bytes(bytes);
            U256::from_limbs([val, 0, 0, 0])
        }
        8 => {
            let val = u64::from_be_bytes(core::ptr::read(ptr as *const [u8; 8]));
            U256::from_limbs([val, 0, 0, 0])
        }
        9 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(7), 9);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        10 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(6), 10);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        11 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(5), 11);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        12 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(4), 12);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        13 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(3), 13);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        14 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(2), 14);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        15 => {
            let mut bytes = [0u8; 16];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(1), 15);
            let val = u128::from_be_bytes(bytes);
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        16 => {
            let val = u128::from_be_bytes(core::ptr::read(ptr as *const [u8; 16]));
            U256::from_limbs([val as u64, (val >> 64) as u64, 0, 0])
        }
        // For 17-32 bytes, we read into a 32-byte buffer
        _ => {
            let mut bytes = [0u8; 32];
            core::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr().add(32 - N), N);
            U256::from_be_bytes(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_push_data_1() {
        let data = [0x42u8];
        let result = unsafe { load_push_data::<1>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x42u64));
    }

    #[test]
    fn test_load_push_data_2() {
        let data = [0x12, 0x34];
        let result = unsafe { load_push_data::<2>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x1234u64));
    }

    #[test]
    fn test_load_push_data_3() {
        let data = [0x12, 0x34, 0x56];
        let result = unsafe { load_push_data::<3>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x123456u64));
    }

    #[test]
    fn test_load_push_data_4() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let result = unsafe { load_push_data::<4>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x12345678u64));
    }

    #[test]
    fn test_load_push_data_8() {
        let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let result = unsafe { load_push_data::<8>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x123456789ABCDEF0u64));
    }

    #[test]
    fn test_load_push_data_16() {
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let result = unsafe { load_push_data::<16>(data.as_ptr()) };
        let expected = U256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
            0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_push_data_32() {
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F, 0x20,
        ];
        let result = unsafe { load_push_data::<32>(data.as_ptr()) };
        let expected = U256::from_be_bytes(data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_push_data_0() {
        let data = [];
        let result = unsafe { load_push_data::<0>(data.as_ptr()) };
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn test_load_push_data_5() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let result = unsafe { load_push_data::<5>(data.as_ptr()) };
        assert_eq!(result, U256::from(0x0102030405u64));
    }

    #[test]
    fn test_load_push_data_20() {
        let data = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14,
        ];
        let result = unsafe { load_push_data::<20>(data.as_ptr()) };
        let mut expected_bytes = [0u8; 32];
        expected_bytes[12..].copy_from_slice(&data);
        let expected = U256::from_be_bytes(expected_bytes);
        assert_eq!(result, expected);
    }
}
