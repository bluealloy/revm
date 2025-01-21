use context_interface::{
    journaled_state::Journal,
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, BlockGetter, Cfg, CfgGetter, JournalDBError, JournalGetter, TransactionGetter,
};
use core::cmp::{self, Ordering};
use handler_interface::{InitialAndFloorGas, ValidationHandler};
use interpreter::gas::{self};
use primitives::{B256, U256};
use specification::{eip4844, hardfork::SpecId};
use state::Account;
use std::boxed::Box;

pub struct EthValidation<CTX, ERROR> {
    pub _phantom: core::marker::PhantomData<fn() -> (CTX, ERROR)>,
}

impl<CTX, ERROR> Default for EthValidation<CTX, ERROR> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR> EthValidation<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl<CTX, ERROR> ValidationHandler for EthValidation<CTX, ERROR>
where
    CTX: EthValidationContext,
    ERROR: From<InvalidTransaction> + From<InvalidHeader> + From<JournalDBError<CTX>>,
{
    type Context = CTX;
    type Error = ERROR;

    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        let spec = context.cfg().spec().into();
        // `prevrandao` is required for the merge
        if spec.is_enabled_in(SpecId::MERGE) && context.block().prevrandao().is_none() {
            return Err(InvalidHeader::PrevrandaoNotSet.into());
        }
        // `excess_blob_gas` is required for Cancun
        if spec.is_enabled_in(SpecId::CANCUN)
            && context.block().blob_excess_gas_and_price().is_none()
        {
            return Err(InvalidHeader::ExcessBlobGasNotSet.into());
        }
        validate_tx_env::<&Self::Context, InvalidTransaction>(context, spec).map_err(Into::into)
    }

    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let tx_caller = context.tx().caller();

        // Load acc
        let account = &mut context.journal().load_account_code(tx_caller)?;
        let account = account.data.clone();

        validate_tx_against_account::<CTX, ERROR>(&account, context)
    }

    fn validate_initial_tx_gas(
        &self,
        context: &Self::Context,
    ) -> Result<InitialAndFloorGas, Self::Error> {
        let spec = context.cfg().spec().into();
        validate_initial_tx_gas(context.tx(), spec).map_err(From::from)
    }
}

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<u128>,
) -> Result<(), InvalidTransaction> {
    if max_priority_fee > max_fee {
        // Or gas_max_fee for eip1559
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    // Check minimal cost against basefee
    if let Some(base_fee) = base_fee {
        let effective_gas_price = cmp::min(max_fee, base_fee.saturating_add(max_priority_fee));
        if effective_gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }

    Ok(())
}

/// Validate EIP-4844 transaction.
pub fn validate_eip4844_tx(
    blobs: &[B256],
    max_blob_fee: u128,
    block_blob_gas_price: u128,
    max_blobs: u8,
) -> Result<(), InvalidTransaction> {
    // Ensure that the user was willing to at least pay the current blob gasprice
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
    }

    // There must be at least one blob
    if blobs.is_empty() {
        return Err(InvalidTransaction::EmptyBlobs);
    }

    // All versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
    for blob in blobs {
        if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
            return Err(InvalidTransaction::BlobVersionNotSupported);
        }
    }

    // Ensure the total blob gas spent is at most equal to the limit
    // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
    if blobs.len() > max_blobs as usize {
        return Err(InvalidTransaction::TooManyBlobs {
            have: blobs.len(),
            max: max_blobs as usize,
        });
    }
    Ok(())
}

/// Validate transaction against block and configuration for mainnet.
pub fn validate_tx_env<CTX: TransactionGetter + BlockGetter + CfgGetter, Error>(
    context: CTX,
    spec_id: SpecId,
) -> Result<(), Error>
where
    Error: From<InvalidTransaction>,
{
    // Check if the transaction's chain id is correct
    let tx_type = context.tx().tx_type();
    let tx = context.tx();

    let base_fee = if context.cfg().is_base_fee_check_disabled() {
        None
    } else {
        Some(context.block().basefee() as u128)
    };

    match TransactionType::from(tx_type) {
        TransactionType::Legacy => {
            // Check chain_id only if it is present in the legacy transaction.
            // EIP-155: Simple replay attack protection
            if let Some(chain_id) = tx.chain_id() {
                if chain_id != context.cfg().chain_id() {
                    return Err(InvalidTransaction::InvalidChainId.into());
                }
            }
            // Gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if tx.gas_price() < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee.into());
                }
            }
        }
        TransactionType::Eip2930 => {
            // Enabled in BERLIN hardfork
            if !spec_id.is_enabled_in(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported.into());
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            // Gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if tx.gas_price() < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee.into());
                }
            }
        }
        TransactionType::Eip1559 => {
            if !spec_id.is_enabled_in(SpecId::LONDON) {
                return Err(InvalidTransaction::Eip1559NotSupported.into());
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;
        }
        TransactionType::Eip4844 => {
            if !spec_id.is_enabled_in(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported.into());
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;

            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                context.block().blob_gasprice().unwrap_or_default(),
                context.cfg().blob_max_count(spec_id),
            )?;
        }
        TransactionType::Eip7702 => {
            // Check if EIP-7702 transaction is enabled.
            if !spec_id.is_enabled_in(SpecId::PRAGUE) {
                return Err(InvalidTransaction::Eip7702NotSupported.into());
            }

            if Some(context.cfg().chain_id()) != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas().unwrap_or_default(),
                base_fee,
            )?;

            let auth_list_len = tx.authorization_list_len();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list_len == 0 {
                return Err(InvalidTransaction::EmptyAuthorizationList.into());
            }
        }
        TransactionType::Custom => {
            // Custom transaction type check is not done here.
        }
    };

    // Check if gas_limit is more than block_gas_limit
    if !context.cfg().is_block_gas_limit_disabled() && tx.gas_limit() > context.block().gas_limit()
    {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock.into());
    }

    // EIP-3860: Limit and meter initcode
    if spec_id.is_enabled_in(SpecId::SHANGHAI) && tx.kind().is_create() {
        let max_initcode_size = context.cfg().max_code_size().saturating_mul(2);
        if context.tx().input().len() > max_initcode_size {
            return Err(InvalidTransaction::CreateInitCodeSizeLimit.into());
        }
    }

    Ok(())
}

/// Validate account against the transaction.
#[inline]
pub fn validate_tx_against_account<CTX: TransactionGetter + CfgGetter, ERROR>(
    account: &Account,
    context: &CTX,
) -> Result<(), ERROR>
where
    ERROR: From<InvalidTransaction>,
{
    let tx = context.tx();
    let tx_type = context.tx().tx_type();
    // EIP-3607: Reject transactions from senders with deployed code
    // This EIP is introduced after london but there was no collision in past
    // so we can leave it enabled always
    if !context.cfg().is_eip3607_disabled() {
        let bytecode = &account.info.code.as_ref().unwrap();
        // Allow EOAs whose code is a valid delegation designation,
        // i.e. 0xef0100 || address, to continue to originate transactions.
        if !bytecode.is_empty() && !bytecode.is_eip7702() {
            return Err(InvalidTransaction::RejectCallerWithCode.into());
        }
    }

    // Check that the transaction's nonce is correct
    if !context.cfg().is_nonce_check_disabled() {
        let tx = tx.nonce();
        let state = account.info.nonce;
        match tx.cmp(&state) {
            Ordering::Greater => {
                return Err(InvalidTransaction::NonceTooHigh { tx, state }.into());
            }
            Ordering::Less => {
                return Err(InvalidTransaction::NonceTooLow { tx, state }.into());
            }
            _ => {}
        }
    }

    // gas_limit * max_fee + value
    let mut balance_check = U256::from(tx.gas_limit())
        .checked_mul(U256::from(tx.max_fee_per_gas()))
        .and_then(|gas_cost| gas_cost.checked_add(tx.value()))
        .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

    if tx_type == TransactionType::Eip4844 {
        let data_fee = tx.calc_max_data_fee();
        balance_check = balance_check
            .checked_add(data_fee)
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
    }

    // Check if account has enough balance for `gas_limit * max_fee`` and value transfer.
    // Transfer will be done inside `*_inner` functions.
    if balance_check > account.info.balance && !context.cfg().is_balance_check_disabled() {
        return Err(InvalidTransaction::LackOfFundForMaxFee {
            fee: Box::new(balance_check),
            balance: Box::new(account.info.balance),
        }
        .into());
    }

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<TransactionT>(
    tx: TransactionT,
    spec_id: SpecId,
) -> Result<InitialAndFloorGas, InvalidTransaction>
where
    TransactionT: Transaction,
{
    let (accounts, storages) = tx.access_list_nums().unwrap_or_default();

    let gas = gas::calculate_initial_tx_gas(
        spec_id,
        tx.input(),
        tx.kind().is_create(),
        accounts as u64,
        storages as u64,
        tx.authorization_list_len() as u64,
    );

    // Additional check to see if limit is big enough to cover initial gas.
    if gas.initial_gas > tx.gas_limit() {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit);
    }

    // EIP-7623: Increase calldata cost
    // floor gas should be less than gas limit.
    if spec_id.is_enabled_in(SpecId::PRAGUE) && gas.floor_gas > tx.gas_limit() {
        return Err(InvalidTransaction::GasFloorMoreThanGasLimit);
    };

    Ok(gas)
}

/// Helper trait that summarizes ValidationHandler requirements from Context.
pub trait EthValidationContext:
    TransactionGetter + BlockGetter + JournalGetter + CfgGetter
{
}

impl<T: TransactionGetter + BlockGetter + JournalGetter + CfgGetter> EthValidationContext for T {}

/// Helper trait that summarizes all possible requirements by EthValidation.
pub trait EthValidationError<CTX: JournalGetter>:
    From<InvalidTransaction> + From<InvalidHeader> + From<JournalDBError<CTX>>
{
}

impl<
        CTX: JournalGetter,
        T: From<InvalidTransaction> + From<InvalidHeader> + From<JournalDBError<CTX>>,
    > EthValidationError<CTX> for T
{
}
