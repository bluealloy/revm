//! # EIP-7212 secp256r1 Precompile
//!
//! This module implements the [EIP-7212](https://eips.ethereum.org/EIPS/eip-7212) precompile for
//! secp256r1 curve support.
//!
//! The main purpose of this precompile is to verify ECDSA signatures that use the secp256r1, or
//! P256 elliptic curve. The [`P256VERIFY`] const represents the implementation of this precompile,
//! with the address that it is currently deployed at.
use crate::{u64_to_address, Precompile, PrecompileWithAddress};
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use revm_primitives::{Bytes, PrecompileError, PrecompileResult, B256};

/// Base gas fee for secp256r1 p256verify operation.
const P256VERIFY_BASE: u64 = 3450;

/// Returns the secp256r1 precompile with its address.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    [P256VERIFY].into_iter()
}

/// [EIP-7212](https://eips.ethereum.org/EIPS/eip-7212#specification) secp256r1 precompile.
pub const P256VERIFY: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(0x100), Precompile::Standard(p256_verify));

/// secp256r1 precompile logic. It takes the input bytes sent to the precompile
/// and the gas limit. The output represents the result of verifying the
/// secp256r1 signature of the input.
///
/// The input is encoded as follows:
///
/// | signed message hash |  r  |  s  | public key x | public key y |
/// | :-----------------: | :-: | :-: | :----------: | :----------: |
/// |          32         | 32  | 32  |     32       |      32      |
pub fn p256_verify(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if P256VERIFY_BASE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }
    let result = if verify_impl(input).is_some() {
        B256::with_last_byte(1).into()
    } else {
        Bytes::new()
    };
    Ok((P256VERIFY_BASE, result))
}

/// Returns `Some(())` if the signature included in the input byte slice is
/// valid, `None` otherwise.
pub fn verify_impl(input: &[u8]) -> Option<()> {
    if input.len() != 160 {
        return None;
    }

    // msg signed (msg is already the hash of the original message)
    let msg = &input[..32];
    // r, s: signature
    let sig = &input[32..96];
    // x, y: public key
    let pk = &input[96..160];

    // prepend 0x04 to the public key: uncompressed form
    let mut uncompressed_pk = [0u8; 65];
    uncompressed_pk[0] = 0x04;
    uncompressed_pk[1..].copy_from_slice(pk);

    // Can fail only if the input is not exact length.
    let signature = Signature::from_slice(sig).ok()?;
    // Can fail if the input is not valid, so we have to propagate the error.
    let public_key = VerifyingKey::from_sec1_bytes(&uncompressed_pk).ok()?;

    public_key.verify_prehash(msg, &signature).ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use revm_primitives::hex::FromHex;
    use rstest::rstest;

    #[rstest]
    // test vectors from https://github.com/daimo-eth/p256-verifier/tree/master/test-vectors
    #[case::ok_1("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", true)]
    #[case::ok_2("3fec5769b5cf4e310a7d150508e82fb8e3eda1c2c94c61492d3bd8aea99e06c9e22466e928fdccef0de49e3503d2657d00494a00e764fd437bdafa05f5922b1fbbb77c6817ccf50748419477e843d5bac67e6a70e97dde5a57e0c983b777e1ad31a80482dadf89de6302b1988c82c29544c9c07bb910596158f6062517eb089a2f54c9a0f348752950094d3228d3b940258c75fe2a413cb70baa21dc2e352fc5", true)]
    #[case::ok_3("e775723953ead4a90411a02908fd1a629db584bc600664c609061f221ef6bf7c440066c8626b49daaa7bf2bcc0b74be4f7a1e3dcf0e869f1542fe821498cbf2de73ad398194129f635de4424a07ca715838aefe8fe69d1a391cfa70470795a80dd056866e6e1125aff94413921880c437c9e2570a28ced7267c8beef7e9b2d8d1547d76dfcf4bee592f5fefe10ddfb6aeb0991c5b9dbbee6ec80d11b17c0eb1a", true)]
    #[case::ok_4("b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", true)]
    #[case::ok_5("858b991cfd78f16537fe6d1f4afd10273384db08bdfc843562a22b0626766686f6aec8247599f40bfe01bec0e0ecf17b4319559022d4d9bf007fe929943004eb4866760dedf31b7c691f5ce665f8aae0bda895c23595c834fecc2390a5bcc203b04afcacbb4280713287a2d0c37e23f7513fab898f2c1fefa00ec09a924c335d9b629f1d4fb71901c3e59611afbfea354d101324e894c788d1c01f00b3c251b2", true)]
    #[case::fail_wrong_msg_1("3cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", false)]
    #[case::fail_wrong_msg_2("afec5769b5cf4e310a7d150508e82fb8e3eda1c2c94c61492d3bd8aea99e06c9e22466e928fdccef0de49e3503d2657d00494a00e764fd437bdafa05f5922b1fbbb77c6817ccf50748419477e843d5bac67e6a70e97dde5a57e0c983b777e1ad31a80482dadf89de6302b1988c82c29544c9c07bb910596158f6062517eb089a2f54c9a0f348752950094d3228d3b940258c75fe2a413cb70baa21dc2e352fc5", false)]
    #[case::fail_wrong_msg_3("f775723953ead4a90411a02908fd1a629db584bc600664c609061f221ef6bf7c440066c8626b49daaa7bf2bcc0b74be4f7a1e3dcf0e869f1542fe821498cbf2de73ad398194129f635de4424a07ca715838aefe8fe69d1a391cfa70470795a80dd056866e6e1125aff94413921880c437c9e2570a28ced7267c8beef7e9b2d8d1547d76dfcf4bee592f5fefe10ddfb6aeb0991c5b9dbbee6ec80d11b17c0eb1a", false)]
    #[case::fail_wrong_msg_4("c5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", false)]
    #[case::fail_wrong_msg_5("958b991cfd78f16537fe6d1f4afd10273384db08bdfc843562a22b0626766686f6aec8247599f40bfe01bec0e0ecf17b4319559022d4d9bf007fe929943004eb4866760dedf31b7c691f5ce665f8aae0bda895c23595c834fecc2390a5bcc203b04afcacbb4280713287a2d0c37e23f7513fab898f2c1fefa00ec09a924c335d9b629f1d4fb71901c3e59611afbfea354d101324e894c788d1c01f00b3c251b2", false)]
    #[case::fail_short_input_1("4cee90eb86eaa050036147a12d49004b6a", false)]
    #[case::fail_short_input_2("4cee90eb86eaa050036147a12d49004b6a958b991cfd78f16537fe6d1f4afd10273384db08bdfc843562a22b0626766686f6aec8247599f40bfe01bec0e0ecf17b4319559022d4d9bf007fe929943004eb4866760dedf319", false)]
    #[case::fail_long_input("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e00", false)]
    #[case::fail_invalid_sig("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff4aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", false)]
    #[case::fail_invalid_pubkey("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", false)]
    fn test_sig_verify(#[case] input: &str, #[case] expect_success: bool) {
        let input = Bytes::from_hex(input).unwrap();
        let target_gas = 3_500u64;
        let (gas_used, res) = p256_verify(&input, target_gas).unwrap();
        assert_eq!(gas_used, 3_450u64);
        let expected_result = if expect_success {
            B256::with_last_byte(1).into()
        } else {
            Bytes::new()
        };
        assert_eq!(res, expected_result);
    }

    #[rstest]
    fn test_not_enough_gas_errors() {
        let input = Bytes::from_hex("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e").unwrap();
        let target_gas = 2_500u64;
        let result = p256_verify(&input, target_gas);

        assert!(result.is_err());
        assert_eq!(result.err(), Some(PrecompileError::OutOfGas));
    }

    #[rstest]
    #[case::ok_1("b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", true)]
    #[case::fail_1("b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2daaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", false)]
    fn test_verify_impl(#[case] input: &str, #[case] expect_success: bool) {
        let input = Bytes::from_hex(input).unwrap();
        let result = verify_impl(&input);

        assert_eq!(result.is_some(), expect_success);
    }
}
