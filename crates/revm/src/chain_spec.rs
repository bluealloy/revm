use crate::{
    handler::{ExecutionHandler, PostExecutionHandler, PreExecutionHandler, ValidationHandler}, interpreter::opcode::InstructionTables, primitives::{db::Database, spec_to_generic, EthereumWiring, EvmWiring as PrimitiveEvmWiring}, EvmBuilder, EvmHandler
};
use std::fmt::Debug;
use std::vec::Vec;

pub trait EvmWiring: PrimitiveEvmWiring {
    /// Creates a new handler with the given hardfork.
    fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>;

    // Returns Revm Builder with 
    fn builder<'evm>() -> EvmBuilder<'evm,Self>;
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
