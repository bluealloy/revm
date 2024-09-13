//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod bn128;
mod env;
mod fast_lz;
mod handler_register;
mod l1block;
mod result;
mod spec;

pub use handler_register::{
    deduct_caller, end, last_frame_return, load_accounts, load_precompiles,
    optimism_handle_register, output, refund, reward_beneficiary, validate_env,
    validate_tx_against_state,
};
pub use l1block::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};
pub use result::{OptimismHaltReason, OptimismInvalidTransaction};
use revm::{
    primitives::{Bytes, B256},
    wiring::TransactionValidation,
};
pub use spec::*;

pub trait OptimismContext {
    /// A reference to the cached L1 block info.
    fn l1_block_info(&self) -> Option<&L1BlockInfo>;

    /// A mutable reference to the cached L1 block info.
    fn l1_block_info_mut(&mut self) -> &mut Option<L1BlockInfo>;
}

/// Trait for an Optimism transaction.
pub trait OptimismTransaction {
    /// The source hash is used to make sure that deposit transactions do
    /// not have identical hashes.
    ///
    /// L1 originated deposit transaction source hashes are computed using
    /// the hash of the l1 block hash and the l1 log index.
    /// L1 attributes deposit source hashes are computed with the l1 block
    /// hash and the sequence number = l2 block number - l2 epoch start
    /// block number.
    ///
    /// These two deposit transaction sources specify a domain in the outer
    /// hash so there are no collisions.
    fn source_hash(&self) -> Option<&B256>;
    /// The amount to increase the balance of the `from` account as part of
    /// a deposit transaction. This is unconditional and is applied to the
    /// `from` account even if the deposit transaction fails since
    /// the deposit is pre-paid on L1.
    fn mint(&self) -> Option<&u128>;
    /// Whether or not the transaction is a system transaction.
    fn is_system_transaction(&self) -> Option<bool>;
    /// An enveloped EIP-2718 typed transaction. This is used
    /// to compute the L1 tx cost using the L1 block info, as
    /// opposed to requiring downstream apps to compute the cost
    /// externally.
    fn enveloped_tx(&self) -> Option<Bytes>;
}

/// Trait for an Optimism chain spec.
pub trait OptimismWiring:
    revm::EvmWiring<
    ChainContext: OptimismContext,
    Hardfork = OptimismSpecId,
    HaltReason = OptimismHaltReason,
    Transaction: OptimismTransaction
                     + TransactionValidation<ValidationError = OptimismInvalidTransaction>,
>
{
}

impl<EvmWiringT> OptimismWiring for EvmWiringT where
    EvmWiringT: revm::EvmWiring<
        ChainContext: OptimismContext,
        Hardfork = OptimismSpecId,
        HaltReason = OptimismHaltReason,
        Transaction: OptimismTransaction
                         + TransactionValidation<ValidationError = OptimismInvalidTransaction>,
    >
{
}
