use once_cell::race::OnceBox;
use revm::precompile::{secp256r1, Precompiles};

/// Returns precompiles for Fjord spec.
pub fn fjord() -> &'static Precompiles {
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
pub fn granite() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = fjord().clone();

        precompiles.extend([
            // Restrict bn256Pairing input size
            crate::bn128::pair::GRANITE,
        ]);

        Box::new(precompiles)
    })
}
