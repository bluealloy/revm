use context::Block;
use database::State;
use primitives::{hardfork::SpecId, ONE_ETHER};
use revm::Database;

/// Post block transition that includes:
///   * Block and uncle rewards before the Merge/Paris hardfork.
///   * system calls
///
/// # Note
///
/// Uncle rewards are not implemented yet.
#[inline]
pub fn post_block_transition<DB: Database>(state: &mut State<DB>, block: impl Block, spec: SpecId) {
    // block reward
    let block_reward = block_reward(spec, 0);
    if block_reward != 0 {
        let _ = state.increment_balances(vec![(block.beneficiary(), block_reward)]);
    }
}

/// Block reward for a block.
#[inline]
pub const fn block_reward(spec: SpecId, ommers: usize) -> u128 {
    if spec.is_enabled_in(SpecId::MERGE) {
        return 0;
    }

    let reward = if spec.is_enabled_in(SpecId::CONSTANTINOPLE) {
        ONE_ETHER * 2
    } else if spec.is_enabled_in(SpecId::BYZANTIUM) {
        ONE_ETHER * 3
    } else {
        ONE_ETHER * 5
    };

    reward + (reward >> 5) * ommers as u128
}
