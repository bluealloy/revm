//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use bytecode::Bytecode;
use context_interface::{
    journaled_state::Journal,
    result::InvalidTransaction,
    transaction::{
        eip7702::Authorization, AccessListTrait, Eip4844Tx, Eip7702Tx, Transaction, TransactionType,
    },
    Block, BlockGetter, Cfg, CfgGetter, JournalDBError, JournalGetter, TransactionGetter,
};
use handler_interface::PreExecutionHandler;
use primitives::{Address, BLOCKHASH_STORAGE_ADDRESS, U256};
use specification::{eip7702, hardfork::SpecId};
use std::{boxed::Box, vec::Vec};

#[derive(Default)]
pub struct EthPreExecution<CTX, ERROR> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> EthPreExecution<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl<CTX, ERROR> PreExecutionHandler for EthPreExecution<CTX, ERROR>
where
    CTX: EthPreExecutionContext,
    ERROR: EthPreExecutionError<CTX>,
{
    type Context = CTX;
    type Error = ERROR;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let spec = context.cfg().spec().into();
        // Set journaling state flag.
        context.journal().set_spec_id(spec);

        // Load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if spec.is_enabled_in(SpecId::SHANGHAI) {
            let coinbase = context.block().beneficiary();
            context.journal().warm_account(coinbase);
        }

        // Load blockhash storage address
        // EIP-2935: Serve historical block hashes from state
        if spec.is_enabled_in(SpecId::PRAGUE) {
            context.journal().warm_account(BLOCKHASH_STORAGE_ADDRESS);
        }

        // Load access list
        if let Some(access_list) = context.tx().access_list().cloned() {
            for access_list in access_list.iter() {
                context.journal().warm_account_and_storage(
                    access_list.0,
                    access_list.1.map(|i| U256::from_be_bytes(i.0)),
                )?;
            }
        };

        Ok(())
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        let spec = context.cfg().spec().into();
        if spec.is_enabled_in(SpecId::PRAGUE) {
            apply_eip7702_auth_list::<CTX, ERROR>(context)
        } else {
            Ok(0)
        }
    }

    #[inline]
    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let basefee = context.block().basefee();
        let blob_price = context.block().blob_gasprice().unwrap_or_default();
        let effective_gas_price = context.tx().effective_gas_price(basefee as u128);
        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            (context.tx().common_fields().gas_limit() as u128).saturating_mul(effective_gas_price);

        // EIP-4844
        if context.tx().tx_type().into() == TransactionType::Eip4844 {
            let blob_gas = context.tx().eip4844().total_blob_gas() as u128;
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        let is_call = context.tx().kind().is_call();
        let caller = context.tx().common_fields().caller();

        // Load caller's account.
        let caller_account = context.journal().load_account(caller)?.data;
        // Set new caller account balance.
        caller_account.info.balance = caller_account
            .info
            .balance
            .saturating_sub(U256::from(gas_cost));

        // Bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
        if is_call {
            // Nonce is already checked
            caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
        }

        // Touch account so we know it is changed.
        caller_account.mark_touch();
        Ok(())
    }
}

/// Apply EIP-7702 auth list and return number gas refund on already created accounts.
#[inline]
pub fn apply_eip7702_auth_list<
    CTX: TransactionGetter + JournalGetter + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalDBError<CTX>>,
>(
    context: &mut CTX,
) -> Result<u64, ERROR> {
    // Return if there is no auth list.
    let tx = context.tx();
    if tx.tx_type().into() != TransactionType::Eip7702 {
        return Ok(0);
    }

    struct Authorization {
        authority: Option<Address>,
        address: Address,
        nonce: u64,
        chain_id: u64,
    }

    let authorization_list = tx
        .eip7702()
        .authorization_list_iter()
        .map(|a| Authorization {
            authority: a.authority(),
            address: a.address(),
            nonce: a.nonce(),
            chain_id: a.chain_id(),
        })
        .collect::<Vec<_>>();
    let chain_id = context.cfg().chain_id();

    let mut refunded_accounts = 0;
    for authorization in authorization_list {
        // 1. Recover authority and authorized addresses.
        // authority = ecrecover(keccak(MAGIC || rlp([chain_id, address, nonce])), y_parity, r, s]
        let Some(authority) = authorization.authority else {
            continue;
        };

        // 2. Verify the chain id is either 0 or the chain's current ID.
        if authorization.chain_id != 0 && authorization.chain_id != chain_id {
            continue;
        }

        // Warm authority account and check nonce.
        // 3. Add authority to accessed_addresses (as defined in EIP-2929.)
        let mut authority_acc = context.journal().load_account_code(authority)?;

        // 4. Verify the code of authority is either empty or already delegated.
        if let Some(bytecode) = &authority_acc.info.code {
            // If it is not empty and it is not eip7702
            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                continue;
            }
        }

        // 5. Verify the nonce of authority is equal to nonce.
        if authorization.nonce != authority_acc.info.nonce {
            continue;
        }

        // 6. Refund the sender PER_EMPTY_ACCOUNT_COST - PER_AUTH_BASE_COST gas if authority exists in the trie.
        if !authority_acc.is_empty() {
            refunded_accounts += 1;
        }

        // 7. Set the code of authority to be 0xef0100 || address. This is a delegation designation.
        let bytecode = Bytecode::new_eip7702(authorization.address);
        authority_acc.info.code_hash = bytecode.hash_slow();
        authority_acc.info.code = Some(bytecode);

        // 8. Increase the nonce of authority by one.
        authority_acc.info.nonce = authority_acc.info.nonce.saturating_add(1);
        authority_acc.mark_touch();
    }

    let refunded_gas =
        refunded_accounts * (eip7702::PER_EMPTY_ACCOUNT_COST - eip7702::PER_AUTH_BASE_COST);

    Ok(refunded_gas)
}

pub trait EthPreExecutionContext:
    TransactionGetter + BlockGetter + JournalGetter + CfgGetter
{
}

impl<CTX: TransactionGetter + BlockGetter + JournalGetter + CfgGetter> EthPreExecutionContext
    for CTX
{
}

pub trait EthPreExecutionError<CTX: JournalGetter>:
    From<InvalidTransaction> + From<JournalDBError<CTX>>
{
}

impl<CTX: JournalGetter, T: From<InvalidTransaction> + From<JournalDBError<CTX>>>
    EthPreExecutionError<CTX> for T
{
}
