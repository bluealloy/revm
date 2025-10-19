//! Pre block state transition

use revm::{
    context::{Block, ContextTr},
    database::{Database, State},
    handler::EvmTr,
    primitives::{address, hardfork::SpecId, Address, B256},
    SystemCallCommitEvm,
};

/// Pre block state transition
///
/// # Note
///
/// Contains pre-block system calls: EIP-2935 (blockhash) and EIP-4788 (beacon root).
pub fn pre_block_transition<
    'a,
    DB: Database + 'a,
    EVM: SystemCallCommitEvm<Error: core::fmt::Debug>
        + EvmTr<Context: ContextTr<Db = &'a mut State<DB>>>,
>(
    evm: &mut EVM,
    spec: SpecId,
    parent_block_hash: Option<B256>,
    parent_beacon_block_root: Option<B256>,
) {
    // skip system calls for block number 0 (Gensis block)
    if evm.ctx().block().number() == 0 {
        return;
    }

    // blockhash system call
    if let Some(parent_block_hash) = parent_block_hash {
        if spec.is_enabled_in(SpecId::PRAGUE) {
            system_call_eip2935_blockhash(evm, parent_block_hash);
        }
    }

    if let Some(parent_beacon_block_root) = parent_beacon_block_root {
        if spec.is_enabled_in(SpecId::CANCUN) {
            system_call_eip4788_beacon_root(evm, parent_beacon_block_root);
        }
    }
}

pub const HISTORY_STORAGE_ADDRESS: Address = address!("0x0000F90827F1C53a10cb7A02335B175320002935");

/// Blockhash system callEIP-2935
#[inline]
pub(crate) fn system_call_eip2935_blockhash(
    evm: &mut impl SystemCallCommitEvm<Error: core::fmt::Debug>,
    parent_block_hash: B256,
) {
    let _ = match evm.system_call_commit(HISTORY_STORAGE_ADDRESS, parent_block_hash.0.into()) {
        Ok(res) => res,
        Err(e) => {
            panic!("System call failed: {e:?}");
        }
    };
}

pub const BEACON_ROOTS_ADDRESS: Address = address!("000F3df6D732807Ef1319fB7B8bB8522d0Beac02");

/// Beacon root system call EIP-4788
pub(crate) fn system_call_eip4788_beacon_root(
    evm: &mut impl SystemCallCommitEvm<Error: core::fmt::Debug>,
    parent_beacon_block_root: B256,
) {
    let _ = match evm.system_call_commit(BEACON_ROOTS_ADDRESS, parent_beacon_block_root.0.into()) {
        Ok(res) => res,
        Err(e) => {
            panic!("System call failed: {e:?}");
        }
    };
}
