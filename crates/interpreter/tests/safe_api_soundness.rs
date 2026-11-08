//! Regression tests verifying memory safety and sound behavior of safe public APIs
//! in `Stack` and `SharedMemory` under both debug and release compilation profiles.

use revm_interpreter::{
    interpreter::{SharedMemory, Stack},
    primitives::U256,
};

#[test]
fn test_stack_invalid_push() {
    let mut stack = Stack::invalid();
    assert!(stack.push(U256::from(42)));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.pop().unwrap(), U256::from(42));
}

#[test]
fn test_stack_invalid_dup() {
    let mut stack = Stack::invalid();
    assert!(stack.push(U256::from(99)));
    assert!(stack.dup(1));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop().unwrap(), U256::from(99));
    assert_eq!(stack.pop().unwrap(), U256::from(99));
}

#[test]
fn test_stack_invalid_push_slice() {
    let mut stack = Stack::invalid();
    assert!(stack.push_slice(&[1, 2, 3]).is_ok());
    assert_eq!(stack.len(), 1);
}

#[test]
fn test_stack_exchange_overflow() {
    let mut stack = Stack::new();
    assert!(stack.push(U256::ZERO));
    // usize::MAX + 1 wraps to 0 in release without overflow checks
    assert!(!stack.exchange(usize::MAX, 1));
}

#[test]
#[should_panic(expected = "slice OOB")]
fn test_shared_memory_slice_range_oob() {
    let memory = SharedMemory::new();
    let _ = memory.slice_range(0..1);
}

#[test]
#[should_panic(expected = "slice OOB")]
fn test_shared_memory_slice_mut_oob() {
    let mut memory = SharedMemory::new();
    let _ = memory.slice_mut(0, 1);
}

#[test]
#[should_panic(expected = "cannot use invalid SharedMemory")]
fn test_shared_memory_invalid_access() {
    let memory = SharedMemory::invalid();
    let _ = memory.slice_range(0..1);
}

#[test]
fn test_shared_memory_context_memory_valid() {
    let mut memory = SharedMemory::new();
    let child = memory.new_child_context();
    assert!(child.context_memory().is_empty());
}
