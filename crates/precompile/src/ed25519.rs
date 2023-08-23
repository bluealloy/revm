use crate::{CustomPrecompileFn, Error, Precompile, PrecompileAddress, PrecompileResult};

pub const ED25519_VERIFY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(10),
    Precompile::Custom(ed25519_verify_run as CustomPrecompileFn),
);

// #[cfg(feature = "ed25519")]
#[allow(clippy::module_inception)]
mod ed25519 {
    use super::*;
    use core::cmp::min;
    use crypto::ed25519::verify;

    // TODO: since we know what the light of the field should be , we should probably enforce the size of the array
    // i.e. data: &[u8 , 128]
    pub fn ed25519_verify(data: &[u8]) -> Result<bool, Error> {
        let len = min(data.len(), 128);

        let mut input = [0u8; 128];
        input[..len].copy_from_slice(&data[..len]);

        let mut buf = [0u8; 4];

        if verify(&input[0..32], &input[32..64], &input[64..128]) {
            buf[3] = 0u8;
        } else {
            buf[3] = 1u8;
        };

        if buf[3] == 0u8 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// reference: https://docs.rs/rust-crypto/0.2.36/crypto/ed25519/fn.verify.html
/// input format:
/// [32 bytes for msg][32 bytes for public key][64 bytes for signature]
fn ed25519_verify_run(input: &[u8], target_gas: u64) -> PrecompileResult {
    // TODO: Use a more sane value for this
    const ED25519_VERIFY_BASE: u64 = 3_000;

    if ED25519_VERIFY_BASE > target_gas {
        return Err(Error::OutOfGas);
    }

    match ed25519::ed25519_verify(input) {
        Ok(true) => Ok((target_gas, vec![0u8])),
        Ok(false) => Ok((target_gas, vec![1u8])),
        Err(e) => Err(e),
    }
}

// We need to unit test here , as the test provided in revm crate are ethereum state tests
// which dont account for custom precompiles.
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use crypto::ed25519::keypair;
    use crypto::ed25519::signature;
    use rand_core::RngCore;

    #[test]
    fn test_ed25519_verify_valid() {
        let seed = generate_random_seed();
        let (secret, public_key) = keypair(&seed);
        let message = b"test message";
        let message_padded = pad_message_to_32_bytes(message);
        let sig = signature(&message_padded, &secret);

        let mut input = Vec::new();
        input.extend_from_slice(&message_padded);
        input.extend_from_slice(&public_key[..]);
        input.extend_from_slice(&sig[..]);

        assert_eq!(ed25519::ed25519_verify(&input).unwrap(), true);
    }

    #[test]
    fn test_ed25519_verify_invalid() {
        let seed = generate_random_seed();
        let (secret, public_key) = keypair(&seed);
        let message = b"test message";
        let sig = signature(message, &secret);

        let mut input = Vec::new();
        input.extend_from_slice(b"wrong message");
        input.extend_from_slice(&public_key[..]);
        input.extend_from_slice(&sig[..]);

        assert_eq!(ed25519::ed25519_verify(&input).unwrap(), false);
    }

    #[test]
    fn test_ed25519_verify_invalid_signature() {
        let seed = generate_random_seed();
        let (_, public_key) = keypair(&seed);
        let message = b"test message";
        let message_padded = pad_message_to_32_bytes(message);

        let mut input = Vec::new();
        input.extend_from_slice(&message_padded);
        input.extend_from_slice(&public_key[..]);
        input.extend_from_slice(&[0u8; 64]);

        assert_eq!(ed25519::ed25519_verify(&input).unwrap(), false);
    }

    fn generate_random_seed() -> [u8; 32] {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        seed
    }

    fn pad_message_to_32_bytes(message: &[u8]) -> Vec<u8> {
        let mut message_padded = vec![0; 32];
        for (i, byte) in message.iter().enumerate() {
            message_padded[i] = *byte;
        }
        message_padded
    }
}
