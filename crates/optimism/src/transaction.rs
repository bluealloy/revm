use revm::wiring::{default::Env, Transaction};

pub mod abstraction;
pub mod deposit;
pub mod error;

pub use abstraction::{OpTransaction, OpTransactionType, OpTxTrait};
