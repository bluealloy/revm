use super::{BundleAccount, RevertAccountState};
use crate::BlockState;
use revm_interpreter::primitives::hash_map;

// TODO
#[derive(Clone, Debug, Default)]
pub struct BundleState {
    /// State
    pub state: BlockState,
    /// Changes to revert
    pub change: Vec<Vec<BundleAccount>>,
}

impl BundleState {
    pub fn apply_block_substate_and_create_reverts(
        &mut self,
        block_state: BlockState,
    ) -> Vec<RevertAccountState> {
        let reverts = Vec::new();
        for (address, _block_account) in block_state.accounts.into_iter() {
            match self.state.accounts.entry(address) {
                hash_map::Entry::Occupied(entry) => {
                    let _this_account = entry.get();
                }
                hash_map::Entry::Vacant(_entry) => {
                    // TODO what to set here, just update i guess
                }
            }
        }
        reverts
    }
}
