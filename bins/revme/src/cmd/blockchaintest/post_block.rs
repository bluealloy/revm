use context::Block;
use database::State;
use primitives::{hardfork::SpecId, Address};
use revm::Database;

pub const ONE_WEI: u128 = 1_000_000_000_000_000_000;

/// Block reward for a block.
#[inline]
pub const fn block_reward(spec: SpecId, ommers: usize) -> u128 {
    if spec.is_enabled_in(SpecId::MERGE) {
        return 0;
    }

    let reward = if spec.is_enabled_in(SpecId::CONSTANTINOPLE) {
        ONE_WEI * 2
    } else if spec.is_enabled_in(SpecId::BYZANTIUM) {
        ONE_WEI * 3
    } else {
        ONE_WEI * 5
    };

    reward + (reward >> 5) * ommers as u128
}

/// Post block transition that includes:
///   * Block and uncle rewards before the Merge/Paris hardfork.
///   * system calls
#[inline]
pub fn post_block_transition<DB: Database>(
    state: &mut State<DB>,
    block: impl Block,
    ommers: &[Address],
    spec: SpecId,
) {
    // block reward
    let block_reward = block_reward(spec, ommers.len());
    if block_reward != 0 {
        let _ = state.increment_balances(vec![(block.beneficiary(), block_reward)]);
    }
}
