use crate::{
    precompile::{Precompile, PrecompileResult},
    primitives::{db::Database, Address, Bytes, HashMap},
    EvmContext,
};
use core::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use revm_precompile::Precompiles;
use std::{boxed::Box, sync::Arc};

/// Precompile and its handlers.
pub enum ContextPrecompile<DB: Database, EXTCTX> {
    /// Ordinary precompiles
    Ordinary(Precompile),
    /// Stateful precompile that is Arc over [`ContextStatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and Context.
    ContextStateful(ContextStatefulPrecompileArc<EvmContext<DB>, EXTCTX>),
    /// Mutable stateful precompile that is Box over [`ContextStatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and context.
    ContextStatefulMut(ContextStatefulPrecompileBox<EvmContext<DB>, EXTCTX>),
}

impl<DB: Database, EXTCTX> Clone for ContextPrecompile<DB, EXTCTX> {
    fn clone(&self) -> Self {
        match self {
            Self::Ordinary(arg0) => Self::Ordinary(arg0.clone()),
            Self::ContextStateful(arg0) => Self::ContextStateful(arg0.clone()),
            Self::ContextStatefulMut(arg0) => Self::ContextStatefulMut(arg0.clone()),
        }
    }
}

#[derive(Clone)]
pub struct ContextPrecompiles<DB: Database, EXTCTX> {
    inner: HashMap<Address, ContextPrecompile<DB, EXTCTX>>,
}

impl<DB: Database, EXTCTX> ContextPrecompiles<DB, EXTCTX> {
    /// Returns precompiles addresses.
    #[inline]
    pub fn addresses(&self) -> impl Iterator<Item = &Address> {
        self.inner.keys()
    }

    /// Extends the precompiles with the given precompiles.
    ///
    /// Other precompiles with overwrite existing precompiles.
    #[inline]
    pub fn extend(
        &mut self,
        other: impl IntoIterator<Item = impl Into<(Address, ContextPrecompile<DB, EXTCTX>)>>,
    ) {
        self.inner.extend(other.into_iter().map(Into::into));
    }

    /// Call precompile and executes it. Returns the result of the precompile execution.
    /// None if the precompile does not exist.
    #[inline]
    pub fn call(
        &mut self,
        addess: Address,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut EvmContext<DB>,
        extctx: &mut EXTCTX,
    ) -> Option<PrecompileResult> {
        let precompile = self.inner.get_mut(&addess)?;

        match precompile {
            ContextPrecompile::Ordinary(p) => Some(p.call(bytes, gas_price, &evmctx.env)),
            ContextPrecompile::ContextStatefulMut(p) => {
                Some(p.call_mut(bytes, gas_price, evmctx, extctx))
            }
            ContextPrecompile::ContextStateful(p) => Some(p.call(bytes, gas_price, evmctx, extctx)),
        }
    }
}

impl<DB: Database, EXTCTX> Default for ContextPrecompiles<DB, EXTCTX> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<DB: Database, EXTCTX> Deref for ContextPrecompiles<DB, EXTCTX> {
    type Target = HashMap<Address, ContextPrecompile<DB, EXTCTX>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB: Database, EXTCTX> DerefMut for ContextPrecompiles<DB, EXTCTX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Context aware stateful precompile trait. It is used to create
/// a arc precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompile<EVMCTX, EXTCTX>: Sync + Send {
    fn call(
        &self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut EVMCTX,
        extctx: &mut EXTCTX,
    ) -> PrecompileResult;
}

/// Context aware mutable stateful precompile trait. It is used to create
/// a boxed precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompileMut<EVMCTX, EXTCTX>: DynClone + Send + Sync {
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut EVMCTX,
        extctx: &mut EXTCTX,
    ) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(<EVMCTX, EXTCTX> ContextStatefulPrecompileMut<EVMCTX, EXTCTX>);

/// Arc over context stateful precompile.
pub type ContextStatefulPrecompileArc<EVMCTX, EXTCTX> =
    Arc<dyn ContextStatefulPrecompile<EVMCTX, EXTCTX>>;

/// Box over context mutable stateful precompile
pub type ContextStatefulPrecompileBox<EVMCTX, EXTCTX> =
    Box<dyn ContextStatefulPrecompileMut<EVMCTX, EXTCTX>>;

impl<DB: Database, EXTCTX> From<Precompile> for ContextPrecompile<DB, EXTCTX> {
    fn from(p: Precompile) -> Self {
        ContextPrecompile::Ordinary(p)
    }
}

impl<DB: Database, EXTCTX> From<Precompiles> for ContextPrecompiles<DB, EXTCTX> {
    fn from(p: Precompiles) -> Self {
        ContextPrecompiles {
            inner: p.inner.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl<DB: Database, EXTCTX> From<&Precompiles> for ContextPrecompiles<DB, EXTCTX> {
    fn from(p: &Precompiles) -> Self {
        ContextPrecompiles {
            inner: p
                .inner
                .iter()
                .map(|(&k, v)| (k, v.clone().into()))
                .collect(),
        }
    }
}
