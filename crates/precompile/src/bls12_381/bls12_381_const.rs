use crate::PrecompileWithAddress;

use crate::bls12_381::g1_add;
use crate::bls12_381::g1_msm;
use crate::bls12_381::g2_add;
use crate::bls12_381::g2_msm;
use crate::bls12_381::pairing;
use crate::bls12_381::map_fp_to_g1;
use crate::bls12_381::map_fp2_to_g2;

pub const G1_ADD_ADDRESS: u64 = 0x0b;
pub const G1_ADD_BASE_GAS_FEE: u64 = 375;
pub const G1_ADD_INPUT_LENGTH: usize = 256;
pub const G1_MSM_ADDRESS: u64=0x0c;
pub const G1_MSM_BASE_GAS_FEE: u64 = 1200;
pub const G1_MSM_INPUT_LENGTH: usize = 160;
pub const G1_OUTPUT_LENGTH: usize = 128;
pub const G1_INPUT_ITEM_LENGTH: usize = 128;
pub const G2_ADD_ADDRESS: u64 = 0x0d;
pub const G2_ADD_BASE_GAS_FEE: u64 = 600;
pub const G2_ADD_INPUT_LENGTH: usize = 512;
pub const G2_MSM_ADDRESS: u64 = 0x0e;
pub const G2_MSM_BASE_GAS_FEE: u64 = 22500;
pub const G2_MSM_INPUT_LENGTH: usize = 288;
pub const G2_OUTPUT_LENGTH: usize = 256;
pub const G2_INPUT_ITEM_LENGTH: usize = 256;
pub const PAIRING_ADDRESS: u64 = 0x0f;
pub const PAIRING_PAIRING_MULTIPLIER_BAS: u64 = 32600;
pub const PAIRING_PAIRING_OFFSET_BASE: u64 = 37700;
pub const PAIRING_INPUT_LENGTH: usize = 384;
pub const MAP_FP_TO_G1_ADDRESS: u64 = 0x10;
pub const MAP_FP_TO_G1_BASE_GAS_FEE: u64 = 5500;
pub const MAP_FP2_TO_G2_ADDRESS: u64 = 0x11;
pub const MAP_FP2_TO_G2_BASE_GAS_FEE: u64 = 0x23800;
pub const MSM_MULTIPLIER: u64 = 1000;
/// Number of bits used in the BLS12-381 curve finite field elements.
pub const UTILS_NBITS: usize = 256;
/// Finite field element input length.
pub const UTILS_FP_LENGTH: usize = 48;
/// Finite field element padded input length.
pub const UTILS_PADDED_FP_LENGTH: usize = 64;
/// Quadratic extension of finite field element input length.
pub const UTILS_PADDED_FP2_LENGTH: usize = 128;
/// Input elements padding length.
pub const UTILS_PADDING_LENGTH: usize = 16;
/// Scalar length.
pub const UTILS_SCALAR_LENGTH: usize = 32;
// Big-endian non-Montgomery form.
pub const UTILS_MODULUS_REPR: [u8; 48] = [
    0x1a, 0x01, 0x11, 0xea, 0x39, 0x7f, 0xe6, 0x9a, 0x4b, 0x1b, 0xa7, 0xb6, 0x43, 0x4b, 0xac, 0xd7,
    0x64, 0x77, 0x4b, 0x84, 0xf3, 0x85, 0x12, 0xbf, 0x67, 0x30, 0xd2, 0xa0, 0xf6, 0xb0, 0xf6, 0x24,
    0x1e, 0xab, 0xff, 0xfe, 0xb1, 0x53, 0xff, 0xff, 0xb9, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xab,
];

#[cfg(feature = "blst")]
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {

    [
        g1_add::PRECOMPILE,
        g1_msm::PRECOMPILE,
        g2_add::PRECOMPILE,
        g2_msm::PRECOMPILE,
        pairing::PRECOMPILE,
        map_fp_to_g1::PRECOMPILE,
        map_fp2_to_g2::PRECOMPILE,
    ]
    .into_iter()
}
