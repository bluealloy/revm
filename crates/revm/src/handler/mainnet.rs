//! Mainnet related handlers.

mod execution;
mod post_execution;
mod pre_execution;
mod validation;

// Public exports

pub use execution::{
    call, call_return, create, create_return, eofcreate, eofcreate_return, execute_frame,
    insert_call_outcome, insert_create_outcome, insert_eofcreate_outcome, last_frame_return,
};
pub use post_execution::{clear, end, output, refund, reimburse_caller, reward_beneficiary};
pub use pre_execution::{
    apply_eip7702_auth_list, deduct_caller, deduct_caller_inner, load_accounts, load_precompiles,
};
pub use validation::{validate_env, validate_initial_tx_gas, validate_tx_against_state};
