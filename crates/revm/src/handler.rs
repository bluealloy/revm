// Modules.
mod handle_types;
pub mod mainnet;
pub mod register;

// Exports.
pub use handle_types::*;

// Includes.
use crate::{
    interpreter::{
        opcode::{make_instruction_table, InstructionTables},
        Host,
    },
    primitives::{db::Database, spec_to_generic, Spec, SpecId},
};
use alloc::vec::Vec;
use register::{EvmHandler, HandleRegisters};

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<'a, H: Host + 'a, EXT, DB: Database> {
    /// Specification ID.
    pub spec_id: SpecId,
    /// Instruction table type.
    pub instruction_table: Option<InstructionTables<'a, H>>,
    /// Registers that will be called on initialization.
    pub registers: Vec<HandleRegisters<'a, EXT, DB>>,
    /// Validity handles.
    pub validation: ValidationHandler<'a, EXT, DB>,
    /// Pre execution handle
    pub pre_execution: PreExecutionHandler<'a, EXT, DB>,
    /// post Execution handle
    pub post_execution: PostExecutionHandler<'a, EXT, DB>,
    /// Execution loop that handles frames.
    pub execution_loop: ExecutionLoopHandler<'a, EXT, DB>,
}

impl<'a, H: Host, EXT: 'a, DB: Database + 'a> Handler<'a, H, EXT, DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec + 'static>() -> Self {
        Self {
            spec_id: SPEC::SPEC_ID,
            instruction_table: Some(InstructionTables::Plain(make_instruction_table::<H, SPEC>())),
            registers: Vec::new(),
            validation: ValidationHandler::new::<SPEC>(),
            pre_execution: PreExecutionHandler::new::<SPEC>(),
            post_execution: PostExecutionHandler::new::<SPEC>(),
            execution_loop: ExecutionLoopHandler::new::<SPEC>(),
        }
    }

    /// Creates handler with variable spec id, inside it will call `mainnet::<SPEC>` for
    /// appropriate spec.
    pub fn mainnet_with_spec(spec_id: SpecId) -> Self {
        spec_to_generic!(spec_id, Self::mainnet::<SPEC>())
    }

    /// Specification ID.
    pub fn spec_id(&self) -> SpecId {
        self.spec_id
    }

    /// Take instruction table.
    pub fn take_instruction_table(&mut self) -> Option<InstructionTables<'a, H>> {
        self.instruction_table.take()
    }

    /// Set instruction table.
    pub fn set_instruction_table(&mut self, table: InstructionTables<'a, H>) {
        self.instruction_table = Some(table);
    }

    /// Returns reference to pre execution handler.
    pub fn pre_execution(&self) -> &PreExecutionHandler<'a, EXT, DB> {
        &self.pre_execution
    }

    /// Returns reference to pre execution handler.
    pub fn post_execution(&self) -> &PostExecutionHandler<'a, EXT, DB> {
        &self.post_execution
    }

    /// Returns reference to frame handler.
    pub fn execution_loop(&self) -> &ExecutionLoopHandler<'a, EXT, DB> {
        &self.execution_loop
    }

    /// Returns reference to validation handler.
    pub fn validation(&self) -> &ValidationHandler<'a, EXT, DB> {
        &self.validation
    }
}

impl<'a, EXT: 'a, DB: Database + 'a> EvmHandler<'a, EXT, DB> {
    /// Append handle register.
    pub fn append_handle_register(&mut self, register: HandleRegisters<'a, EXT, DB>) {
        register.register(self);
        self.registers.push(register);
    }

    /// Creates the Handler with Generic Spec.
    pub fn create_handle_generic<SPEC: Spec + 'static>(&mut self) -> EvmHandler<'a, EXT, DB> {
        let registers = core::mem::take(&mut self.registers);
        let mut base_handler = Handler::mainnet::<SPEC>();
        // apply all registers to default handeler and raw mainnet instruction table.
        for register in registers {
            base_handler.append_handle_register(register)
        }
        base_handler
    }

    /// Creates the Handler with variable SpecId, inside it will call function with Generic Spec.
    pub fn change_spec_id(mut self, spec_id: SpecId) -> EvmHandler<'a, EXT, DB> {
        if self.spec_id == spec_id {
            return self;
        }

        let registers = core::mem::take(&mut self.registers);
        let mut handler = Handler::mainnet_with_spec(spec_id);
        // apply all registers to default handeler and raw mainnet instruction table.
        for register in registers {
            handler.append_handle_register(register)
        }
        handler
    }
}
