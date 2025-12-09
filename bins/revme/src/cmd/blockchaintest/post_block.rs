use revm::{
    context::{Block, ContextTr},
    database::{DatabaseCommitExt as _, State},
    handler::EvmTr,
    primitives::{hardfork::SpecId, ONE_ETHER, ONE_GWEI},
    Database, SystemCallCommitEvm,
};
use statetest_types::blockchain::Withdrawal;

/// Post block transition that includes:
///   * Block and uncle rewards before the Merge/Paris hardfork.
///   * system calls
///
/// # Note
///
/// Uncle rewards are not implemented yet.
#[inline]
pub fn post_block_transition<
    'a,
    DB: Database + 'a,
    EVM: SystemCallCommitEvm<Error: core::fmt::Debug>
        + EvmTr<Context: ContextTr<Db = &'a mut State<DB>>>,
>(
    evm: &mut EVM,
    block: impl Block,
    withdrawals: &[Withdrawal],
    spec: SpecId,
) {
    // block reward
    let block_reward = block_reward(spec, 0);
    if block_reward != 0 {
        let _ = evm
            .ctx_mut()
            .db_mut()
            .increment_balances(vec![(block.beneficiary(), block_reward)]);
    }

    // withdrawals
    if spec.is_enabled_in(SpecId::SHANGHAI) {
        for withdrawal in withdrawals {
            evm.ctx_mut()
                .db_mut()
                .increment_balances(vec![(
                    withdrawal.address,
                    withdrawal.amount.to::<u128>().saturating_mul(ONE_GWEI),
                )])
                .expect("Db actions to pass");
        }
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
