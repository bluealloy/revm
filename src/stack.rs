use primitive_types::H256;
use super::error::ExitError;

pub const STACK_MAX_LIMIT: usize = 1000000;

pub struct Stack {
    mem: Vec<H256>,
}


impl Stack {
    pub fn new() -> Self {
        Self {
            mem: Vec::new()
        }
    }

    /// Pop a value from the stack. If the stack is already empty, returns the
	/// `StackUnderflow` error.
	pub fn pop(&mut self) -> Result<H256, ExitError> {
		self.mem.pop().ok_or(ExitError::StackUnderflow)
	}

    #[inline]
	/// Push a new value into the stack. If it will exceed the stack limit,
	/// returns `StackOverflow` error and leaves the stack unchanged.
	pub fn push(&mut self, value: H256) -> Result<(), ExitError> {
		if self.mem.len() + 1 > STACK_MAX_LIMIT {
			return Err(ExitError::StackOverflow);
		}
		self.mem.push(value);
		Ok(())
	}

    #[inline]
	/// Peek a value at given index for the stack, where the top of
	/// the stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn peek(&self, no_from_top: usize) -> Result<H256, ExitError> {
		if self.mem.len() > no_from_top {
			Ok(self.mem[self.mem.len() - no_from_top - 1])
		} else {
			Err(ExitError::StackUnderflow)
		}
	}

    #[inline]
	/// Set a value at given index for the stack, where the top of the
	/// stack is at index `0`. If the index is too large,
	/// `StackError::Underflow` is returned.
	pub fn set(&mut self, no_from_top: usize, val: H256) -> Result<(), ExitError> {
		if self.mem.len() > no_from_top {
			let len = self.mem.len();
			self.mem[len - no_from_top - 1] = val;
			Ok(())
		} else {
			Err(ExitError::StackUnderflow)
		}
	}
}