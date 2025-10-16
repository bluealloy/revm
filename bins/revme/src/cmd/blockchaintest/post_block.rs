use context::{Block, ContextTr, JournalTr};
use database::State;
use primitives::{address, hardfork::SpecId, Address, Bytes, ONE_ETHER, ONE_GWEI, U256};
use revm::{handler::EvmTr, Database, SystemCallCommitEvm};
use statetest_types::blockchain::Withdrawal;

/// Post block transition that includes:
///   * Block and uncle rewards before the Merge/Paris hardfork.
///   * Withdrawals (EIP-4895)
///   * Post-block system calls: EIP-7002 (withdrawal requests) and EIP-7251 (consolidation requests)
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
            .journal_mut()
            .balance_incr(block.beneficiary(), U256::from(block_reward))
            .expect("Db actions to pass");
    }

    // withdrawals
    if spec.is_enabled_in(SpecId::SHANGHAI) {
        for withdrawal in withdrawals {
            let _ = evm
                .ctx_mut()
                .journal_mut()
                .balance_incr(
                    withdrawal.address,
                    withdrawal.amount.saturating_mul(U256::from(ONE_GWEI)),
                )
                .expect("Db actions to pass");
        }
    }

    // EIP-7002: Withdrawal requests system call
    if spec.is_enabled_in(SpecId::PRAGUE) {
        system_call_eip7002_withdrawal_request(evm);
    }

    // EIP-7251: Consolidation requests system call
    if spec.is_enabled_in(SpecId::PRAGUE) {
        system_call_eip7251_consolidation_request(evm);
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

pub const WITHDRAWAL_REQUEST_ADDRESS: Address =
    address!("0x00000961Ef480Eb55e80D19ad83579A64c007002");

/// EIP-7002: Withdrawal requests system call
pub(crate) fn system_call_eip7002_withdrawal_request(
    evm: &mut impl SystemCallCommitEvm<Error: core::fmt::Debug>,
) {
    // empty data is valid for EIP-7002
    let _ = match evm.system_call_commit(WITHDRAWAL_REQUEST_ADDRESS, Bytes::new()) {
        Ok(res) => res,
        Err(e) => {
            panic!("System call failed: {e:?}");
        }
    };
}

pub const CONSOLIDATION_REQUEST_ADDRESS: Address =
    address!("0x00431d76A6B8c7a6F8E4A2C1f9f08E3e3bA8C5f9");

/// EIP-7251: Consolidation requests system call
pub(crate) fn system_call_eip7251_consolidation_request(
    evm: &mut impl SystemCallCommitEvm<Error: core::fmt::Debug>,
) {
    let _ = match evm.system_call_commit(CONSOLIDATION_REQUEST_ADDRESS, Bytes::new()) {
        Ok(res) => res,
        Err(e) => {
            panic!("System call failed: {e:?}");
        }
    };
}
