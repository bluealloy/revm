use super::InnerEvmContext;
use crate::{
    precompile::{Precompile, PrecompileResult},
    primitives::{db::Database, Address, Bytes, EvmWiring, HashMap, HashSet},
};
use core::fmt::Debug;
use derive_where::derive_where;
use dyn_clone::DynClone;
use revm_precompile::{PrecompileSpecId, PrecompileWithAddress, Precompiles};
use std::{boxed::Box, sync::Arc};

/// A single precompile handler.
#[derive_where(Clone)]
pub enum ContextPrecompile<EvmWiringT: EvmWiring, DB: Database> {
    /// Ordinary precompiles
    Ordinary(Precompile),
    /// Stateful precompile that is Arc over [`ContextStatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and Context.
    ContextStateful(ContextStatefulPrecompileArc<EvmWiringT, DB>),
    /// Mutable stateful precompile that is Box over [`ContextStatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and context.
    ContextStatefulMut(ContextStatefulPrecompileBox<EvmWiringT, DB>),
}

impl<EvmWiringT: EvmWiring, DB: Database> Debug for ContextPrecompile<EvmWiringT, DB> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ordinary(arg0) => f.debug_tuple("Ordinary").field(arg0).finish(),
            Self::ContextStateful(_arg0) => f.debug_tuple("ContextStateful").finish(),
            Self::ContextStatefulMut(_arg0) => f.debug_tuple("ContextStatefulMut").finish(),
        }
    }
}

#[derive_where(Clone, Debug)]
enum PrecompilesCow<EvmWiringT: EvmWiring, DB: Database> {
    /// Default precompiles, returned by `Precompiles::new`. Used to fast-path the default case.
    StaticRef(&'static Precompiles),
    Owned(HashMap<Address, ContextPrecompile<EvmWiringT, DB>>),
}

/// Precompiles context.

#[derive_where(Clone, Debug, Default)]
pub struct ContextPrecompiles<EvmWiringT: EvmWiring, DB: Database> {
    inner: PrecompilesCow<EvmWiringT, DB>,
}

impl<EvmWiringT: EvmWiring, DB: Database> ContextPrecompiles<EvmWiringT, DB> {
    /// Creates a new precompiles context at the given spec ID.
    ///
    /// This is a cheap operation that does not allocate by reusing the global precompiles.
    #[inline]
    pub fn new(spec_id: PrecompileSpecId) -> Self {
        Self::from_static_precompiles(Precompiles::new(spec_id))
    }

    /// Creates a new precompiles context from the given static precompiles.
    ///
    /// NOTE: The internal precompiles must not be `StatefulMut` or `call` will panic.
    /// This is done because the default precompiles are not stateful.
    #[inline]
    pub fn from_static_precompiles(precompiles: &'static Precompiles) -> Self {
        Self {
            inner: PrecompilesCow::StaticRef(precompiles),
        }
    }

    /// Creates a new precompiles context from the given precompiles.
    #[inline]
    pub fn from_precompiles(
        precompiles: HashMap<Address, ContextPrecompile<EvmWiringT, DB>>,
    ) -> Self {
        Self {
            inner: PrecompilesCow::Owned(precompiles),
        }
    }

    /// Returns precompiles addresses as a HashSet.
    pub fn addresses_set(&self) -> HashSet<Address> {
        match self.inner {
            PrecompilesCow::StaticRef(inner) => inner.addresses_set().clone(),
            PrecompilesCow::Owned(ref inner) => inner.keys().cloned().collect(),
        }
    }

    /// Returns precompiles addresses.
    #[inline]
    pub fn addresses<'a>(&'a self) -> Box<dyn ExactSizeIterator<Item = &Address> + 'a> {
        match self.inner {
            PrecompilesCow::StaticRef(inner) => Box::new(inner.addresses()),
            PrecompilesCow::Owned(ref inner) => Box::new(inner.keys()),
        }
    }

    /// Returns `true` if the precompiles contains the given address.
    #[inline]
    pub fn contains(&self, address: &Address) -> bool {
        match self.inner {
            PrecompilesCow::StaticRef(inner) => inner.contains(address),
            PrecompilesCow::Owned(ref inner) => inner.contains_key(address),
        }
    }

    /// Call precompile and executes it. Returns the result of the precompile execution.
    ///
    /// Returns `None` if the precompile does not exist.
    #[inline]
    pub fn call(
        &mut self,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<EvmWiringT, DB>,
    ) -> Option<PrecompileResult> {
        Some(match self.inner {
            PrecompilesCow::StaticRef(p) => {
                p.get(address)?.call_ref(bytes, gas_limit, &evmctx.env.cfg)
            }
            PrecompilesCow::Owned(ref mut owned) => match owned.get_mut(address)? {
                ContextPrecompile::Ordinary(p) => p.call(bytes, gas_limit, &evmctx.env.cfg),
                ContextPrecompile::ContextStateful(p) => p.call(bytes, gas_limit, evmctx),
                ContextPrecompile::ContextStatefulMut(p) => p.call_mut(bytes, gas_limit, evmctx),
            },
        })
    }

    /// Returns a mutable reference to the precompiles map.
    ///
    /// Clones the precompiles map if it is shared.
    #[inline]
    pub fn to_mut(&mut self) -> &mut HashMap<Address, ContextPrecompile<EvmWiringT, DB>> {
        if let PrecompilesCow::StaticRef(_) = self.inner {
            self.mutate_into_owned();
        }

        let PrecompilesCow::Owned(inner) = &mut self.inner else {
            unreachable!("self is mutated to Owned.")
        };
        inner
    }

    /// Mutates Self into Owned variant, or do nothing if it is already Owned.
    /// Mutation will clone all precompiles.
    #[cold]
    fn mutate_into_owned(&mut self) {
        let PrecompilesCow::StaticRef(precompiles) = self.inner else {
            return;
        };
        self.inner = PrecompilesCow::Owned(
            precompiles
                .inner()
                .iter()
                .map(|(k, v)| (*k, v.clone().into()))
                .collect(),
        );
    }
}

impl<EvmWiringT: EvmWiring, DB: Database> Extend<(Address, ContextPrecompile<EvmWiringT, DB>)>
    for ContextPrecompiles<EvmWiringT, DB>
{
    fn extend<T: IntoIterator<Item = (Address, ContextPrecompile<EvmWiringT, DB>)>>(
        &mut self,
        iter: T,
    ) {
        self.to_mut().extend(iter.into_iter().map(Into::into))
    }
}

impl<EvmWiringT: EvmWiring, DB: Database> Extend<PrecompileWithAddress>
    for ContextPrecompiles<EvmWiringT, DB>
{
    fn extend<T: IntoIterator<Item = PrecompileWithAddress>>(&mut self, iter: T) {
        self.to_mut().extend(iter.into_iter().map(|precompile| {
            let (address, precompile) = precompile.into();
            (address, precompile.into())
        }));
    }
}

impl<EvmWiringT: EvmWiring, DB: Database> Default for PrecompilesCow<EvmWiringT, DB> {
    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

/// Context aware stateful precompile trait. It is used to create
/// a arc precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompile<EvmWiringT: EvmWiring, DB: Database>: Sync + Send {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<EvmWiringT, DB>,
    ) -> PrecompileResult;
}

/// Context aware mutable stateful precompile trait. It is used to create
/// a boxed precompile in [`ContextPrecompile`].
pub trait ContextStatefulPrecompileMut<EvmWiringT: EvmWiring, DB: Database>:
    DynClone + Send + Sync
{
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<EvmWiringT, DB>,
    ) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(<EvmWiringT, DB> ContextStatefulPrecompileMut<EvmWiringT, DB>);

/// Arc over context stateful precompile.
pub type ContextStatefulPrecompileArc<EvmWiringT, DB> =
    Arc<dyn ContextStatefulPrecompile<EvmWiringT, DB>>;

/// Box over context mutable stateful precompile
pub type ContextStatefulPrecompileBox<EvmWiringT, DB> =
    Box<dyn ContextStatefulPrecompileMut<EvmWiringT, DB>>;

impl<EvmWiringT: EvmWiring, DB: Database> From<Precompile> for ContextPrecompile<EvmWiringT, DB> {
    fn from(p: Precompile) -> Self {
        ContextPrecompile::Ordinary(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::EmptyDB, primitives::EthereumWiring};

    #[test]
    fn test_precompiles_context() {
        let custom_address = Address::with_last_byte(0xff);

        let mut precompiles =
            ContextPrecompiles::<EthereumWiring, EmptyDB>::new(PrecompileSpecId::HOMESTEAD);
        assert_eq!(precompiles.addresses().count(), 4);
        assert!(matches!(precompiles.inner, PrecompilesCow::StaticRef(_)));
        assert!(!precompiles.contains(&custom_address));

        let precompile = Precompile::Standard(|_, _| panic!());
        precompiles.extend([(custom_address, precompile.into())]);
        assert_eq!(precompiles.addresses().count(), 5);
        assert!(matches!(precompiles.inner, PrecompilesCow::Owned(_)));
        assert!(precompiles.contains(&custom_address));
    }
}
