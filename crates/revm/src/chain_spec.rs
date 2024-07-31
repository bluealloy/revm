use crate::{
    handler::{ExecutionHandler, PostExecutionHandler, PreExecutionHandler, ValidationHandler},
    interpreter::opcode::InstructionTables,
    primitives::{db::Database, spec_to_generic, EthEvmWiring},
    EvmHandler,
};
use std::vec::Vec;

pub trait EvmWiring: crate::primitives::EvmWiring {
    /// The type that contains all context information for the chain's EVM execution.
    type Context: Default;

    /// Creates a new handler with the given hardfork.
    fn handler<'evm, EXT, DB>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self, EXT, DB>
    where
        DB: Database;
}

impl EvmWiring for EthEvmWiring {
    type Context = ();

    fn handler<'evm, EXT, DB>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self, EXT, DB>
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
