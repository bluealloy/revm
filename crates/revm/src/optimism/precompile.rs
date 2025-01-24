use once_cell::race::OnceBox;
#[cfg(feature = "blst")]
use revm_precompile::bls12_381;
use revm_precompile::{secp256r1, Precompiles};
use std::boxed::Box;

/// Returns precompiles for Fjord spec.
pub(crate) fn fjord() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::cancun().clone();

        precompiles.extend([
            // EIP-7212: secp256r1 P256verify
            secp256r1::P256VERIFY,
        ]);

        Box::new(precompiles)
    })
}

/// Returns precompiles for Granite spec.
pub(crate) fn granite() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = fjord().clone();

        precompiles.extend([
            // Restrict bn256Pairing input size
            crate::optimism::bn128::pair::GRANITE,
        ]);

        Box::new(precompiles)
    })
}

/// Returns precompiles for isthmus
pub(crate) fn isthmus() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let precompiles = Precompiles::cancun().clone();

        // Don't include BLS12-381 precompiles in no_std builds.
        #[cfg(feature = "blst")]
        let precompiles = {
            let mut precompiles = precompiles;
            precompiles.extend(bls12_381::precompiles());
            precompiles
        };

        Box::new(precompiles)
    })
}
