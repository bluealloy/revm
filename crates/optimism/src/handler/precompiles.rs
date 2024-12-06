use crate::{OpSpec, OpSpecId};
use once_cell::race::OnceBox;
use precompile::{secp256r1, PrecompileErrors, Precompiles};
use revm::{
    context::Cfg, context_interface::CfgGetter, handler::EthPrecompileProvider,
    handler_interface::PrecompileProvider, specification::hardfork::SpecId,
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

impl<CTX, ERROR> PrecompileProvider for OpPrecompileProvider<CTX, ERROR>
where
    CTX: CfgGetter,
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
    ERROR: From<PrecompileErrors>,
{
    type Context = CTX;
    type Error = ERROR;

    #[inline]
    fn new(context: &mut Self::Context) -> Self {
        let spec = context.cfg().spec();
        match spec {
            // no changes
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
                OpSpecId::BEDROCK | OpSpecId::REGOLITH | OpSpecId::CANYON | OpSpecId::ECOTONE,
            )) => Self::new(Precompiles::new(spec.into_eth_spec().into())),
            OpSpec::Op(OpSpecId::FJORD) => Self::new(fjord()),
            OpSpec::Op(OpSpecId::GRANITE)
            | OpSpec::Eth(SpecId::PRAGUE | SpecId::PRAGUE_EOF | SpecId::LATEST) => {
                Self::new(granite())
            }
        }
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &precompile::Address,
        bytes: &precompile::Bytes,
        gas_limit: u64,
    ) -> Result<Option<revm::interpreter::InterpreterResult>, Self::Error> {
        self.precompile_provider
            .run(context, address, bytes, gas_limit)
    }

    #[inline]
    fn warm_addresses(&self) -> impl Iterator<Item = precompile::Address> {
        self.precompile_provider.warm_addresses()
    }

    #[inline]
    fn contains(&self, address: &precompile::Address) -> bool {
        self.precompile_provider.contains(address)
    }
}
