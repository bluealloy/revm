use super::{AccountRevert, BundleAccount, TransitionState};
use revm_interpreter::primitives::{hash_map, HashMap, B160};

// TODO
#[derive(Clone, Debug, Default)]
pub struct BundleState {
    /// State
    pub state: HashMap<B160, BundleAccount>,
    // TODO contracts etc.
    /// Changes to revert
    pub change: Vec<Vec<BundleAccount>>,
}

impl BundleState {
    pub fn apply_block_substate_and_create_reverts(
        &mut self,
        transitions: TransitionState,
    ) -> Vec<AccountRevert> {
        let reverts = Vec::new();
        for (address, _block_account) in transitions.accounts.into_iter() {
            match self.state.entry(address) {
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
