#![warn(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;

mod context;
mod evm;
pub mod handler;
mod r#impl;

mod gas;
mod types;

pub use context::EVMData;
pub use evm::{evm_inner, new, RWASM};
pub use handler::Handler;
pub use r#impl::{EVMImpl, Transact, CALL_STACK_LIMIT};
// reexport `revm_primitives`
#[doc(inline)]
pub use revm_primitives as primitives;
