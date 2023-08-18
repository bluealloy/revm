use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn};
use hd_keys_ecdsa::*;

pub const FUN: PrecompileAddress = PrecompileAddress(
    crate::u64_to_address(4),
    Precompile::Standard(derive_key as StandardPrecompileFn),
);

/// The base cost of the operation.
const IDENTITY_BASE: u64 = 15;
/// The cost per word.
const IDENTITY_PER_WORD: u64 = 3;

fn derive_cait_sith_pubkey(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let gas_used = calc_linear_cost_u32(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
    if gas_used > gas_limit {
        return Err(Error::OutOfGas);
    }

    // parse input

    // convert input to projectivepoint
    let root_public_keys: [C::ProjectivePoint] =  

    let deriver = HdKeyDeriver::<Secp256k1>::new(
        b"cait-sith-id",
        b"LIT_HD_KEY_ID_K256_XMD:SHA-256_SSWU_RO_NUL_",
    )
    .unwrap();
    let public = deriver.compute_public_key(&root_public_keys[..i]);

    Ok((gas_used, public.to_vec()))
}
