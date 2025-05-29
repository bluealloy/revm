use auto_impl::auto_impl;
use revm::context::Cfg;

#[auto_impl(&, &mut, Box, Arc)]
pub trait CfgExt: Cfg {
    fn allow_mocking(&self) -> bool;
}
