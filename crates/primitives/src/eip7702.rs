pub mod authorization_list;
pub mod bytecode;

pub use authorization_list::{
    Authorization, AuthorizationList, RecoveredAuthorization, Signature, SignedAuthorization,
};
pub use bytecode::{Eip7702Bytecode, EIP7702_MAGIC, EIP7702_MAGIC_BYTES};


// Base cost of updating authorized account.
pub const PER_AUTH_BASE_COST: u64 = 2500;

/// Cost of creating authorized account that was previously empty.
pub const PER_EMPTY_ACCOUNT_COST: u64 = 25000;
