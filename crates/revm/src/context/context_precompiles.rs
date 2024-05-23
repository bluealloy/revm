use crate::{
    precompile::{Precompile, PrecompileResult},
    primitives::{db::Database, Address, Bytes, ChainSpec, HashMap},
};
use core::ops::{Deref, DerefMut};
use dyn_clone::DynClone;
use revm_precompile::Precompiles;
use std::{boxed::Box, sync::Arc};

use super::InnerEvmContext;

/// Precompile and its handlers.
pub enum ContextPrecompile<ChainSpecT: ChainSpec, DB: Database> {
    /// Ordinary precompiles
    Ordinary(Precompile),
    /// Stateful precompile that is Arc over [`ContextStatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and Context.
    ContextStateful(ContextStatefulPrecompileArc<ChainSpecT, DB>),
    /// Mutable stateful precompile that is Box over [`ContextStatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and context.
    ContextStatefulMut(ContextStatefulPrecompileBox<ChainSpecT, DB>),
}

impl<ChainSpecT: ChainSpec, DB: Database> Clone for ContextPrecompile<ChainSpecT, DB> {
    fn clone(&self) -> Self {
        match self {
            Self::Ordinary(arg0) => Self::Ordinary(arg0.clone()),
            Self::ContextStateful(arg0) => Self::ContextStateful(arg0.clone()),
            Self::ContextStatefulMut(arg0) => Self::ContextStatefulMut(arg0.clone()),
        }
    }
}

#[derive(Clone)]
pub struct ContextPrecompiles<ChainSpecT: ChainSpec, DB: Database> {
    inner: HashMap<Address, ContextPrecompile<ChainSpecT, DB>>,
}

impl<ChainSpecT: ChainSpec, DB: Database> ContextPrecompiles<ChainSpecT, DB> {
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
        other: impl IntoIterator<Item = impl Into<(Address, ContextPrecompile<ChainSpecT, DB>)>>,
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
        evmctx: &mut InnerEvmContext<ChainSpecT, DB>,
    ) -> Option<PrecompileResult> {
        let precompile = self.inner.get_mut(&addess)?;

        match precompile {
            ContextPrecompile::Ordinary(p) => Some(p.call(bytes, gas_price, &evmctx.env.cfg)),
            ContextPrecompile::ContextStatefulMut(p) => Some(p.call_mut(bytes, gas_price, evmctx)),
            ContextPrecompile::ContextStateful(p) => Some(p.call(bytes, gas_price, evmctx)),
        }
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> Default for ContextPrecompiles<ChainSpecT, DB> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> Deref for ContextPrecompiles<ChainSpecT, DB> {
    type Target = HashMap<Address, ContextPrecompile<ChainSpecT, DB>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> DerefMut for ContextPrecompiles<ChainSpecT, DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Context aware stateful precompile trait. It is used to create
/// a arc precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompile<ChainSpecT: ChainSpec, DB: Database>: Sync + Send {
    fn call(
        &self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut InnerEvmContext<ChainSpecT, DB>,
    ) -> PrecompileResult;
}

/// Context aware mutable stateful precompile trait. It is used to create
/// a boxed precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompileMut<ChainSpecT: ChainSpec, DB: Database>:
    DynClone + Send + Sync
{
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_price: u64,
        evmctx: &mut InnerEvmContext<ChainSpecT, DB>,
    ) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(<ChainSpecT, DB> ContextStatefulPrecompileMut<ChainSpecT, DB>);

/// Arc over context stateful precompile.
pub type ContextStatefulPrecompileArc<ChainSpecT, DB> =
    Arc<dyn ContextStatefulPrecompile<ChainSpecT, DB>>;

/// Box over context mutable stateful precompile
pub type ContextStatefulPrecompileBox<ChainSpecT, DB> =
    Box<dyn ContextStatefulPrecompileMut<ChainSpecT, DB>>;

impl<ChainSpecT: ChainSpec, DB: Database> From<Precompile> for ContextPrecompile<ChainSpecT, DB> {
    fn from(p: Precompile) -> Self {
        ContextPrecompile::Ordinary(p)
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> From<Precompiles> for ContextPrecompiles<ChainSpecT, DB> {
    fn from(p: Precompiles) -> Self {
        ContextPrecompiles {
            inner: p.inner.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> From<&Precompiles>
    for ContextPrecompiles<ChainSpecT, DB>
{
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
