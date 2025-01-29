use crate::{OpSpec, OpSpecId};
use once_cell::race::OnceBox;
use precompile::{secp256r1, PrecompileErrors, Precompiles};
use revm::{
    context::Cfg, context_interface::CfgGetter, handler::EthPrecompileProvider,
    handler_interface::PrecompileProvider, interpreter::InterpreterResult,
    specification::hardfork::SpecId,
};
use std::boxed::Box;

pub struct OpPrecompileProvider<CTX, ERROR> {
    precompile_provider: EthPrecompileProvider<CTX, ERROR>,
}

impl<CTX, ERROR> Clone for OpPrecompileProvider<CTX, ERROR> {
    fn clone(&self) -> Self {
        Self {
            precompile_provider: self.precompile_provider.clone(),
        }
    }
}

impl<CTX, ERROR> OpPrecompileProvider<CTX, ERROR> {
    pub fn new(precompiles: &'static Precompiles) -> Self {
        Self {
            precompile_provider: EthPrecompileProvider {
                precompiles,
                _phantom: core::marker::PhantomData,
            },
        }
    }

    #[inline]
    pub fn new_with_spec(spec: OpSpec) -> Self {
        match spec {
            // No changes
            spec @ (OpSpec::Eth(
                SpecId::FRONTIER
                | SpecId::FRONTIER_THAWING
                | SpecId::HOMESTEAD
                | SpecId::DAO_FORK
                | SpecId::TANGERINE
                | SpecId::SPURIOUS_DRAGON
                | SpecId::BYZANTIUM
                | SpecId::CONSTANTINOPLE
                | SpecId::PETERSBURG
                | SpecId::ISTANBUL
                | SpecId::MUIR_GLACIER
                | SpecId::BERLIN
                | SpecId::LONDON
                | SpecId::ARROW_GLACIER
                | SpecId::GRAY_GLACIER
                | SpecId::MERGE
                | SpecId::SHANGHAI
                | SpecId::CANCUN,
            )
            | OpSpec::Op(
                OpSpecId::BEDROCK
                | OpSpecId::REGOLITH
                | OpSpecId::CANYON
                | OpSpecId::ECOTONE
                | OpSpecId::HOLOCENE,
            )) => Self::new(Precompiles::new(spec.into_eth_spec().into())),
            OpSpec::Op(OpSpecId::FJORD) => Self::new(fjord()),
            OpSpec::Op(OpSpecId::GRANITE) => Self::new(granite()),
            OpSpec::Op(OpSpecId::ISTHMUS)
            | OpSpec::Eth(SpecId::PRAGUE | SpecId::OSAKA | SpecId::LATEST) => Self::new(isthmus()),
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

/// Returns precompiles for Isthmus spec.
pub fn isthmus() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::cancun().clone();
        // Restrict bn256Pairing input size
        precompiles.extend([secp256r1::P256VERIFY]);
        // Restrict bls12Pairing input size
        precompiles.extend([crate::bls12::pair::ISTHMUS]);
        Box::new(precompiles)
    })
}

impl<CTX, ERROR> PrecompileProvider for OpPrecompileProvider<CTX, ERROR>
where
    CTX: CfgGetter,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    ERROR: From<PrecompileErrors>,
{
    type Context = CTX;
    type Error = ERROR;
    type Output = InterpreterResult;
    type Spec = OpSpec;

    #[inline]
    fn set_spec(&mut self, spec: Self::Spec) {
        *self = Self::new_with_spec(spec);
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &precompile::Address,
        bytes: &precompile::Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, Self::Error> {
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

impl<CTX, ERROR> Default for OpPrecompileProvider<CTX, ERROR> {
    fn default() -> Self {
        Self::new_with_spec(OpSpec::Op(OpSpecId::ISTHMUS))
    }
}
