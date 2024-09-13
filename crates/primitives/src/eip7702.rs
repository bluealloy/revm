pub mod authorization_list;
pub mod bytecode;

use crate::U256;
pub use authorization_list::{
    Authorization, AuthorizationList, InvalidAuthorization, RecoveredAuthorization, Signature,
    SignedAuthorization,
};
pub use bytecode::{
    Eip7702Bytecode, Eip7702DecodeError, EIP7702_MAGIC, EIP7702_MAGIC_BYTES, EIP7702_VERSION,
};

// Base cost of updating authorized account.
pub const PER_AUTH_BASE_COST: u64 = 2500;

/// Cost of creating authorized account that was previously empty.
pub const PER_EMPTY_ACCOUNT_COST: u64 = 25000;

/// The order of the secp256k1 curve, divided by two. Signatures that should be checked according
/// to EIP-2 should have an S value less than or equal to this.
///
/// `57896044618658097711785492504343953926418782139537452191302581570759080747168`
const SECP256K1N_HALF: U256 = U256::from_be_bytes([
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D, 0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
]);
