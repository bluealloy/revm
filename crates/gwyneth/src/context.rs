use crate::cfg::CfgExt;
use revm::context::{Cfg, ContextTr};

// Type alias for Optimism context
pub trait GwynethContextTr: ContextTr<Cfg: CfgExt, Chain = GwynethContext> {}

///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GwynethContext {
    pub xcall_options: Option<crate::xcall::XCallOptions>,
}
