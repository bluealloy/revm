use crate::{Bytes, Env};
use core::fmt;
use dyn_clone::DynClone;
use std::{boxed::Box, sync::Arc};

/// A precompile operation result.
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<(u64, Bytes), PrecompileError>;

pub type StandardPrecompileFn = fn(&Bytes, u64) -> PrecompileResult;
pub type EnvPrecompileFn = fn(&Bytes, u64, env: &Env) -> PrecompileResult;

/// Stateful precompile trait. It is used to create
/// a arc precompile Precompile::Stateful.
pub trait StatefulPrecompile: Sync + Send {
    fn call(&self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult;
}

/// Stateful precompile trait with a generic context. It is used to create
/// a arc precompile Precompile::Stateful.
pub trait ContextStatefulPrecompile<CTX, EXTCTX>: Sync + Send {
    fn call(
        &self,
        bytes: &Bytes,
        gas_price: u64,
        context: &mut CTX,
        extctx: &mut EXTCTX,
    ) -> PrecompileResult;
}

/// Mutable stateful precompile trait. It is used to create
/// a boxed precompile in Precompile::StatefulMut.
pub trait StatefulPrecompileMut: DynClone + Send + Sync {
    fn call_mut(&mut self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(StatefulPrecompileMut);

/// Mutable stateful precompile trait. It is used to create
/// a boxed precompile in Precompile::StatefulMut.
pub trait ContextStatefulPrecompileMut<CTX, EXTCTX>: DynClone + Send + Sync {
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_price: u64,
        ctx: &mut CTX,
        extctx: &mut EXTCTX,
    ) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(<CTX, EXTCTX> ContextStatefulPrecompileMut<CTX, EXTCTX>);

/// Arc over stateful precompile.
pub type StatefulPrecompileArc = Arc<dyn StatefulPrecompile>;

/// Box over mutable stateful precompile
pub type StatefulPrecompileBox = Box<dyn StatefulPrecompileMut>;

/// Arc over stateful precompile.
pub type ContextStatefulPrecompileArc<CTX, EXTCTX> =
    Arc<dyn ContextStatefulPrecompile<CTX, EXTCTX>>;

/// Box over mutable stateful precompile
pub type ContextStatefulPrecompileBox<CTX, EXTCTX> =
    Box<dyn ContextStatefulPrecompileMut<CTX, EXTCTX>>;

/// Precompile and its handlers.
pub enum Precompile<CTX, EXTCTX> {
    /// Standard simple precompile that takes input and gas limit.
    Standard(StandardPrecompileFn),
    /// Similar to Standard but takes reference to environment.
    Env(EnvPrecompileFn),
    /// Stateful precompile that is Arc over [`StatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and environment.
    Stateful(StatefulPrecompileArc),
    /// Mutable stateful precompile that is Box over [`StatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and environment.
    StatefulMut(StatefulPrecompileBox),

    /// Stateful precompile that is Arc over [`StatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and environment.
    ContextStateful(ContextStatefulPrecompileArc<CTX, EXTCTX>),
    /// Mutable stateful precompile that is Box over [`StatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and environment.
    ContextStatefulMut(ContextStatefulPrecompileBox<CTX, EXTCTX>),
}

impl<CTX, EXTCTX> Clone for Precompile<CTX, EXTCTX> {
    fn clone(&self) -> Self {
        match self {
            Precompile::Standard(p) => Precompile::Standard(*p),
            Precompile::Env(p) => Precompile::Env(*p),
            Precompile::Stateful(p) => Precompile::Stateful(p.clone()),
            Precompile::StatefulMut(p) => Precompile::StatefulMut(p.clone()),
            Precompile::ContextStateful(p) => Precompile::ContextStateful(p.clone()),
            Precompile::ContextStatefulMut(p) => Precompile::ContextStatefulMut(p.clone()),
        }
    }
}

impl<CTX, EXTCXT> From<StandardPrecompileFn> for Precompile<CTX, EXTCXT> {
    fn from(p: StandardPrecompileFn) -> Self {
        Precompile::Standard(p)
    }
}

impl<CTX, EXTCXT> From<EnvPrecompileFn> for Precompile<CTX, EXTCXT> {
    fn from(p: EnvPrecompileFn) -> Self {
        Precompile::Env(p)
    }
}

impl<CTX, EXTCXT> From<StatefulPrecompileArc> for Precompile<CTX, EXTCXT> {
    fn from(p: StatefulPrecompileArc) -> Self {
        Precompile::Stateful(p)
    }
}

impl<CTX, EXTCXT> From<StatefulPrecompileBox> for Precompile<CTX, EXTCXT> {
    fn from(p: StatefulPrecompileBox) -> Self {
        Precompile::StatefulMut(p)
    }
}

impl<CTX, EXTCXT> fmt::Debug for Precompile<CTX, EXTCXT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precompile::Standard(_) => f.write_str("Standard"),
            Precompile::Env(_) => f.write_str("Env"),
            Precompile::Stateful(_) => f.write_str("Stateful"),
            Precompile::StatefulMut(_) => f.write_str("StatefulMut"),
            Precompile::ContextStateful(_) => f.write_str("ContextStateful"),
            Precompile::ContextStatefulMut(_) => f.write_str("ContextStatefulMut"),
        }
    }
}

impl<CTX, EXTCXT> Precompile<CTX, EXTCXT> {
    /// Create a new stateful precompile.
    pub fn new_stateful<P: StatefulPrecompile + 'static>(p: P) -> Self {
        Self::Stateful(Arc::new(p))
    }

    /// Create a new mutable stateful precompile.
    pub fn new_stateful_mut<P: StatefulPrecompileMut + 'static>(p: P) -> Self {
        Self::StatefulMut(Box::new(p))
    }

    pub fn call_ordinary(&mut self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult {
        match self {
            Self::Standard(p) => p(bytes, gas_price),
            Self::Env(p) => p(bytes, gas_price, env),
            Self::Stateful(p) => p.call(bytes, gas_price, env),
            Self::StatefulMut(p) => p.call_mut(bytes, gas_price, env),
            _ => panic!("not a standard"),
        }
    }

    pub fn is_standard(&self) -> bool {
        matches!(
            self,
            Self::Standard(_) | Self::Env(_) | Self::Stateful(_) | Self::StatefulMut(_)
        )
    }

    /// Call the precompile with the given input and gas limit and return the result.
    pub fn call_with_context(
        &mut self,
        bytes: &Bytes,
        gas_price: u64,
        context: &mut CTX,
        external_context: &mut EXTCXT,
    ) -> PrecompileResult {
        match self {
            Self::ContextStateful(p) => p.call(bytes, gas_price, context, external_context),
            Self::ContextStatefulMut(p) => p.call_mut(bytes, gas_price, context, external_context),
            _ => panic!("not a context stateful"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileError {
    /// out of gas is the main error. Others are here just for completeness
    OutOfGas,
    // Blake2 errors
    Blake2WrongLength,
    Blake2WrongFinalIndicatorFlag,
    // Modexp errors
    ModexpExpOverflow,
    ModexpBaseOverflow,
    ModexpModOverflow,
    // Bn128 errors
    Bn128FieldPointNotAMember,
    Bn128AffineGFailedToCreate,
    Bn128PairLength,
    // Blob errors
    /// The input length is not exactly 192 bytes.
    BlobInvalidInputLength,
    /// The commitment does not match the versioned hash.
    BlobMismatchedVersion,
    /// The proof verification failed.
    BlobVerifyKzgProofFailed,
}

#[cfg(feature = "std")]
impl std::error::Error for PrecompileError {}

impl fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecompileError::OutOfGas => write!(f, "out of gas"),
            PrecompileError::Blake2WrongLength => write!(f, "wrong input length for blake2"),
            PrecompileError::Blake2WrongFinalIndicatorFlag => {
                write!(f, "wrong final indicator flag for blake2")
            }
            PrecompileError::ModexpExpOverflow => write!(f, "modexp exp overflow"),
            PrecompileError::ModexpBaseOverflow => write!(f, "modexp base overflow"),
            PrecompileError::ModexpModOverflow => write!(f, "modexp mod overflow"),
            PrecompileError::Bn128FieldPointNotAMember => {
                write!(f, "field point not a member of bn128 curve")
            }
            PrecompileError::Bn128AffineGFailedToCreate => {
                write!(f, "failed to create affine g point for bn128 curve")
            }
            PrecompileError::Bn128PairLength => write!(f, "bn128 invalid pair length"),
            PrecompileError::BlobInvalidInputLength => write!(f, "invalid blob input length"),
            PrecompileError::BlobMismatchedVersion => write!(f, "mismatched blob version"),
            PrecompileError::BlobVerifyKzgProofFailed => {
                write!(f, "verifying blob kzg proof failed")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stateful_precompile_mut() {
        #[derive(Default, Clone)]
        struct MyPrecompile {}

        impl StatefulPrecompileMut for MyPrecompile {
            fn call_mut(
                &mut self,
                _bytes: &Bytes,
                _gas_price: u64,
                _env: &Env,
            ) -> PrecompileResult {
                PrecompileResult::Err(PrecompileError::OutOfGas)
            }
        }

        let mut p: Precompile<(), ()> = Precompile::new_stateful_mut(MyPrecompile::default());
        match &mut p {
            Precompile::StatefulMut(p) => {
                let _ = p.call_mut(&Bytes::new(), 0, &Env::default());
            }
            _ => panic!("not a state"),
        }
    }
}
