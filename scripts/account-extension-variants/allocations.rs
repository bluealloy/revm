//! Allocation calls, independent of Criterion timing and input-buffer setup.
EXTENSION_IMPORT
use std::{alloc::{GlobalAlloc, Layout, System}, hint::black_box, sync::atomic::{AtomicUsize, Ordering}};

struct CountAlloc;
static CALLS: AtomicUsize = AtomicUsize::new(0);
// All operations delegate to System with the original pointer and layout.
unsafe impl GlobalAlloc for CountAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: CountAlloc = CountAlloc;

fn measure(name: &str, make: impl FnOnce() -> Extension) {
    let before = CALLS.load(Ordering::Relaxed);
    let value = black_box(make());
    let constructed = CALLS.load(Ordering::Relaxed);
    let first = black_box(black_box(&value).clone());
    let cloned = CALLS.load(Ordering::Relaxed);
    let second = black_box(black_box(&value).clone());
    let end = CALLS.load(Ordering::Relaxed);
    println!("{name},{},{},{}", constructed-before, cloned-constructed, end-cloned);
    drop((value, first, second));
}

fn main() {
    eprintln!("AccountInfo={} Option<AccountInfo>={} Cow<AccountInfo>={} AccountInfoLoad={}",
        size_of::<revm::state::AccountInfo>(),
        size_of::<Option<revm::state::AccountInfo>>(),
        size_of::<std::borrow::Cow<'static, revm::state::AccountInfo>>(),
        size_of::<revm::context_interface::journaled_state::AccountInfoLoad<'static>>());
    println!("case,construct,first_clone,second_clone");
    for n in [0, 32] {
        let payload = black_box(vec![42u8; n]);
        measure(&format!("slice/{n}"), || Extension::copy_from_slice(&payload));
        let payload = black_box(vec![42u8; n]);
        measure(&format!("vec/{n}"), || Extension::from(payload));
    }
}
