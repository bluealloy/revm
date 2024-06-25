use crate::{
    handler::register::{EvmHandler, HandleRegisters},
    primitives::db::Database,
};

use super::OptimismChainSpec;

impl<EXT, DB: Database> EvmHandler<'_, OptimismChainSpec, EXT, DB> {
    /// Optimism with spec. Similar to [`Self::mainnet_with_spec`].
    pub fn optimism_with_spec(spec_id: crate::optimism::OptimismSpecId) -> Self {
        let mut handler = Self::mainnet_with_spec(spec_id);

        handler.append_handler_register(HandleRegisters::Plain(
            crate::optimism::optimism_handle_register::<DB, EXT>,
        ));

        handler
    }
}
