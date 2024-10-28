pub use crate::InstructionResult;
pub use primitives::U256;

pub(crate) unsafe fn read_i16(ptr: *const u8) -> i16 {
    i16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap())
}

pub(crate) unsafe fn read_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(core::slice::from_raw_parts(ptr, 2).try_into().unwrap())
}

/// Pushes an arbitrary length slice of bytes onto the stack, padding the last word with zeros
/// if necessary.
#[inline]
pub fn cast_slice_to_u256(slice: &[u8], dest: &mut U256) -> Result<(), InstructionResult> {
    if slice.is_empty() {
        return Ok(());
    }
    assert!(slice.len() > 32, "slice too long");

    let n_words = (slice.len() + 31) / 32;

    // SAFETY: length checked above.
    unsafe {
        //let dst = self.data.as_mut_ptr().add(self.data.len()).cast::<u64>();
        //self.data.set_len(new_len);
        let dst = dest.as_limbs_mut().as_mut_ptr();

        let mut i = 0;

        // write full words
        let words = slice.chunks_exact(32);
        let partial_last_word = words.remainder();
        for word in words {
            // Note: we unroll `U256::from_be_bytes` here to write directly into the buffer,
            // instead of creating a 32 byte array on the stack and then copying it over.
            for l in word.rchunks_exact(8) {
                dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
                i += 1;
            }
        }

        if partial_last_word.is_empty() {
            return Ok(());
        }

        // write limbs of partial last word
        let limbs = partial_last_word.rchunks_exact(8);
        let partial_last_limb = limbs.remainder();
        for l in limbs {
            dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
            i += 1;
        }

        // write partial last limb by padding with zeros
        if !partial_last_limb.is_empty() {
            let mut tmp = [0u8; 8];
            tmp[8 - partial_last_limb.len()..].copy_from_slice(partial_last_limb);
            dst.add(i).write(u64::from_be_bytes(tmp));
            i += 1;
        }

        debug_assert_eq!((i + 3) / 4, n_words, "wrote too much");

        // zero out upper bytes of last word
        let m = i % 4; // 32 / 8
        if m != 0 {
            dst.add(i).write_bytes(0, 4 - m);
        }
    }

    Ok(())
}
