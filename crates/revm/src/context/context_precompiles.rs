use super::InnerEvmContext;
use crate::{
    precompile::{Precompile, PrecompileResult},
    primitives::{db::Database, Address, Bytes, HashMap, HashSet},
};
use dyn_clone::DynClone;
use revm_precompile::{PrecompileSpecId, PrecompileWithAddress, Precompiles};
use std::{boxed::Box, sync::Arc};

/// A single precompile handler.
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
            Self::Ordinary(p) => Self::Ordinary(p.clone()),
            Self::ContextStateful(p) => Self::ContextStateful(p.clone()),
            Self::ContextStatefulMut(p) => Self::ContextStatefulMut(p.clone()),
        }
    }
}

enum PrecompilesCow<DB: Database> {
    /// Default precompiles, returned by `Precompiles::new`. Used to fast-path the default case.
    StaticRef(&'static Precompiles),
    Owned(HashMap<Address, ContextPrecompile<DB>>),
}

impl<DB: Database> Clone for PrecompilesCow<DB> {
    fn clone(&self) -> Self {
        match *self {
            PrecompilesCow::StaticRef(p) => PrecompilesCow::StaticRef(p),
            PrecompilesCow::Owned(ref inner) => PrecompilesCow::Owned(inner.clone()),
        }
    }
}

/// Precompiles context.
pub struct ContextPrecompiles<DB: Database> {
    inner: PrecompilesCow<DB>,
}

impl<DB: Database> Clone for ContextPrecompiles<DB> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<DB: Database> ContextPrecompiles<DB> {
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
    pub fn from_precompiles(precompiles: HashMap<Address, ContextPrecompile<DB>>) -> Self {
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
        gas_price: u64,
        evmctx: &mut InnerEvmContext<DB>,
    ) -> Option<PrecompileResult> {
        Some(match self.inner {
            PrecompilesCow::StaticRef(p) => p.get(address)?.call_ref(bytes, gas_price, &evmctx.env),
            PrecompilesCow::Owned(ref mut owned) => match owned.get_mut(address)? {
                ContextPrecompile::Ordinary(p) => p.call(bytes, gas_price, &evmctx.env),
                ContextPrecompile::ContextStateful(p) => p.call(bytes, gas_price, evmctx),
                ContextPrecompile::ContextStatefulMut(p) => p.call_mut(bytes, gas_price, evmctx),
            },
        })
    }

    /// Returns a mutable reference to the precompiles map.
    ///
    /// Clones the precompiles map if it is shared.
    #[inline]
    pub fn to_mut(&mut self) -> &mut HashMap<Address, ContextPrecompile<DB>> {
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

impl<DB: Database> Extend<(Address, ContextPrecompile<DB>)> for ContextPrecompiles<DB> {
    fn extend<T: IntoIterator<Item = (Address, ContextPrecompile<DB>)>>(&mut self, iter: T) {
        self.to_mut().extend(iter.into_iter().map(Into::into))
    }
}

impl<DB: Database> Extend<PrecompileWithAddress> for ContextPrecompiles<DB> {
    fn extend<T: IntoIterator<Item = PrecompileWithAddress>>(&mut self, iter: T) {
        self.to_mut().extend(iter.into_iter().map(|precompile| {
            let (address, precompile) = precompile.into();
            (address, precompile.into())
        }));
    }
}

impl<DB: Database> Default for ContextPrecompiles<DB> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<DB: Database> Default for PrecompilesCow<DB> {
    fn default() -> Self {
        Self::Owned(Default::default())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::EmptyDB;

    #[test]
    fn test_precompiles_context() {
        let custom_address = Address::with_last_byte(0xff);

        let mut precompiles = ContextPrecompiles::<EmptyDB>::new(PrecompileSpecId::HOMESTEAD);
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
