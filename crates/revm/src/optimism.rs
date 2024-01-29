//! Optimism-specific constants, types, and helpers.

mod handler_register;
mod l1block;

pub use handler_register::{
    deduct_caller, end, last_frame_return, optimism_handle_register, output, reward_beneficiary,
};
pub use l1block::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};
