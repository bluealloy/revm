use crate::{
    handler::{ExecutionHandler, PostExecutionHandler, PreExecutionHandler, ValidationHandler},
    EvmHandler,
};
use database_interface::Database;
use interpreter::opcode::InstructionTables;
use specification::spec_to_generic;
use std::fmt::Debug;
use std::vec::Vec;
use wiring::{EthereumWiring, EvmWiring as PrimitiveEvmWiring};

pub trait EvmWiring: PrimitiveEvmWiring {
    /// Creates a new handler with the given hardfork.
    fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>;
}

impl<DB: Database, EXT: Debug> EvmWiring for EthereumWiring<DB, EXT> {
    fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
    where
        DB: Database,
    {
        spec_to_generic!(
            hardfork,
            EvmHandler {
                spec_id: hardfork,
                instruction_table: InstructionTables::new_plain::<SPEC>(),
                registers: Vec::new(),
                validation: ValidationHandler::new::<SPEC>(),
                pre_execution: PreExecutionHandler::new::<SPEC>(),
                post_execution: PostExecutionHandler::mainnet::<SPEC>(),
                execution: ExecutionHandler::new::<SPEC>(),
            }
        )
    }
}
