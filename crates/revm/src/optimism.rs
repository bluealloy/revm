//! Optimism-specific constants, types, and helpers.

mod env;
mod handler;
mod handler_register;
mod l1block;
mod result;
mod spec;

pub use handler_register::{
    deduct_caller, end, last_frame_return, optimism_handle_register, output, reward_beneficiary,
    validate_env, validate_tx_against_state,
};
pub use l1block::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};
pub use result::{InvalidOptimismTransaction, OptimismHaltReason};
pub use spec::*;
