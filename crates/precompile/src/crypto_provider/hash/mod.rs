//! Hash cryptographic implementations for the crypto provider

/// SHA-256 hash implementation
pub fn sha256(input: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(input).into()
}

/// RIPEMD-160 hash implementation
pub fn ripemd160(input: &[u8]) -> [u8; 20] {
    use ripemd::Digest;
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(input);
    hasher.finalize().into()
}
