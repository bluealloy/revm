use crate::alloc::vec;
use crate::alloc::vec::Vec;
use crate::U256;
use core::cmp::min;

pub struct SharedMemory {
    data: Vec<u8>,
    pub limit: u64,
    /// Memory sizes checkpoint for each depth
    msizes: Vec<usize>,
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

        let new_msize = base_offset + last_slice_offset;

        self.msizes.push(new_msize);

        let range = new_msize..;

        self.current_slice = &mut self.data[range];
        self.current_len = 0;
    }

    pub fn free_memory(&mut self) {
        if let Some(old_size) = self.msizes.pop() {
            self.resize(old_size);
            let last = *self.msizes.last().unwrap_or(&0);
            self.current_slice = &mut self.data[last..];
            self.current_len = old_size - last;
        } else {
            panic!()
        }
    }

    pub fn new(_gas_limit: u64, _memory_limit: Option<u64>) -> Self {
        // https://2π.com/22/eth-max-mem/
        // let mut upper_bound =
        //     512 * 2_f32.sqrt() as isize * (gas_limit as f64 + 1151_f64).sqrt() as isize - 48 * 512;

        let mut data = vec![0; u32::MAX as usize];
        let msizes = Vec::with_capacity(1024);
        let current_slice: *mut [u8] = &mut data[..];
        SharedMemory {
            data,
            limit: u64::MAX,
            msizes,
            current_slice,
            current_len: 0,
        }
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

    /// Resize the memory. assume that we already checked if
    /// we have enought gas to resize this vector and that we made new_size as multiply of 32
    pub fn resize(&mut self, new_size: usize) {
        if new_size as u64 >= self.limit {
            panic!("Max limit reached")
        }

        let range = if new_size > self.current_len {
            // extend with zeros
            self.current_len..new_size
        } else {
            // truncate
            new_size..self.current_len
        };

        self.get_current_slice_mut()[range]
            .iter_mut()
            .for_each(|byte| *byte = 0);
        self.current_len = new_size;
    }

    /// Get memory region at given offset. Dont check offset and size
    #[inline(always)]
    pub fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.get_current_slice()[offset..offset + size]
    }

    /// Set memory region at given offset
    ///
    /// # Safety
    /// The caller is responsible for checking the offset and value
    #[inline(always)]
    pub unsafe fn set_byte(&mut self, index: usize, byte: u8) {
        self.get_current_slice_mut()[index] = byte;
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

// #[inline]
// pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
//     let r = x.bitand(31).not().wrapping_add(1).bitand(31);
//     x.checked_add(r)
// }

#[cfg(test)]
mod tests {
    use core::{cell::RefCell, u8};

    use alloc::rc::Rc;
    use ruint::aliases::U256;

    use super::SharedMemory;

    // #[test]
    // fn new() {
    //     let gas_limit = 100_000;
    //     let upper_bound = (512
    //         * 2_f32.sqrt() as isize
    //         * (gas_limit as f64 + 1151_f64).sqrt() as isize
    //         - 48 * 512) as usize;
    //     let mem = SharedMemory::new(gas_limit, None);
    //
    //     assert_eq!(mem.data.len(), upper_bound);
    //     assert_eq!(mem.get_current_slice().len(), upper_bound);
    //     assert_eq!(mem.current_len, 0);
    // }

    #[test]
    fn use_new_memory_1() {
        let mut mem = SharedMemory::new(100_000, None);

        mem.use_new_memory();
        assert_eq!(mem.len(), 0);
        assert_eq!(mem.msizes, vec![0]);
        assert_eq!(mem.len(), 0);
        assert_eq!(mem.get_current_slice().len(), mem.data.len());
    }

    #[test]
    fn set() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();

        mem.set(32, &U256::MAX.to_le_bytes::<32>());
        assert_eq!(
            mem.get_current_slice()[32..64].to_vec(),
            U256::MAX.to_le_bytes::<32>().to_vec()
        );
    }

    #[test]
    fn set_data() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();

        mem.set_data(32, 0, 8, &U256::MAX.to_le_bytes::<32>());
        assert_eq!(
            mem.get_current_slice()[32..40].to_vec(),
            [u8::MAX; 8].to_vec()
        );
    }

    #[test]
    fn set_byte() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();
        unsafe {
            mem.set_byte(2, 8);
            mem.set_byte(1, 7);
            mem.set_byte(0, 6);
        };
        assert_eq!(mem.get_current_slice()[0], 6);
        assert_eq!(mem.get_current_slice()[1], 7);
        assert_eq!(mem.get_current_slice()[2], 8);
    }

    #[test]
    fn set_u256() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();

        mem.set_u256(32, U256::MAX);
        assert_ne!(
            mem.get_current_slice()[0..32],
            U256::MAX.to_le_bytes::<32>()
        );
        assert_eq!(
            mem.get_current_slice()[32..64],
            U256::MAX.to_le_bytes::<32>()
        );
    }

    #[test]
    fn resize_1() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();

        mem.set_u256(0, U256::MAX);
        mem.resize(32);
        assert_eq!(
            mem.get_current_slice()[..32],
            U256::ZERO.to_le_bytes::<32>()
        );
        assert_eq!(mem.len(), 32);
    }

    #[test]
    fn use_new_memory_2() {
        let mut mem = SharedMemory::new(100_000, None);
        mem.use_new_memory();

        // mstore(0, U256::MAX) equivalent
        mem.resize(32);
        assert_eq!(mem.len(), 32);
        mem.set_u256(0, U256::MAX);

        // new depth
        mem.use_new_memory();
        assert_eq!(mem.msizes.len(), 2);
        assert_eq!(mem.msizes[1], 32);

        mem.set_u256(0, U256::MAX);
        assert_eq!(
            mem.data.get(32..64).unwrap(),
            mem.get_current_slice()[..32].to_vec(),
        );
        assert_eq!(
            mem.get_current_slice()[..32].to_vec(),
            U256::MAX.to_le_bytes::<32>().to_vec()
        );

        mem.free_memory();
        assert_eq!(
            mem.data.get(..32).unwrap(),
            mem.get_current_slice()[..32].to_vec(),
        );
        assert_eq!(
            mem.data()[32..64].to_vec(),
            U256::ZERO.to_le_bytes::<32>().to_vec()
        );

        assert_eq!(mem.len(), 32);
        assert_eq!(mem.msizes.len(), 1);
        assert_eq!(mem.msizes[0], 0);
        assert_eq!(
            mem.get_current_slice()[..32].to_vec(),
            U256::MAX.to_le_bytes::<32>().to_vec()
        );
    }

    #[test]
    fn use_new_memory_3() {
        let mem = Rc::new(RefCell::new(SharedMemory::new(100_000, None)));
        let mem_1 = Rc::clone(&mem);
        mem_1.borrow_mut().use_new_memory();

        // mstore(0, U256::MAX) equivalent
        mem_1.borrow_mut().resize(32);
        assert_eq!(mem_1.borrow().len(), 32);
        mem_1.borrow_mut().set_u256(0, U256::MAX);

        // new depth
        let mem_2 = Rc::clone(&mem);
        mem_2.borrow_mut().use_new_memory();
        assert_eq!(mem_2.borrow().msizes.len(), 2);
        assert_eq!(mem_2.borrow().msizes[1], 32);

        mem_2.borrow_mut().set_u256(0, U256::MAX);
        assert_eq!(
            mem_2.borrow().data.get(32..64).unwrap(),
            mem_2.borrow().get_current_slice()[..32].to_vec(),
        );
        assert_eq!(
            mem_2.borrow().get_current_slice()[..32].to_vec(),
            U256::MAX.to_le_bytes::<32>().to_vec()
        );

        mem_2.borrow_mut().free_memory();
        drop(mem_2);
        assert_eq!(
            mem_1.borrow().data.get(..32).unwrap(),
            mem_1.borrow().get_current_slice()[..32].to_vec(),
        );
        assert_eq!(
            mem_1.borrow().data()[32..64].to_vec(),
            U256::ZERO.to_le_bytes::<32>().to_vec()
        );

        assert_eq!(mem_1.borrow().len(), 32);
        assert_eq!(mem_1.borrow().msizes.len(), 1);
        assert_eq!(mem_1.borrow().msizes[0], 0);
        assert_eq!(
            mem_1.borrow().get_current_slice()[..32].to_vec(),
            U256::MAX.to_le_bytes::<32>().to_vec()
        );
    }
}
