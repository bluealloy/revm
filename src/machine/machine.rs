use std::{ops::Range};

use crate::opcode::eval;
use bytes::Bytes;
use primitive_types::U256;

use super::{contract::Contract, memory::Memory, stack::Stack};
use crate::{
    error::{ExitReason, ExitSucceed},
    opcode::{Control, OpCode},
    spec::Spec,
    ExtHandler
};

pub struct Machine {
    /// Contract information and invoking data
    pub contract: Contract,
    /// Program counter.
    program_counter: usize,
    /// Return value.
    pub return_range: Range<U256>,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// Return stuff
    pub status: Result<(), ExitReason>,
    pub return_data_buffer: Bytes,
}

impl Machine {
    pub fn new(contract: Contract) -> Self {
        Self {
            program_counter: 0,
            return_range: Range::default(),
            memory: Memory::new(100),
            stack: Stack::new(),
            status: Ok(()),
            return_data_buffer: Bytes::new(),
            contract,
        }
    }
    pub fn contract(&self) -> &Contract {
        &self.contract
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
    pub fn step<H: ExtHandler, SPEC: Spec>(&mut self, handler: &mut H) -> Result<(), ExitReason> {
        let program_counter = self.program_counter;

        // extract next opcode from code
        let opcode = self
            .contract
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
        handler.trace_opcode(&self.contract, opcode, &self.stack);
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
    pub fn return_value(&self) -> Bytes {
        if self.return_range.start > U256::from(usize::MAX) {
            let mut ret = Vec::new();
            ret.resize(
                (self.return_range.end - self.return_range.start).as_usize(),
                0,
            );
            Bytes::from(ret)
        } else if self.return_range.end > U256::from(usize::MAX) {
            let mut ret = self
                .memory
                .get(
                    self.return_range.start.as_usize(),
                    usize::MAX - self.return_range.start.as_usize(),
                )
                .to_vec();
            while ret.len() < (self.return_range.end - self.return_range.start).as_usize() {
                ret.push(0);
            }
            Bytes::from(ret)
        } else {
            self.memory.get(
                self.return_range.start.as_usize(),
                (self.return_range.end - self.return_range.start).as_usize(),
            )
        }
    }
}
