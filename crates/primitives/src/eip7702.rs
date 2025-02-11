pub mod authorization_list;
pub mod bytecode;

pub use authorization_list::{
    Authorization, AuthorizationList, PrimitiveSignature, RecoveredAuthority,
    RecoveredAuthorization, SignedAuthorization,
};
pub use bytecode::{
    Eip7702Bytecode, Eip7702DecodeError, EIP7702_MAGIC, EIP7702_MAGIC_BYTES, EIP7702_MAGIC_HASH,
    EIP7702_VERSION,
};

// Base cost of updating authorized account.
pub const PER_AUTH_BASE_COST: u64 = 12500;

/// Cost of creating authorized account that was previously empty.
pub const PER_EMPTY_ACCOUNT_COST: u64 = 25000;
