//! Utility functions for hints.
//! Used from Hashbrown <https://github.com/rust-lang/hashbrown/blob/0622304393c802aef285257e4864147cc2ac7374/src/util.rs#L12>.

// FIXME: Replace with `core::hint::{likely, unlikely}` once they are stable.
// pub use core::intrinsics::{likely, unlikely};

/// Cold path function.
#[inline(always)]
#[cold]
pub fn cold_path() {}

#[inline(always)]
fn cold_if(b: bool) {
    if b {
        cold_path();
    }
}

/// Returns `b` but mark `false` path as cold
#[inline(always)]
pub fn likely(b: bool) -> bool {
    cold_if(!b);
    b
}

/// Returns `b` but mark `true` path as cold
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    cold_if(b);
    b
}
