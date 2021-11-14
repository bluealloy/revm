use crate::{alloc::vec::Vec, Return};
use bytes::Bytes;
use core::{
    cmp::min,
    ops::{BitAnd, Not},
};

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Debug)]
pub struct Memory {
    data: Vec<u8>,
    limit: usize,
}

impl Memory {
    /// Create a new memory with the given limit.
    pub fn new(limit: usize) -> Self {
        Self {
            data: Vec::with_capacity(4 * 1024), // took it from evmone
            limit,
        }
    }

    pub fn effective_len(&self) -> usize {
        self.data.len()
    }

    /// Memory limit.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Get the length of the current memory range.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return true if current effective memory range is zero.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the full memory.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Resize the memory. asume that we already checked if
    /// we have enought gas to resize this vector and that we made new_size as multiply of 32
    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.data.len() {
            self.data.resize(new_size, 0);
        }
    }

    /// Get memory region at given offset.
    ///
    /// ## Panics
    ///
    /// Value of `size` is considered trusted. If they're too large,
    /// the program can run out of memory, or it can overflow.
    pub fn get(&self, offset: usize, size: usize) -> Bytes {
        let start = min(self.data.len(), offset);
        let end = min(self.data.len(), size + offset);
        let len = end - start;
        let mut ret = Vec::with_capacity(len);
        unsafe {
            ret.set_len(len);
        }
        ret[..len].copy_from_slice(&self.data[start..end]);
        ret.resize(size, 0);

        ret.into()
    }

    /// Set memory region at given offset. The offset and value is considered
    /// untrusted.
    pub fn set(&mut self, offset: usize, value: &[u8], target_size: Option<usize>) -> Return {
        let target_size = target_size.unwrap_or(value.len());
        if target_size == 0 {
            return Return::Continue;
        }

        if offset
            .checked_add(target_size)
            .map(|pos| pos > self.limit)
            .unwrap_or(true)
        {
            return Return::InvalidMemoryRange;
        }

        if self.data.len() < offset + target_size {
            self.data.resize(offset + target_size, 0);
        }

        if target_size > value.len() {
            self.data[offset..((value.len()) + offset)].clone_from_slice(value);
            for index in (value.len())..target_size {
                self.data[offset + index] = 0;
            }
        } else {
            self.data[offset..(target_size + offset)].clone_from_slice(&value[..target_size]);
        }

        Return::Continue
    }

    /// Copy `data` into the memory, of given `len`.
    pub fn copy_large(
        &mut self,
        memory_offset: usize,
        data_offset: usize,
        len: usize,
        data: &[u8],
    ) -> Return {
        if len == 0 {
            return Return::Continue;
        }

        let data = if let Some(end) = data_offset.checked_add(len) {
            if data_offset > data.len() {
                &[]
            } else {
                &data[data_offset..min(end, data.len())]
            }
        } else {
            &[]
        };

        self.set(memory_offset, data, Some(len))
    }
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}

#[cfg(test)]
mod tests {
    use super::next_multiple_of_32;

    #[test]
    fn test_next_multiple_of_32() {
        // next_multiple_of_32 returns x when it is a multiple of 32
        for i in 0..32 {
            let x = i * 32;
            assert_eq!(Some(x), next_multiple_of_32(x));
        }

        // next_multiple_of_32 rounds up to the nearest multiple of 32 when `x % 32 != 0`
        for x in 0..1024 {
            if x % 32 == 0 {
                continue;
            }
            let next_multiple = x + 32 - (x % 32);
            assert_eq!(Some(next_multiple), next_multiple_of_32(x.into()));
        }

        // // next_multiple_of_32 returns None when the next multiple of 32 is too big
        // let last_multiple_of_32 = U256::MAX & !U256::from(31);
        // for i in 0..63 {
        //     let x = U256::MAX - U256::from(i);
        //     if x > last_multiple_of_32 {
        //         assert_eq!(None, next_multiple_of_32(x));
        //     } else {
        //         assert_eq!(Some(last_multiple_of_32), next_multiple_of_32(x));
        //     }
        // }
    }
}
