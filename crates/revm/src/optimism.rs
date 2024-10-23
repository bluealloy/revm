//! Optimism-specific constants, types, and helpers.

mod bn128;
mod fast_lz;
mod handler_register;
mod l1block;

pub use handler_register::{
    deduct_caller, end, last_frame_return, load_accounts, load_precompiles,
    optimism_handle_register, output, refund, reward_beneficiary, validate_env,
    validate_tx_against_state,
};
pub use l1block::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};
