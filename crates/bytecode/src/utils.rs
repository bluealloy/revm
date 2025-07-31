//! Various utilities for the bytecode

/// Reads a big-endian `i16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_i16(ptr: *const u8) -> i16 {
    read_u16(ptr) as i16
}

/// Reads a big-endian `u16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(unsafe { ptr.cast::<[u8; 2]>().read() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16_big_endian() {
        // These functions should always read big-endian regardless of CPU architecture
        let data = [0x12, 0x34, 0x56, 0x78];
        let ptr = data.as_ptr();

        unsafe {
            // Always reads as big-endian: first byte is MSB
            assert_eq!(read_u16(ptr), 0x1234);
            assert_eq!(read_u16(ptr.add(1)), 0x3456);
            assert_eq!(read_u16(ptr.add(2)), 0x5678);

            // Verify it matches explicit big-endian conversion
            assert_eq!(read_u16(ptr), u16::from_be_bytes([0x12, 0x34]));
        }
    }

    #[test]
    fn test_read_i16_big_endian() {
        let data = [0x12, 0x34, 0xFF, 0xFF, 0x80, 0x00];
        let ptr = data.as_ptr();

        unsafe {
            assert_eq!(read_i16(ptr), 0x1234);
            assert_eq!(read_i16(ptr.add(2)), -1);
            assert_eq!(read_i16(ptr.add(4)), -32768);
        }
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_big_endian_on_little_endian_cpu() {
        // On little-endian CPU, verify our functions still read big-endian
        let data = [0x01, 0x02];
        let ptr = data.as_ptr();

        unsafe {
            let result = read_u16(ptr);
            let native = u16::from_ne_bytes([0x01, 0x02]);

            // Our function returns big-endian
            assert_eq!(result, 0x0102);
            // Native on little-endian would be different
            assert_eq!(native, 0x0201);
            assert_ne!(result, native);
        }
    }

    #[test]
    #[cfg(target_endian = "big")]
    fn test_big_endian_on_big_endian_cpu() {
        // On big-endian CPU, verify our functions match native
        let data = [0x01, 0x02];
        let ptr = data.as_ptr();

        unsafe {
            let result = read_u16(ptr);
            let native = u16::from_ne_bytes([0x01, 0x02]);

            // Both should be big-endian
            assert_eq!(result, 0x0102);
            assert_eq!(native, 0x0102);
            assert_eq!(result, native);
        }
    }

    #[test]
    fn test_read_u16_all_zeros() {
        let data = [0x00, 0x00];
        let ptr = data.as_ptr();

        unsafe {
            assert_eq!(read_u16(ptr), 0x0000);
        }
    }

    #[test]
    fn test_read_u16_all_ones() {
        let data = [0xFF, 0xFF];
        let ptr = data.as_ptr();

        unsafe {
            assert_eq!(read_u16(ptr), 0xFFFF);
        }
    }

    #[test]
    fn test_read_i16_boundary_values() {
        // Test boundary values in big-endian format
        unsafe {
            // i16::MAX = 32767 = 0x7FFF in big-endian
            let max_data = [0x7F, 0xFF];
            assert_eq!(read_i16(max_data.as_ptr()), i16::MAX);

            // i16::MIN = -32768 = 0x8000 in big-endian
            let min_data = [0x80, 0x00];
            assert_eq!(read_i16(min_data.as_ptr()), i16::MIN);

            // -1 = 0xFFFF in big-endian
            let neg_one_data = [0xFF, 0xFF];
            assert_eq!(read_i16(neg_one_data.as_ptr()), -1);

            // 0 = 0x0000
            let zero_data = [0x00, 0x00];
            assert_eq!(read_i16(zero_data.as_ptr()), 0);

            // 1 = 0x0001 in big-endian
            let one_data = [0x00, 0x01];
            assert_eq!(read_i16(one_data.as_ptr()), 1);
        }
    }

    #[test]
    fn test_pointer_arithmetic() {
        // Test reading from different offsets
        let data = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let ptr = data.as_ptr();

        unsafe {
            assert_eq!(read_u16(ptr), 0xAABB);
            assert_eq!(read_u16(ptr.add(1)), 0xBBCC);
            assert_eq!(read_u16(ptr.add(2)), 0xCCDD);
            assert_eq!(read_u16(ptr.add(3)), 0xDDEE);
            assert_eq!(read_u16(ptr.add(4)), 0xEEFF);
        }
    }
}
