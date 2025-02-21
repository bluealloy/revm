use crate::OpSpecId;
use once_cell::race::OnceBox;
use precompile::{secp256r1, PrecompileError, Precompiles};
use revm::{
    context::Cfg,
    context_interface::ContextTr,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::InterpreterResult,
};
use std::boxed::Box;

pub struct OpPrecompileProvider<CTX> {
    precompile_provider: EthPrecompiles<CTX>,
}

impl<CTX> Clone for OpPrecompileProvider<CTX> {
    fn clone(&self) -> Self {
        Self {
            precompile_provider: self.precompile_provider.clone(),
        }
    }
}

impl<CTX> OpPrecompileProvider<CTX> {
    pub fn new(precompiles: &'static Precompiles) -> Self {
        Self {
            precompile_provider: EthPrecompiles {
                precompiles,
                _phantom: core::marker::PhantomData,
            },
        }
    }

    #[inline]
    pub fn new_with_spec(spec: OpSpecId) -> Self {
        match spec {
            spec @ (OpSpecId::BEDROCK
            | OpSpecId::REGOLITH
            | OpSpecId::CANYON
            | OpSpecId::ECOTONE) => Self::new(Precompiles::new(spec.into_eth_spec().into())),
            OpSpecId::FJORD => Self::new(fjord()),
            OpSpecId::GRANITE | OpSpecId::HOLOCENE => Self::new(granite()),
            OpSpecId::ISTHMUS | OpSpecId::INTEROP => Self::new(isthumus()),
        }
    }
}

/// Returns precompiles for Fjor spec.
pub fn fjord() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::cancun().clone();
        // EIP-7212: secp256r1 P256verify
        precompiles.extend([crate::bn128::pair::GRANITE]);
        Box::new(precompiles)
    })
}

/// Returns precompiles for Granite spec.
pub fn granite() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::cancun().clone();
        // Restrict bn256Pairing input size
        precompiles.extend([secp256r1::P256VERIFY]);
        Box::new(precompiles)
    })
}

/// Returns precompiles for isthumus spec.
pub fn isthumus() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let precompiles = granite().clone();
        // Prague bls12 precompiles
        // Don't include BLS12-381 precompiles in no_std builds.
        #[cfg(feature = "blst")]
        let precompiles = {
            let mut precompiles = precompiles;
            precompiles.extend(precompile::bls12_381::precompiles());
            precompiles
        };
        Box::new(precompiles)
    })
}

impl<CTX> PrecompileProvider for OpPrecompileProvider<CTX>
where
    CTX: ContextTr<Cfg: Cfg<Spec = OpSpecId>>,
{
    type Context = CTX;
    type Output = InterpreterResult;

    #[inline]
    fn set_spec(&mut self, spec: <<Self::Context as ContextTr>::Cfg as Cfg>::Spec) {
        *self = Self::new_with_spec(spec);
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &precompile::Address,
        bytes: &precompile::Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, PrecompileError> {
        self.precompile_provider
            .run(context, address, bytes, gas_limit)
    }

    #[inline]
    fn warm_addresses(&self) -> Box<impl Iterator<Item = precompile::Address> + '_> {
        self.precompile_provider.warm_addresses()
    }

    #[inline]
    fn contains(&self, address: &precompile::Address) -> bool {
        self.precompile_provider.contains(address)
    }
}

impl<CTX> Default for OpPrecompileProvider<CTX> {
    fn default() -> Self {
        Self::new_with_spec(OpSpecId::ISTHMUS)
    }
}
