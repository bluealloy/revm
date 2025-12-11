//! Contains Monad specific precompiles.
use revm::{
    context::Cfg,
    context_interface::ContextTr,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{CallInputs, InterpreterResult},
    precompile::{
        self, bn254, secp256r1, Precompile, PrecompileError, PrecompileId, PrecompileResult,
        Precompiles,
    },
    primitives::{hardfork::SpecId, Address, OnceLock},
};
use std::{boxed::Box, string::String};

/// Monad precompile provider
#[derive(Debug, Clone)]
pub struct MonadPrecompiles {
    /// Inner precompile provider is same as Ethereums.
    inner: EthPrecompiles,
}

impl MonadPrecompiles {
    /// Create a new precompile provider with the given spec.
    #[inline]
    pub fn new_with_spec(spec: SpecId) -> Self {
        Self {
            inner: EthPrecompiles {
                precompiles: Precompiles::new(spec.into()),
                spec,
            },
        }
    }

    /// Precompiles getter.
    #[inline]
    pub fn precompiles(&self) -> &'static Precompiles {
        self.inner.precompiles
    }
}

impl<CTX> PrecompileProvider<CTX> for MonadPrecompiles
where
    // TODO:Update SpecId to MonadSpecId
    CTX: ContextTr<Cfg: Cfg<Spec = SpecId>>,
{
    type Output = InterpreterResult;

    #[inline]
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        // TODO: use .spec once added to the struct
        if spec == self.inner.spec {
            return false;
        }
        *self = Self::new_with_spec(spec);
        true
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<Self::Output>, String> {
        self.inner.run(context, inputs)
    }

    #[inline]
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        self.inner.warm_addresses()
    }

    #[inline]
    fn contains(&self, address: &Address) -> bool {
        self.inner.contains(address)
    }
}

impl Default for MonadPrecompiles {
    fn default() -> Self {
        Self::new_with_spec(SpecId::OSAKA)
    }
}
