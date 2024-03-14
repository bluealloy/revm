use crate::{
    precompile::{Precompile, PrecompileResult},
    primitives::{db::Database, Address, Bytes, HashMap},
};
use core::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use revm_precompile::Precompiles;
use std::{boxed::Box, sync::Arc};

use super::InnerEvmContext;

/// Precompile and its handlers.
pub enum ContextPrecompile<DB: Database> {
    /// Ordinary precompiles
    Ordinary(Precompile),
    /// Stateful precompile that is Arc over [`ContextStatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and Context.
    ContextStateful(ContextStatefulPrecompileArc<DB>),
    /// Mutable stateful precompile that is Box over [`ContextStatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and context.
    ContextStatefulMut(ContextStatefulPrecompileBox<DB>),
}

impl<DB: Database> Clone for ContextPrecompile<DB> {
    fn clone(&self) -> Self {
        match self {
            Self::Ordinary(arg0) => Self::Ordinary(arg0.clone()),
            Self::ContextStateful(arg0) => Self::ContextStateful(arg0.clone()),
            Self::ContextStatefulMut(arg0) => Self::ContextStatefulMut(arg0.clone()),
        }
    }
}

#[derive(Clone)]
pub struct ContextPrecompiles<DB: Database> {
    inner: HashMap<Address, ContextPrecompile<DB>>,
}

impl<DB: Database> ContextPrecompiles<DB> {
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
        other: impl IntoIterator<Item = impl Into<(Address, ContextPrecompile<DB>)>>,
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
        evmctx: &mut InnerEvmContext<DB>,
    ) -> Option<PrecompileResult> {
        let precompile = self.inner.get_mut(&addess)?;

        match precompile {
            ContextPrecompile::Ordinary(p) => Some(p.call(bytes, gas_price, &evmctx.env)),
            ContextPrecompile::ContextStatefulMut(p) => Some(p.call_mut(bytes, gas_price, evmctx)),
            ContextPrecompile::ContextStateful(p) => Some(p.call(bytes, gas_price, evmctx)),
        }
    }
}

impl<DB: Database> Default for ContextPrecompiles<DB> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<DB: Database> Deref for ContextPrecompiles<DB> {
    type Target = HashMap<Address, ContextPrecompile<DB>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB: Database> DerefMut for ContextPrecompiles<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Context aware stateful precompile trait. It is used to create
/// a arc precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompile<DB: Database>: Sync + Send {
    fn call(
        &self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult;
}

/// Context aware mutable stateful precompile trait. It is used to create
/// a boxed precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompileMut<DB: Database>: DynClone + Send + Sync {
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(<DB> ContextStatefulPrecompileMut<DB>);

/// Arc over context stateful precompile.
pub type ContextStatefulPrecompileArc<DB> = Arc<dyn ContextStatefulPrecompile<DB>>;

/// Box over context mutable stateful precompile
pub type ContextStatefulPrecompileBox<DB> = Box<dyn ContextStatefulPrecompileMut<DB>>;

impl<DB: Database> From<Precompile> for ContextPrecompile<DB> {
    fn from(p: Precompile) -> Self {
        ContextPrecompile::Ordinary(p)
    }
}

impl<DB: Database> From<Precompiles> for ContextPrecompiles<DB> {
    fn from(p: Precompiles) -> Self {
        ContextPrecompiles {
            inner: p.inner.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl<DB: Database> From<&Precompiles> for ContextPrecompiles<DB> {
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
