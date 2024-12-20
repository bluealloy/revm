use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    Context,
};

/// Helper type for easier integration with previous version of inspector.
pub type PrevContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB>;
