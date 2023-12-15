//! Handler contains all the logic that is specific to the Evm.
//! It is used to define different behavior depending on the chain (Optimism,Mainnet) or
//! hardfork (Berlin, London, ..).

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
    primitives::{db::Database, Spec, SpecId},
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
    /// Main handles.
    pub main: MainHandler<'a, EXT, DB>,
    /// Frame handles.
    pub frame: FrameHandler<'a, EXT, DB>,
}

impl<'a, H: Host, EXT: 'a, DB: Database + 'a> Handler<'a, H, EXT, DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec + 'static>() -> Self {
        Self {
            spec_id: SPEC::SPEC_ID,
            instruction_table: Some(InstructionTables::Plain(make_instruction_table::<H, SPEC>())),
            registers: Vec::new(),
            validation: ValidationHandler::new::<SPEC>(),
            main: MainHandler::new::<SPEC>(),
            frame: FrameHandler::new::<SPEC>(),
        }
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

    /// Returns reference to main handler.
    pub fn main(&self) -> &MainHandler<'a, EXT, DB> {
        &self.main
    }

    /// Returns reference to frame handler.
    pub fn frame(&self) -> &FrameHandler<'a, EXT, DB> {
        &self.frame
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
        use crate::primitives::specification::*;
        // We are transitioning from var to generic spec.
        match spec_id {
            SpecId::FRONTIER | SpecId::FRONTIER_THAWING => {
                self.create_handle_generic::<FrontierSpec>()
            }
            SpecId::HOMESTEAD | SpecId::DAO_FORK => self.create_handle_generic::<HomesteadSpec>(),
            SpecId::TANGERINE => self.create_handle_generic::<TangerineSpec>(),
            SpecId::SPURIOUS_DRAGON => self.create_handle_generic::<SpuriousDragonSpec>(),
            SpecId::BYZANTIUM => self.create_handle_generic::<ByzantiumSpec>(),
            SpecId::PETERSBURG | SpecId::CONSTANTINOPLE => {
                self.create_handle_generic::<PetersburgSpec>()
            }
            SpecId::ISTANBUL | SpecId::MUIR_GLACIER => self.create_handle_generic::<IstanbulSpec>(),
            SpecId::BERLIN => self.create_handle_generic::<BerlinSpec>(),
            SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => {
                self.create_handle_generic::<LondonSpec>()
            }
            SpecId::MERGE => self.create_handle_generic::<MergeSpec>(),
            SpecId::SHANGHAI => self.create_handle_generic::<ShanghaiSpec>(),
            SpecId::CANCUN => self.create_handle_generic::<CancunSpec>(),
            SpecId::LATEST => self.create_handle_generic::<LatestSpec>(),
            #[cfg(feature = "optimism")]
            SpecId::BEDROCK => self.create_handle_generic::<BedrockSpec>(),
            #[cfg(feature = "optimism")]
            SpecId::REGOLITH => self.create_handle_generic::<RegolithSpec>(),
        }
    }
}
