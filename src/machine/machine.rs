use std::{ops::Range, rc::Rc};

use crate::opcode::eval;
use primitive_types::U256;

use crate::{Context, ExtHandler, Handler, error::{ExitReason, ExitSucceed}, opcode::{Control, OpCode}, spec::Spec};
use super::{memory::Memory,stack::Stack};

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
    pub fn new(code: Vec<u8>, context: Context) -> Self {
        Self {
            /// Program data.
            data: Rc::new(Vec::new()),
            /// Program code.
            code: Rc::new(Vec::new()),
            /// Program counter.
            program_counter: 0,
            /// Return value.
            return_range: Range::default(),
            /// Code validity maps.
            valid_jump_addresses: ValidJumpAddress::new(&code),
            /// Memory.
            memory: Memory::new(100),
            /// Stack.
            stack: Stack::new(),
            /// machine execution contexts,
            context,
            /// Return stuff
            status: Ok(()),
            return_data_buffer: Vec::new(),
        }
    }
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
    pub fn run<H: ExtHandler, SPEC: Spec>(&mut self, handler: &mut H) -> ExitReason {
        loop {
            if let Err(reson) = self.step::<H, SPEC>(handler) {
                return reson;
            }
        }
    }

    #[inline]
    /// Step the machine, executing one opcode. It then returns.
    pub fn step<H: ExtHandler, SPEC: Spec>(
        &mut self,
        handler: &mut H,
    ) -> Result<(), ExitReason> {
        let program_counter = self.program_counter;

        // extract next opcode from code
        let opcode = self
            .code
            .get(program_counter)
            .map(|&opcode| OpCode::try_from_u8(opcode))
            .flatten();
        // if there is no opcode in code or OpCode is invalid, return error.
        if opcode.is_none() {
            self.status = Err(ExitSucceed::Stopped.into());
            return Err(ExitSucceed::Stopped.into());
        }
        let opcode = opcode.unwrap();

        // call prevalidation to calcuate gas consumption for this opcode
        handler.trace_opcode(&self.context, opcode, &self.stack);
        /*
        match handler.opcode(&self.context, opcode, &self.stack) {
            Ok(()) => (),
            Err(e) => {
                self.status = Err(ExitReason::Error(e));
                return self.status.clone();
            }
        }*/
        // check machine status and return if not present
        self.status.as_ref().map_err(|reason| reason.clone())?;

        // evaluate opcode/execute instruction
        match eval::<H, SPEC, false, false, false>(self, opcode, program_counter, handler) {
            Control::Continue => {
                self.program_counter = program_counter + 1;
                Ok(())
            }
            Control::ContinueN(p) => {
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

    /// Copy and get the return value of the machine, if any.
    pub fn return_value(&self) -> Vec<u8> {
        if self.return_range.start > U256::from(usize::MAX) {
            let mut ret = Vec::new();
            ret.resize(
                (self.return_range.end - self.return_range.start).as_usize(),
                0,
            );
            ret
        } else if self.return_range.end > U256::from(usize::MAX) {
            let mut ret = self.memory.get(
                self.return_range.start.as_usize(),
                usize::MAX - self.return_range.start.as_usize(),
            );
            while ret.len() < (self.return_range.end - self.return_range.start).as_usize() {
                ret.push(0);
            }
            ret
        } else {
            self.memory.get(
                self.return_range.start.as_usize(),
                (self.return_range.end - self.return_range.start).as_usize(),
            )
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
