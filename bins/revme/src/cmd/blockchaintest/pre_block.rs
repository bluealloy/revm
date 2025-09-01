//! Pre block state transition

use database::State;
use primitives::hardfork::SpecId;
use revm::Database;
use statetest_types::blockchain::Withdrawal;

use crate::cmd::blockchaintest::post_block::ONE_WEI;

pub fn pre_block_transition<DB: Database>(
    state: &mut State<DB>,
    spec: SpecId,
    withdrawals: &[Withdrawal],
) {
    // withdrawals
    if spec.is_enabled_in(SpecId::SHANGHAI) {
        for withdrawal in withdrawals {
            state
                .increment_balances(vec![(
                    withdrawal.address,
                    withdrawal.amount.to::<u128>().saturating_mul(ONE_WEI),
                )])
                .expect("Db actions to pass");
        }
    }
}
