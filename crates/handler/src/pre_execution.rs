//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{EvmTr, PrecompileProvider};
use bytecode::Bytecode;
use context_interface::{
    journaled_state::{account::JournaledAccountTr, JournalCheckpoint, JournalTr},
    result::InvalidTransaction,
    transaction::{AccessListItemTr, AuthorizationTr, Transaction, TransactionType},
    Block, Cfg, ContextTr, Database,
};
use core::cmp::Ordering;
use interpreter::{GasTracker, InitialAndFloorGas};
use primitives::{hardfork::SpecId, Address, AddressMap, HashSet, StorageKey, TxKind, U256};
use state::AccountInfo;

/// Loads and warms accounts for execution, including precompiles and access list.
pub fn load_accounts<
    EVM: EvmTr<Precompiles: PrecompileProvider<EVM::Context>>,
    ERROR: From<<<EVM::Context as ContextTr>::Db as Database>::Error>,
>(
    evm: &mut EVM,
) -> Result<(), ERROR> {
    let (context, precompiles) = evm.ctx_precompiles();

    let gen_spec = context.cfg().spec();
    let spec = gen_spec.clone().into();
    // sets eth spec id in journal
    context.journal_mut().set_spec_id(spec);
    let precompiles_changed = precompiles.set_spec(gen_spec);
    let empty_warmed_precompiles = context.journal_mut().precompile_addresses().is_empty();

    if precompiles_changed || empty_warmed_precompiles {
        // load new precompile addresses into journal.
        // When precompiles addresses are changed we reset the warmed hashmap to those new addresses.
        context
            .journal_mut()
            .warm_precompiles(precompiles.warm_addresses());
    }

    // Load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if spec.is_enabled_in(SpecId::SHANGHAI) {
        let coinbase = context.block().beneficiary();
        context.journal_mut().warm_coinbase_account(coinbase);
    }

    // Load access list
    let (tx, journal) = context.tx_journal_mut();
    // legacy is only tx type that does not have access list.
    if tx.tx_type() != TransactionType::Legacy {
        if let Some(access_list) = tx.access_list() {
            let mut map: AddressMap<HashSet<StorageKey>> = AddressMap::default();
            for item in access_list {
                map.entry(*item.address())
                    .or_default()
                    .extend(item.storage_slots().map(|key| U256::from_be_bytes(key.0)));
            }
            journal.warm_access_list(map);
        }
    }

    Ok(())
}

/// Validates caller account nonce and code according to EIP-3607.
#[inline]
pub fn validate_account_nonce_and_code_with_components(
    caller_info: &AccountInfo,
    tx: impl Transaction,
    cfg: impl Cfg,
) -> Result<(), InvalidTransaction> {
    validate_account_nonce_and_code(
        caller_info,
        tx.nonce(),
        cfg.is_eip3607_disabled(),
        cfg.is_nonce_check_disabled(),
    )
}

/// Validates caller account nonce and code according to EIP-3607.
#[inline]
pub fn validate_account_nonce_and_code(
    caller_info: &AccountInfo,
    tx_nonce: u64,
    is_eip3607_disabled: bool,
    is_nonce_check_disabled: bool,
) -> Result<(), InvalidTransaction> {
    // EIP-3607: Reject transactions from senders with deployed code
    // This EIP is introduced after london but there was no collision in past
    // so we can leave it enabled always
    if !is_eip3607_disabled {
        let bytecode = match caller_info.code.as_ref() {
            Some(code) => code,
            None => &Bytecode::default(),
        };
        // Allow EOAs whose code is a valid delegation designation,
        // i.e. 0xef0100 || address, to continue to originate transactions.
        if !bytecode.is_empty() && !bytecode.is_eip7702() {
            return Err(InvalidTransaction::RejectCallerWithCode);
        }
    }

    // Check that the transaction's nonce is correct
    if !is_nonce_check_disabled {
        let tx = tx_nonce;
        let state = caller_info.nonce;
        if tx == u64::MAX && state == u64::MAX {
            return Err(InvalidTransaction::NonceOverflowInTransaction);
        }
        match tx.cmp(&state) {
            Ordering::Greater => {
                return Err(InvalidTransaction::NonceTooHigh { tx, state });
            }
            Ordering::Less => {
                return Err(InvalidTransaction::NonceTooLow { tx, state });
            }
            _ => {}
        }
    }
    Ok(())
}

/// Check maximum possible fee and deduct the effective fee.
///
/// Returns new balance.
#[inline]
pub fn calculate_caller_fee(
    balance: U256,
    tx: impl Transaction,
    block: impl Block,
    cfg: impl Cfg,
) -> Result<U256, InvalidTransaction> {
    // If fee charge is disabled, return the balance as-is without deducting fees.
    // This is useful for `eth_call` and similar simulation scenarios.
    if cfg.is_fee_charge_disabled() {
        return Ok(balance);
    }

    let basefee = block.basefee() as u128;
    let blob_price = block.blob_gasprice().unwrap_or_default();
    let is_balance_check_disabled = cfg.is_balance_check_disabled();

    if !is_balance_check_disabled {
        tx.ensure_enough_balance(balance)?;
    }

    let effective_balance_spending = tx
        .effective_balance_spending(basefee, blob_price)
        .expect("effective balance is always smaller than max balance so it can't overflow");

    let gas_balance_spending = effective_balance_spending - tx.value();

    // new balance
    let mut new_balance = balance.saturating_sub(gas_balance_spending);

    if is_balance_check_disabled {
        // Make sure the caller's balance is at least the value of the transaction.
        new_balance = new_balance.max(tx.value());
    }

    Ok(new_balance)
}

/// Validates caller state and deducts transaction costs from the caller's balance.
#[inline]
pub fn validate_against_state_and_deduct_caller<
    CTX: ContextTr,
    ERROR: From<InvalidTransaction> + From<<CTX::Db as Database>::Error>,
>(
    context: &mut CTX,
) -> Result<(), ERROR> {
    let (block, tx, cfg, journal, _, _) = context.all_mut();

    // Load caller's account.
    let mut caller = journal.load_account_with_code_mut(tx.caller())?.data;

    validate_account_nonce_and_code_with_components(&caller.account().info, tx, cfg)?;

    let new_balance = calculate_caller_fee(*caller.balance(), tx, block, cfg)?;

    caller.set_balance(new_balance);
    if tx.kind().is_call() {
        caller.bump_nonce();
    }
    Ok(())
}

/// Gas decisions made by the pre-execution phase, carried to the execution
/// phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreExecutionOutput {
    /// EIP-7702 regular gas refund for authorities that already existed.
    pub eip7702_refund: u64,
    /// Journal checkpoint opened by [`crate::Handler::pre_execution`] before
    /// the authorization list is applied, spanning the EIP-2780 runtime gas
    /// phase.
    ///
    /// It is left open because the runtime gas phase continues at first-frame
    /// creation (`create_init_frame` charges the recipient and create-target
    /// costs): [`crate::Handler::execution`] commits it once the first frame
    /// is created, or reverts it — dropping the applied delegations — when
    /// the frame-creation charges run out of gas. Pre-Amsterdam there are no
    /// runtime charges, so the checkpoint is always committed.
    pub checkpoint: JournalCheckpoint,
}

/// Apply EIP-7702 auth list and return number gas refund on already created accounts.
///
/// Note that this function will do nothing if the transaction type is not EIP-7702.
/// If you need to apply auth list for other transaction types, use [`apply_auth_list`] function.
///
/// Internally uses [`apply_auth_list`] function.
///
/// Under EIP-2780 the authorization charges are instead metered on the
/// transaction-level `gas` as the authorizations are applied
/// ([`apply_auth_list_eip2780`]) and no refund is returned (the pessimistic
/// intrinsic charge and its refund are replaced by conditional runtime
/// charges). Charging as the authorizations are applied makes the phase stop
/// at the first unaffordable charge: later authorities must not be loaded
/// (observable through the EIP-7928 block access list).
///
/// Returns the EIP-7702 gas refund, or `None` when the authorization charges
/// ran out of gas: the caller owns the runtime gas phase checkpoint and must
/// revert it, dropping the applied delegations; the transaction stays valid
/// but must be included as an out-of-gas halt without entering execution.
///
/// `init_and_floor_gas` is unused by this implementation — the EIP-2780
/// charges are recorded on the transaction-level `gas` — and is kept in the
/// signature so chain variants that meter the authorizations against the
/// intrinsic/floor gas can reuse this entry point.
#[inline]
pub fn apply_eip7702_auth_list<
    CTX: ContextTr,
    ERROR: From<InvalidTransaction> + From<<CTX::Db as Database>::Error>,
>(
    context: &mut CTX,
    _init_and_floor_gas: &mut InitialAndFloorGas,
    gas: &mut GasTracker,
) -> Result<Option<u64>, ERROR> {
    // EIP-2780: state-dependent charges (authority creation, delegation bytes,
    // delegation-target access, recipient new-account state gas) are charged at
    // the runtime phase instead of pessimistically at the intrinsic phase.
    if context.cfg().is_amsterdam_eip2780_enabled() {
        if context.tx().tx_type() != TransactionType::Eip7702 {
            return Ok(Some(0));
        }
        let chain_id = context.cfg().chain_id();
        let is_eip8037 = context.cfg().is_amsterdam_eip8037_enabled();
        let params = context.cfg().gas_params();
        let account_write_cost = params.tx_account_write_cost();
        let new_account_state_gas = if is_eip8037 {
            params.new_account_state_gas()
        } else {
            0
        };
        let delegation_bytes_state_gas = if is_eip8037 {
            params.tx_eip7702_state_gas_bytecode()
        } else {
            0
        };
        let (tx, journal) = context.tx_journal_mut();

        // Accounts this transaction has already written (their `ACCOUNT_WRITE`
        // is already paid): the sender's leaf is written at inclusion (priced
        // into `TX_BASE`), and the recipient's when value is transferred
        // (priced into `TX_VALUE_COST`).
        let mut written_accounts: HashSet<Address> = HashSet::default();
        written_accounts.insert(tx.caller());
        if let TxKind::Call(target) = tx.kind() {
            if !tx.value().is_zero() {
                written_accounts.insert(target);
            }
        }
        let oog = apply_auth_list_eip2780::<_, ERROR>(
            chain_id,
            tx.authorization_list(),
            journal,
            account_write_cost,
            new_account_state_gas,
            delegation_bytes_state_gas,
            &mut written_accounts,
            gas,
        )?;
        return Ok(if oog { None } else { Some(0) });
    }

    let chain_id = context.cfg().chain_id();
    let (tx, journal) = context.tx_journal_mut();

    // Return if not EIP-7702 transaction.
    if tx.tx_type() != TransactionType::Eip7702 {
        return Ok(Some(0));
    }
    let number_of_refunded_accounts =
        apply_auth_list::<_, ERROR>(chain_id, tx.authorization_list(), journal)?;

    let params = context.cfg().gas_params();

    let regular_gas_refund = params
        .tx_eip7702_auth_refund_regular()
        .saturating_mul(number_of_refunded_accounts);

    Ok(Some(regular_gas_refund))
}

/// Applies an EIP-7702 auth list under EIP-2780, recording the
/// state-dependent runtime charges on the transaction-level `gas` instead of
/// the pessimistic intrinsic-charge/refund bookkeeping of [`apply_auth_list`].
///
/// Rejected authorizations charge nothing here: the intrinsic
/// `REGULAR_PER_AUTH_BASE_COST` already covers the work every authorization
/// performs (calldata, recovery, authority access), so there is nothing to
/// refund either.
///
/// `written_accounts` holds the accounts whose leaf write is already paid for
/// (the sender, and the recipient of a value-bearing transaction); applying an
/// authorization to any other authority pays `ACCOUNT_WRITE` on the first
/// write to that authority within the transaction.
///
/// The charges are recorded on `gas` as the authorizations are applied, so
/// the phase stops at the first unaffordable charge without loading the
/// remaining authorities (observable through the EIP-7928 block access list).
///
/// Returns whether the authorization processing ran out of gas.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn apply_auth_list_eip2780<
    JOURNAL: JournalTr,
    ERROR: From<InvalidTransaction> + From<<JOURNAL::Database as Database>::Error>,
>(
    chain_id: u64,
    auth_list: impl Iterator<Item = impl AuthorizationTr>,
    journal: &mut JOURNAL,
    account_write_cost: u64,
    new_account_state_gas: u64,
    delegation_bytes_state_gas: u64,
    written_accounts: &mut HashSet<Address>,
    gas: &mut GasTracker,
) -> Result<bool, ERROR> {
    // EIP-8037 per-authority rules: each charge is applied at most once per
    // authority. The new-account charges self-limit (after the first
    // application the authority exists), the delegation-bytes charge is
    // tracked explicitly to cover a set-clear-set sequence within one
    // transaction.
    let mut charged_delegation_bytes: HashSet<Address> = HashSet::default();

    for authorization in auth_list {
        // 1. Verify the chain id is either 0 or the chain's current ID.
        let auth_chain_id = authorization.chain_id();
        if !auth_chain_id.is_zero() && auth_chain_id != U256::from(chain_id) {
            continue;
        }

        // 2. Verify the `nonce` is less than `2**64 - 1`.
        if authorization.nonce() == u64::MAX {
            continue;
        }

        // recover authority and authorized addresses.
        // 3. `authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]`
        let Some(authority) = authorization.authority() else {
            continue;
        };

        // warm authority account and check nonce.
        // 4. Add `authority` to `accessed_addresses` (as defined in [EIP-2929](./eip-2929.md).)
        let mut authority_acc = journal.load_account_with_code_mut(authority)?;
        let authority_acc_info = &authority_acc.account().info;

        // 5. Verify the code of `authority` is either empty or already delegated.
        if let Some(bytecode) = &authority_acc_info.code {
            // if it is not empty and it is not eip7702
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                continue;
            }
        }

        // 6. Verify the nonce of `authority` is equal to `nonce`. In case `authority` does not exist in the trie, verify that `nonce` is equal to `0`.
        if authorization.nonce() != authority_acc_info.nonce {
            continue;
        }

        // Refund-relevant facts for this accepted authorization (mirrors
        // execution-specs `set_delegation` / evm2 `apply_one_auth`).
        //   existed             — the authority account already existed in state.
        //   delegated_now       — its current code is a delegation (non-empty code
        //                          is necessarily EIP-7702 here, having passed the
        //                          validity check above).
        //   delegated_before_tx — its code at the start of the transaction was a
        //                          delegation (may differ from `delegated_now` when
        //                          an earlier authorization in this tx cleared it).
        //                          Derived from the code hash because the original
        //                          info carries no bytecode when the database serves
        //                          code separately from the account; a non-empty
        //                          hash is necessarily a delegation here, since code
        //                          only changes within a transaction through earlier
        //                          accepted authorizations, which keep it
        //                          empty-or-delegation.
        //   clearing            — this authorization clears the delegation.
        let existed = !(authority_acc_info.is_empty()
            && authority_acc
                .account()
                .is_loaded_as_not_existing_not_touched());
        let delegated_now = !authority_acc_info.is_code_hash_empty_or_zero();
        let delegated_before_tx = !authority_acc
            .account()
            .original_info()
            .is_code_hash_empty_or_zero();
        let clearing = authorization.address().is_zero();

        // Non-existent authority: pay for the new account leaf's state bytes.
        if !existed && !gas.record_state_cost(new_account_state_gas) {
            return Ok(true);
        }

        // First write to the authority's leaf within the transaction pays
        // `ACCOUNT_WRITE`, unless that write is already paid for (the sender at
        // inclusion, the recipient of a value-bearing transaction, or a
        // preceding valid authorization on the same authority).
        if !written_accounts.contains(&authority) {
            if !gas.record_regular_cost(account_write_cost) {
                return Ok(true);
            }
            written_accounts.insert(authority);
        }

        // Net-new delegation bytes: the 23-byte delegation indicator written
        // into a previously empty slot.
        if !clearing
            && !delegated_now
            && !delegated_before_tx
            && !charged_delegation_bytes.contains(&authority)
        {
            if !gas.record_state_cost(delegation_bytes_state_gas) {
                return Ok(true);
            }
            charged_delegation_bytes.insert(authority);
        }

        // 8. Set the code of `authority` to be `0xef0100 || address`. This is a delegation designation.
        //  * As a special case, if `address` is `0x0000000000000000000000000000000000000000` do not write the designation.
        //    Clear the accounts code and reset the account's code hash to the empty hash `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
        // 9. Increase the nonce of `authority` by one.
        authority_acc.delegate(authorization.address());
    }

    Ok(false)
}

/// Apply EIP-7702 style auth list and return number gas refund on already created accounts.
///
/// It is more granular function from [`apply_eip7702_auth_list`] function as it takes only the list, journal and chain id.
///
/// The refund per existing account authorization is
/// `PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST` (25000 - 12500 = 12500), see
/// [`GasParams::tx_eip7702_auth_refund_regular`](context_interface::cfg::gas_params::GasParams::tx_eip7702_auth_refund_regular).
///
/// Returns the number of refunded (already existing) accounts.
#[inline]
pub fn apply_auth_list<
    JOURNAL: JournalTr,
    ERROR: From<InvalidTransaction> + From<<JOURNAL::Database as Database>::Error>,
>(
    chain_id: u64,
    auth_list: impl Iterator<Item = impl AuthorizationTr>,
    journal: &mut JOURNAL,
) -> Result<u64, ERROR> {
    let mut refunded_accounts = 0;
    for authorization in auth_list {
        // 1. Verify the chain id is either 0 or the chain's current ID.
        let auth_chain_id = authorization.chain_id();
        if !auth_chain_id.is_zero() && auth_chain_id != U256::from(chain_id) {
            continue;
        }

        // 2. Verify the `nonce` is less than `2**64 - 1`.
        if authorization.nonce() == u64::MAX {
            continue;
        }

        // recover authority and authorized addresses.
        // 3. `authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]`
        let Some(authority) = authorization.authority() else {
            continue;
        };

        // warm authority account and check nonce.
        // 4. Add `authority` to `accessed_addresses` (as defined in [EIP-2929](./eip-2929.md).)
        let mut authority_acc = journal.load_account_with_code_mut(authority)?;
        let authority_acc_info = &authority_acc.account().info;

        // 5. Verify the code of `authority` is either empty or already delegated.
        if let Some(bytecode) = &authority_acc_info.code {
            // if it is not empty and it is not eip7702
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                continue;
            }
        }

        // 6. Verify the nonce of `authority` is equal to `nonce`. In case `authority` does not exist in the trie, verify that `nonce` is equal to `0`.
        if authorization.nonce() != authority_acc_info.nonce {
            continue;
        }

        // 7. Add `PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST` gas to the global refund counter if `authority` exists in the trie.
        let existed = !(authority_acc_info.is_empty()
            && authority_acc
                .account()
                .is_loaded_as_not_existing_not_touched());
        if existed {
            refunded_accounts += 1;
        }

        // 8. Set the code of `authority` to be `0xef0100 || address`. This is a delegation designation.
        //  * As a special case, if `address` is `0x0000000000000000000000000000000000000000` do not write the designation.
        //    Clear the accounts code and reset the account's code hash to the empty hash `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
        // 9. Increase the nonce of `authority` by one.
        authority_acc.delegate(authorization.address());
    }

    Ok(refunded_accounts)
}

#[cfg(test)]
mod tests {
    use super::validate_account_nonce_and_code;
    use context_interface::result::InvalidTransaction;
    use state::AccountInfo;

    #[test]
    fn rejects_transactions_when_sender_nonce_is_max() {
        let caller_info = AccountInfo {
            nonce: u64::MAX,
            ..AccountInfo::default()
        };

        let err = validate_account_nonce_and_code(&caller_info, u64::MAX, false, false)
            .expect_err("nonce-max sender should be rejected before execution");

        assert_eq!(err, InvalidTransaction::NonceOverflowInTransaction);
    }

    #[test]
    fn allows_matching_non_max_nonce() {
        let caller_info = AccountInfo {
            nonce: 7,
            ..AccountInfo::default()
        };

        assert!(validate_account_nonce_and_code(&caller_info, 7, false, false).is_ok());
    }
}
