//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::tx_validation::{self, ValidationKind, ValidationParams};
use crate::{EvmTr, PrecompileProvider};
use context_interface::{
    journaled_state::{account::JournaledAccountTr, JournalTr},
    result::InvalidTransaction,
    transaction::{AccessListItemTr, AuthorizationTr, Transaction, TransactionType},
    Block, Cfg, ContextTr, Database,
};
use primitives::{hardfork::SpecId, AddressMap, HashSet, StorageKey, ValidationChecks, U256};
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
            .warm_precompiles(precompiles.warm_addresses().collect());
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
///
/// This function uses the [`tx_validation`] module internally to perform validation.
#[inline]
pub fn validate_account_nonce_and_code_with_components(
    caller_info: &AccountInfo,
    tx: impl Transaction,
    cfg: impl Cfg,
) -> Result<(), InvalidTransaction> {
    let params = ValidationParams::caller_params_from_cfg(&cfg);
    tx_validation::validate_caller(caller_info, tx.nonce(), params.validation_kind)
}

/// Validates caller account nonce and code according to EIP-3607.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
#[inline]
pub fn validate_account_nonce_and_code(
    caller_info: &AccountInfo,
    tx_nonce: u64,
    is_eip3607_disabled: bool,
    is_nonce_check_disabled: bool,
) -> Result<(), InvalidTransaction> {
    // Start with CALLER checks only (NONCE, BALANCE, EIP3607) - not ALL
    let mut checks = ValidationChecks::CALLER;
    if is_eip3607_disabled {
        checks.remove(ValidationChecks::EIP3607);
    }
    if is_nonce_check_disabled {
        checks.remove(ValidationChecks::NONCE);
    }
    let kind = ValidationKind::Custom(checks);
    tx_validation::validate_caller(caller_info, tx_nonce, kind)
}

/// Check maximum possible fee and deduct the effective fee.
///
/// Returns new balance.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
#[inline]
pub fn calculate_caller_fee(
    balance: U256,
    tx: impl Transaction,
    block: impl Block,
    cfg: impl Cfg,
) -> Result<U256, InvalidTransaction> {
    let skip_balance_check = cfg.is_balance_check_disabled();
    let caller_fee = tx_validation::calculate_caller_fee(balance, &tx, &block, skip_balance_check)?;
    Ok(caller_fee.new_balance)
}

/// Validates caller state and deducts transaction costs from the caller's balance.
///
/// This function uses the [`tx_validation`] module internally to perform validation.
/// The actual state mutation (setting balance and bumping nonce) is performed
/// by this function after validation.
#[inline]
pub fn validate_against_state_and_deduct_caller<
    CTX: ContextTr,
    ERROR: From<InvalidTransaction> + From<<CTX::Db as Database>::Error>,
>(
    context: &mut CTX,
) -> Result<(), ERROR> {
    let (block, tx, cfg, journal, _, _) = context.all_mut();

    // Create validation params from config
    let params = ValidationParams::caller_params_from_cfg(cfg);
    let skip_balance_check = cfg.is_balance_check_disabled();

    // Load caller's account.
    let mut caller = journal.load_account_with_code_mut(tx.caller())?.data;

    // Validate (no mutation)
    tx_validation::validate_caller(&caller.account().info, tx.nonce(), params.validation_kind)?;
    let caller_fee =
        tx_validation::calculate_caller_fee(*caller.balance(), tx, block, skip_balance_check)?;

    // Apply mutation (Handler responsibility)
    caller.set_balance(caller_fee.new_balance);
    if tx.kind().is_call() {
        caller.bump_nonce();
    }
    Ok(())
}

/// Apply EIP-7702 auth list and return number gas refund on already created accounts.
///
/// Note that this function will do nothing if the transaction type is not EIP-7702.
/// If you need to apply auth list for other transaction types, use [`apply_auth_list`] function.
///
/// Internally uses [`apply_auth_list`] function.
#[inline]
pub fn apply_eip7702_auth_list<
    CTX: ContextTr,
    ERROR: From<InvalidTransaction> + From<<CTX::Db as Database>::Error>,
>(
    context: &mut CTX,
) -> Result<u64, ERROR> {
    let chain_id = context.cfg().chain_id();
    let refund_per_auth = context.cfg().gas_params().tx_eip7702_auth_refund();
    let (tx, journal) = context.tx_journal_mut();

    // Return if not EIP-7702 transaction.
    if tx.tx_type() != TransactionType::Eip7702 {
        return Ok(0);
    }
    apply_auth_list(chain_id, refund_per_auth, tx.authorization_list(), journal)
}

/// Apply EIP-7702 style auth list and return number gas refund on already created accounts.
///
/// It is more granular function from [`apply_eip7702_auth_list`] function as it takes only the list, journal and chain id.
///
/// The `refund_per_auth` parameter specifies the gas refund per existing account authorization.
/// By default this is `PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST` (25000 - 12500 = 12500),
/// but can be configured via [`GasParams::tx_eip7702_auth_refund`](context_interface::cfg::gas_params::GasParams::tx_eip7702_auth_refund).
#[inline]
pub fn apply_auth_list<
    JOURNAL: JournalTr,
    ERROR: From<InvalidTransaction> + From<<JOURNAL::Database as Database>::Error>,
>(
    chain_id: u64,
    refund_per_auth: u64,
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
        if !(authority_acc_info.is_empty()
            && authority_acc
                .account()
                .is_loaded_as_not_existing_not_touched())
        {
            refunded_accounts += 1;
        }

        // 8. Set the code of `authority` to be `0xef0100 || address`. This is a delegation designation.
        //  * As a special case, if `address` is `0x0000000000000000000000000000000000000000` do not write the designation.
        //    Clear the accounts code and reset the account's code hash to the empty hash `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
        // 9. Increase the nonce of `authority` by one.
        authority_acc.delegate(authorization.address());
    }

    let refunded_gas = refunded_accounts * refund_per_auth;

    Ok(refunded_gas)
}
