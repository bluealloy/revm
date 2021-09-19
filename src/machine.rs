use std::{ops::Range, rc::Rc};

use super::opcode::eval;
use primitive_types::U256;

use crate::{
    error::{ExitReason, ExitSucceed},
    memory::Memory,
    opcode::{Control, OpCode},
    stack::Stack,
    Context, Handler,
};

pub struct Machine {
    /// Program data.
    pub data: Rc<Vec<u8>>,
    /// Program code.
    pub code: Rc<Vec<u8>>,
    /// Program counter.
    program_counter: usize,
    /// Return value.
    pub return_range: Range<U256>,
    /// Code validity maps.
    pub valid_jump_addresses: ValidJumpAddress,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// machine execution contexts,
    pub context: Context,
    /// Return stuff
    pub status: Result<(), ExitReason>,
    pub return_data_buffer: Vec<u8>,
}

impl Machine {
    /// Reference of machine stack.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }
    /// Mutable reference of machine stack.
    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
    /// Reference of machine memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    /// Mutable reference of machine memory.
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }
    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

	/// loop steps until we are finished with execution
	pub fn run<H: Handler, const GAS_TRACE: bool>(&mut self, handler: &mut H) -> Result<(),ExitReason> {
		loop {
			self.step::<H,GAS_TRACE>(handler)?
		}
	}

    #[inline]
    /// Step the machine, executing one opcode. It then returns.
    pub fn step<H: Handler, const GAS_TRACE: bool>(
        &mut self,
        handler: &mut H,
    ) -> Result<(), ExitReason> {
        let program_counter = self.program_counter;

        // extract opcode from code and do prevalidating for gas computation
        let opcode = self
            .code
            .get(program_counter)
            .map(|&opcode| OpCode::try_from_u8(opcode))
            .flatten();
        // if there is no opcode in code or OpCode is invalid return error.
        if opcode.is_none() {
            self.status = Err(ExitSucceed::Stopped.into());
            return Err(ExitSucceed::Stopped.into());
        }
        let opcode = opcode.unwrap();

        // call prevalidation to calcuate gas for this opcode
        match handler.pre_validate::<GAS_TRACE>(&self.context, opcode, &self.stack) {
            Ok(()) => (),
            Err(e) => {
                self.status = Err(ExitReason::Error(e));
                return self.status.clone();
            }
        }
        // check status and return if not present
        self.status.as_ref().map_err(|reason| reason.clone())?;

		// evaluate next opcode
        match eval::<H, false, false, false>(self, opcode, program_counter, handler) {
            Control::ContinueOne => {
                self.program_counter = program_counter + 1;
                Ok(())
            }
            Control::Continue(p) => {
                self.program_counter = program_counter + p;
                Ok(())
            }
            Control::Exit(e) => {
                self.status = Err(e.clone());
                Err(e)
            }
            Control::Jump(p) => {
                self.program_counter = p;
                Ok(())
            }
        }
    }
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidJumpAddress(Vec<bool>);

impl ValidJumpAddress {
    /// Create a new valid mapping from given code bytes.
    pub fn new(code: &[u8]) -> Self {
        let mut jumps: Vec<bool> = Vec::with_capacity(code.len());
        jumps.resize(code.len(), false);

        let mut i = 0;
        while i < code.len() {
            let opcode = code[i] as u8;
            if opcode == OpCode::JUMPDEST as u8 {
                jumps[i] = true;
                i += 1;
            } else if let Some(v) = OpCode::is_push(opcode) {
                i += v as usize + 1;
            } else {
                i += 1;
            }
        }

        Self(jumps)
    }

    /// Get the length of the valid mapping. This is the same as the
    /// code bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the valids list is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.0.len() {
            return false;
        }

        self.0[position]
    }
}
