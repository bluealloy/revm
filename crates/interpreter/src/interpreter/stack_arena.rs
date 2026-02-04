//! Arena allocator for EVM stacks.
//!
//! Instead of each frame having its own Vec<U256> stack, we use a single
//! contiguous arena that pre-allocates space for multiple frames. This reduces
//! allocations and improves cache locality.

use primitives::U256;
use std::{sync::Arc, vec::Vec};

/// Maximum number of frames the arena can support.
pub const MAX_ARENA_FRAMES: usize = 16;

/// Stack limit per frame (EVM spec).
pub(super) const STACK_LIMIT: usize = 1024;

/// Arena that holds stack memory for multiple frames.
///
/// The arena is pre-allocated with space for MAX_ARENA_FRAMES frames,
/// each with STACK_LIMIT capacity. Frames access non-overlapping slices
/// of this arena.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StackArena {
    /// Single backing buffer: MAX_ARENA_FRAMES * STACK_LIMIT U256s.
    ///
    /// Layout: [Frame0 (0..1024) | Frame1 (1024..2048) | ... | Frame15 (15360..16384)]
    memory: Arc<Vec<U256>>,
}

impl StackArena {
    /// Creates a new stack arena with pre-allocated memory for all frames.
    pub fn new() -> Self {
        Self {
            memory: Arc::new(vec![U256::ZERO; MAX_ARENA_FRAMES * STACK_LIMIT]),
        }
    }

    /// Returns the memory offset for a given frame index.
    #[inline]
    const fn frame_offset(frame_index: usize) -> usize {
        frame_index * STACK_LIMIT
    }

    /// Returns a mutable pointer to the start of a frame's stack region.
    ///
    /// # Safety
    ///
    /// - `frame_index` must be < MAX_ARENA_FRAMES
    /// - Caller must ensure non-overlapping access (only one frame uses its region at a time)
    #[inline]
    pub unsafe fn frame_ptr(&self, frame_index: usize) -> *mut U256 {
        debug_assert!(frame_index < MAX_ARENA_FRAMES);
        let offset = Self::frame_offset(frame_index);
        self.memory.as_ptr().cast_mut().add(offset)
    }

    /// Returns the capacity available for each frame.
    #[inline]
    pub const fn frame_capacity() -> usize {
        STACK_LIMIT
    }

    /// Returns a clone of the backing buffer Arc.
    /// Use this with [`crate::interpreter::Stack::new_with_arena`] to create
    /// stacks that share this arena's memory.
    #[inline]
    pub fn backing(&self) -> Arc<Vec<U256>> {
        self.memory.clone()
    }
}

impl Default for StackArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_offsets() {
        let _arena = StackArena::new();

        // Check frame offsets
        assert_eq!(StackArena::frame_offset(0), 0);
        assert_eq!(StackArena::frame_offset(1), 1024);
        assert_eq!(StackArena::frame_offset(2), 2048);
        assert_eq!(StackArena::frame_offset(15), 15360);
    }

    #[test]
    fn test_arena_non_overlapping() {
        let arena = StackArena::new();

        unsafe {
            let ptr0 = arena.frame_ptr(0);
            let ptr1 = arena.frame_ptr(1);

            // Write to frame 0
            ptr0.write(U256::from(42));

            // Write to frame 1
            ptr1.write(U256::from(99));

            // Verify no overlap
            assert_eq!(ptr0.read(), U256::from(42));
            assert_eq!(ptr1.read(), U256::from(99));
        }
    }

    #[test]
    fn test_arena_with_stacks() {
        use crate::interpreter::Stack;

        let arena = StackArena::new();
        let backing = arena.backing();

        // Create two stacks backed by the same arena
        let mut s0 = Stack::new_with_arena(backing.clone(), 0);
        let mut s1 = Stack::new_with_arena(backing.clone(), 1);

        // Push to s0
        assert!(s0.push(U256::from(10)));
        assert!(s0.push(U256::from(20)));
        assert!(s0.push(U256::from(30)));

        // Push to s1 (different region)
        assert!(s1.push(U256::from(100)));
        assert!(s1.push(U256::from(200)));

        // Verify they don't interfere
        assert_eq!(s0.len(), 3);
        assert_eq!(s1.len(), 2);
        assert_eq!(s0.data(), [U256::from(10), U256::from(20), U256::from(30)]);
        assert_eq!(s1.data(), [U256::from(100), U256::from(200)]);

        // Pop from s0, verify s1 unaffected
        assert_eq!(s0.pop(), Ok(U256::from(30)));
        assert_eq!(s1.len(), 2);
    }

    #[test]
    fn test_arena_clone() {
        let arena1 = StackArena::new();
        let arena2 = arena1.clone();

        // Should share the same underlying Arc
        assert!(Arc::ptr_eq(&arena1.memory, &arena2.memory));
    }
}
