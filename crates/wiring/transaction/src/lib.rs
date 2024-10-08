//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

mod access_list;
mod common;
pub mod eip1559;
pub mod eip2930;
pub mod eip4844;
pub mod eip7702;
pub mod legacy;
pub mod transaction;
pub mod transaction_type;

pub use access_list::AccessListTrait;
pub use common::CommonTxFields;
pub use eip1559::{Eip1559CommonTxFields, Eip1559Tx};
pub use eip2930::Eip2930Tx;
pub use eip4844::Eip4844Tx;
pub use eip7702::Eip7702Tx;
pub use legacy::LegacyTx;
pub use transaction::{Transaction, TransactionError};
pub use transaction_type::TransactionType;
