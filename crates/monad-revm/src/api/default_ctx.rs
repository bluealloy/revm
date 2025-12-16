// Default Monad context type and factory.

use crate::MonadSpecId;
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    database_interface::EmptyDB,
    Context, Journal, MainContext,
};

/// Type alias for the default Monad context.
///
/// Uses standard Ethereum types since Monad doesn't need custom tx/block types.
/// The key difference is using MonadSpecId instead of SpecId.
pub type MonadContext<DB> = Context<BlockEnv, TxEnv, CfgEnv<MonadSpecId>, DB, Journal<DB>, ()>;

/// Trait for creating a default Monad context.
pub trait DefaultMonad {
    fn monad() -> MonadContext<EmptyDB>;
}

impl DefaultMonad for MonadContext<EmptyDB> {
    fn monad() -> Self {
        Context::mainnet()
            .with_cfg(CfgEnv::new_with_spec(MonadSpecId::default()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::builder::MonadBuilder;
    use revm::{inspector::NoOpInspector, ExecuteEvm};

    #[test]
    fn default_run_monad() {
        let ctx = Context::monad();
        let mut evm = ctx.build_monad_with_inspector(NoOpInspector {});
        let tx = TxEnv::default();
        let _ = evm.transact(tx);
    }
}