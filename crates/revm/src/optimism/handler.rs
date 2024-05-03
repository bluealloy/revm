use revm_interpreter::opcode::InstructionTables;

use crate::{
    handler::{
        register::{EvmHandler, HandleRegisters},
        ExecutionHandler, PostExecutionHandler, PreExecutionHandler, ValidationHandler,
    },
    optimism_spec_to_generic,
    primitives::db::Database,
};

use super::{OptimismChainSpec, OptimismSpec};

impl<EXT, DB: Database> EvmHandler<'_, OptimismChainSpec, EXT, DB> {
    /// Default handler for optimism.
    pub fn optimism<SPEC: OptimismSpec>() -> Self {
        let mut handler = Self {
            spec_id: SPEC::OPTIMISM_SPEC_ID,
            instruction_table: Some(InstructionTables::new_plain::<SPEC>()),
            registers: Vec::new(),
            validation: ValidationHandler::new::<SPEC>(),
            pre_execution: PreExecutionHandler::new::<SPEC>(),
            post_execution: PostExecutionHandler::new::<SPEC>(),
            execution: ExecutionHandler::new::<SPEC>(),
        };

        handler.append_handler_register(HandleRegisters::Plain(
            crate::optimism::optimism_handle_register::<DB, EXT>,
        ));

        handler
    }

    /// Optimism with spec. Similar to [`Self::mainnet_with_spec`].
    pub fn optimism_with_spec(spec_id: crate::optimism::OptimismSpecId) -> Self {
        optimism_spec_to_generic!(spec_id, Self::optimism::<SPEC>())
    }
}
