use crate::U256;
use core::{
    cmp::min,
    ops::{BitAnd, Not},
};

pub struct SharedMemory {
    pub data: Vec<u8>,
    pub limit: u64,
    /// Memory sizes checkpoint for each depth
    pub msizes: Vec<usize>,
    current_slice: *mut [u8],
    current_len: usize,
}

impl SharedMemory {
    fn get_current_slice(&self) -> &[u8] {
        unsafe { &*self.current_slice }
    }

    fn get_current_slice_mut(&mut self) -> &mut [u8] {
        unsafe { &mut *self.current_slice }
    }

    pub fn use_new_memory(&mut self) {
        let base_offset = self.msizes.last().unwrap_or(&0);
        let last_slice_offset = self.current_len;

        self.msizes.push(base_offset + last_slice_offset);

        let new_msize = self.msizes.last().unwrap();

        let range = if new_msize == &0 {
            0..
        } else {
            new_msize + 1..
        };

        self.current_slice = &mut self.data[range];
        self.current_len = 0;
    }

    pub fn new(gas_limit: u64, memory_limit: Option<u64>) -> Self {
        let upper_bound = ((589_312 + 512 * gas_limit) as f64).sqrt() - 768_f64;
        let limit = if let Some(mlimit) = memory_limit {
            mlimit
        } else {
            upper_bound as u64
        };
        // self.data = Vec::with_capacity(upper_bound as usize);
        let mut data = vec![0; limit as usize];
        let msizes = vec![0_usize; 1024];
        let current_slice: *mut [u8] = &mut data[..];
        SharedMemory {
            data,
            limit,
            msizes,
            current_slice,
            current_len: 0,
        }
    }

    /// Resize the memory. asume that we already checked if
    /// we have enought gas to resize this vector and that we made new_size as multiply of 32
    pub fn resize(&mut self, new_size: usize) {
        if new_size as u64 >= self.limit {
            panic!("Max limit reached")
        }

        let range = if new_size > self.current_len {
            // extend with zeros
            self.current_len + 1..=new_size
        } else {
            // truncate
            new_size..=self.current_len
        };

        self.get_current_slice_mut()[range]
            .iter_mut()
            .for_each(|byte| *byte = 0);
        self.current_len = new_size;
    }

    pub fn effective_len(&self) -> usize {
        self.current_len
    }

    /// Get the length of the current memory range.
    pub fn len(&self) -> usize {
        self.current_len
    }

    /// Return true if current effective memory range is zero.
    pub fn is_empty(&self) -> bool {
        self.current_len == 0
    }

    /// Return the full memory.
    pub fn data(&self) -> &[u8] {
        self.get_current_slice()
    }

    /// Get memory region at given offset. Dont check offset and size
    #[inline(always)]
    pub fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.get_current_slice()[offset..offset + size]
    }

    #[inline(always)]
    pub fn set_u256(&mut self, index: usize, value: U256) {
        self.get_current_slice_mut()[index..index + 32]
            .copy_from_slice(&value.to_be_bytes::<{ U256::BYTES }>());
    }

    /// Set memory region at given offset. The offset and value are already checked
    #[inline(always)]
    pub fn set(&mut self, offset: usize, value: &[u8]) {
        if !value.is_empty() {
            self.get_current_slice_mut()[offset..(value.len() + offset)].copy_from_slice(value);
        }
    }

    /// Set memory from data. Our memory offset+len is expected to be correct but we
    /// are doing bound checks on data/data_offeset/len and zeroing parts that is not copied.
    #[inline(always)]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        if data_offset >= data.len() {
            // nulify all memory slots
            for i in &mut self.get_current_slice_mut()[memory_offset..memory_offset + len] {
                *i = 0;
            }
            return;
        }
        let data_end = min(data_offset + len, data.len());
        let memory_data_end = memory_offset + (data_end - data_offset);
        self.get_current_slice_mut()[memory_offset..memory_data_end]
            .copy_from_slice(&data[data_offset..data_end]);

        // nulify rest of memory slots
        // Safety: Memory is assumed to be valid. And it is commented where that assumption is made
        for i in &mut self.get_current_slice_mut()[memory_data_end..memory_offset + len] {
            *i = 0;
        }
    }

    /// In memory copy given a src, dst, and length
    ///
    /// # Safety
    /// The caller is responsible to check that we resized memory properly.
    #[inline(always)]
    pub fn copy(&mut self, dst: usize, src: usize, length: usize) {
        self.get_current_slice_mut()
            .copy_within(src..src + length, dst);
    }
}

#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}
