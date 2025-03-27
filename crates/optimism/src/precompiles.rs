use crate::OpSpecId;
use once_cell::race::OnceBox;
use revm::{
    context::Cfg,
    context_interface::ContextTr,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::InterpreterResult,
    precompile::{
        self, bn128, secp256r1, PrecompileError, Precompiles,
        {PrecompileResult, PrecompileWithAddress},
    },
    primitives::{Address, Bytes},
};
use std::boxed::Box;
use std::string::String;

// Optimism precompile provider
#[derive(Debug, Clone)]
pub struct OpPrecompiles {
    /// Inner precompile provider is same as Ethereums.
    inner: EthPrecompiles,
}

impl OpPrecompiles {
    /// Create a new [`OpPrecompiles`] with the given precompiles.
    pub fn new(precompiles: &'static Precompiles) -> Self {
        Self {
            inner: EthPrecompiles { precompiles },
        }
    }

    /// Create a new precompile provider with the given optimismispec.
    #[inline]
    pub fn new_with_spec(spec: OpSpecId) -> Self {
        match spec {
            spec @ (OpSpecId::BEDROCK
            | OpSpecId::REGOLITH
            | OpSpecId::CANYON
            | OpSpecId::ECOTONE) => Self::new(Precompiles::new(spec.into_eth_spec().into())),
            OpSpecId::FJORD => Self::new(fjord()),
            OpSpecId::GRANITE | OpSpecId::HOLOCENE => Self::new(granite()),
            OpSpecId::ISTHMUS | OpSpecId::INTEROP | OpSpecId::OSAKA => Self::new(isthmus()),
        }
    }
}

/// Returns precompiles for Fjord spec.
pub fn fjord() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::cancun().clone();
        // EIP-7212: secp256r1 P256verify
        precompiles.extend([secp256r1::P256VERIFY]);
        Box::new(precompiles)
    })
}

/// Returns precompiles for Granite spec.
pub fn granite() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = fjord().clone();
        // Restrict bn256Pairing input size
        precompiles.extend([bn128_pair::GRANITE]);
        Box::new(precompiles)
    })
}

/// Returns precompiles for isthumus spec.
pub fn isthmus() -> &'static Precompiles {
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
        #[cfg(not(feature = "blst"))]
        let precompiles = {
            let mut precompiles = precompiles;
            precompiles.extend(precompile::bls12_381_utils::bls12_381_precompiles_not_supported());
            precompiles
        };
        Box::new(precompiles)
    })
}

impl<CTX> PrecompileProvider<CTX> for OpPrecompiles
where
    CTX: ContextTr<Cfg: Cfg<Spec = OpSpecId>>,
{
    type Output = InterpreterResult;

    #[inline]
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) {
        *self = Self::new_with_spec(spec);
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        self.inner.run(context, address, bytes, gas_limit)
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

impl Default for OpPrecompiles {
    fn default() -> Self {
        Self::new_with_spec(OpSpecId::ISTHMUS)
    }
}

pub mod bn128_pair {
    use super::*;

    pub const GRANITE_MAX_INPUT_SIZE: usize = 112687;
    pub const GRANITE: PrecompileWithAddress =
        PrecompileWithAddress(bn128::pair::ADDRESS, |input, gas_limit| {
            run_pair(input, gas_limit)
        });

    pub fn run_pair(input: &[u8], gas_limit: u64) -> PrecompileResult {
        if input.len() > GRANITE_MAX_INPUT_SIZE {
            return Err(PrecompileError::Bn128PairLength);
        }
        bn128::run_pair(
            input,
            bn128::pair::ISTANBUL_PAIR_PER_POINT,
            bn128::pair::ISTANBUL_PAIR_BASE,
            gas_limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::{precompile::PrecompileError, primitives::hex};
    use std::vec;

    #[test]
    fn test_bn128_pair() {
        let input = hex::decode(
            "\
      1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
      3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
      209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
      04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
      2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
      120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
      111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
      2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
      198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
      1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
      090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
      12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        )
        .unwrap();
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let outcome = bn128_pair::run_pair(&input, 260_000).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Invalid input length
        let input = hex::decode(
            "\
          1111111111111111111111111111111111111111111111111111111111111111\
          1111111111111111111111111111111111111111111111111111111111111111\
          111111111111111111111111111111\
      ",
        )
        .unwrap();

        let res = bn128_pair::run_pair(&input, 260_000);
        assert!(matches!(res, Err(PrecompileError::Bn128PairLength)));

        // Valid input length shorter than 112687
        let input = vec![1u8; 586 * bn128::PAIR_ELEMENT_LEN];
        let res = bn128_pair::run_pair(&input, 260_000);
        assert!(matches!(res, Err(PrecompileError::OutOfGas)));

        // Input length longer than 112687
        let input = vec![1u8; 587 * bn128::PAIR_ELEMENT_LEN];
        let res = bn128_pair::run_pair(&input, 260_000);
        assert!(matches!(res, Err(PrecompileError::Bn128PairLength)));
    }

    #[test]
    fn test_cancun_precompiles_in_fjord() {
        // additional to cancun, fjord has p256verify
        assert_eq!(fjord().difference(Precompiles::cancun()).len(), 1)
    }

    #[test]
    fn test_cancun_precompiles_in_granite() {
        // granite has p256verify (fjord)
        // granite has modification of cancun's bn128 pair (doesn't count as new precompile)
        assert_eq!(granite().difference(Precompiles::cancun()).len(), 1)
    }

    #[test]
    fn test_prague_precompiles_in_isthmus() {
        let new_prague_precompiles = Precompiles::prague().difference(Precompiles::cancun());

        // isthmus contains all precompiles that were new in prague, without modifications
        assert!(new_prague_precompiles.difference(isthmus()).is_empty())
    }

    #[test]
    fn test_default_precompiles_is_latest() {
        let latest = OpPrecompiles::new_with_spec(OpSpecId::default())
            .inner
            .precompiles;
        let default = OpPrecompiles::default().inner.precompiles;
        assert_eq!(latest.len(), default.len());

        let intersection = default.intersection(latest);
        assert_eq!(intersection.len(), latest.len())
    }
}
