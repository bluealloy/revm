use crate::primitives::{
    Address, Bytes, Env, Precompile, PrecompileErrors, PrecompileResult, StatefulPrecompile,
};
use crate::PrecompileWithAddress;
use std::{string::String, sync::Arc};

/// Disable kzg precompile. This will return Fatal error on precompile call
pub fn fatal_precompile(address: Address, msg: String) -> PrecompileWithAddress {
    PrecompileWithAddress(address, FatalPrecompile::new_precompile(msg))
}

/// Fatal precompile that returns Fatal error on precompile call
pub struct FatalPrecompile {
    msg: String,
}

impl FatalPrecompile {
    /// Create a new fatal precompile
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    /// Create a new stateful fatal precompile
    pub fn new_precompile(msg: String) -> Precompile {
        Precompile::Stateful(Arc::new(Self::new(msg)))
    }
}

impl StatefulPrecompile for FatalPrecompile {
    fn call(&self, _: &Bytes, _: u64, _: &Env) -> PrecompileResult {
        Err(PrecompileErrors::Fatal {
            msg: self.msg.clone(),
        })
    }
}
