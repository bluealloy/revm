use crate::MonadSpecId;
use revm::{
    handler::instructions::EthInstructions,
    interpreter::{
        gas::params::{GasId, GasParams},
        instructions::instruction_table_gas_changes_spec,
        interpreter::EthInterpreter,
        Host,
    },
};

/// Type alias for Monad instructions.
pub type MonadInstructions<CTX> = EthInstructions<EthInterpreter, CTX>;

/// Monad-specific gas parameters for a given hardfork.
/// Override Ethereum defaults with Monad's gas costs.
pub fn monad_gas_params(spec: MonadSpecId) -> GasParams {
    let eth_spec = spec.into_eth_spec();
    let mut params = GasParams::new_spec(eth_spec);

    if MonadSpecId::Monad.is_enabled_in(spec) {
        params.override_gas([
            (GasId::cold_storage_cost(), COLD_SLOAD_COST),
            (
                GasId::cold_account_additional_cost(),
                COLD_ACCOUNT_ACCESS_COST,
            ),
        ]);
    }

    params
}

// Create Monad instructions table with custom gas costs.
/// This function combines:
/// 1. Standard instruction table for the underlying Ethereum spec
/// 2. Monad-specific gas parameters for the hardfork
/// 3. Any custom Monad opcodes (future)
pub fn monad_instructions<CTX: Host>(spec: MonadSpecId) -> MonadInstructions<CTX> {
    let eth_spec = spec.into_eth_spec();
    let instructions = EthInstructions::new(
        instruction_table_gas_changes_spec(eth_spec),
        monad_gas_params(spec),
        eth_spec,
    );

    instructions
}

/// Override gas cost for cold storage
pub const COLD_SLOAD_COST: u64 = 8100;
/// Override gas cost for cold account access
pub const COLD_ACCOUNT_ACCESS_COST: u64 = 10100;
