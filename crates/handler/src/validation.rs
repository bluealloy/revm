use context_interface::{
    journaled_state::JournaledState,
    result::{InvalidHeader, InvalidTransaction},
    transaction::{
        eip7702::Authorization, Eip1559CommonTxFields, Eip2930Tx, Eip4844Tx, Eip7702Tx, LegacyTx,
        Transaction, TransactionType,
    },
    Block, BlockGetter, Cfg, CfgGetter, JournalStateGetter, JournalStateGetterDBError,
    TransactionGetter,
};
use core::cmp::{self, Ordering};
use handler_interface::ValidationHandler;
use interpreter::gas;
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
    ERROR: From<InvalidTransaction> + From<InvalidHeader> + From<JournalStateGetterDBError<CTX>>,
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
        let tx_caller = context.tx().common_fields().caller();

        // load acc
        let account = &mut context.journal().load_account_code(tx_caller)?;
        let account = account.data.clone();

        validate_tx_against_account::<CTX, ERROR>(&account, context)
    }

    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error> {
        let spec = context.cfg().spec().into();
        validate_initial_tx_gas::<&Self::Context, InvalidTransaction>(context, spec)
            .map_err(Into::into)
    }
}

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<U256>,
) -> Result<(), InvalidTransaction> {
    if max_priority_fee > max_fee {
        // or gas_max_fee for eip1559
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    // check minimal cost against basefee
    if let Some(base_fee) = base_fee {
        let effective_gas_price = cmp::min(
            U256::from(max_fee),
            base_fee.saturating_add(U256::from(max_priority_fee)),
        );
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
) -> Result<(), InvalidTransaction> {
    // ensure that the user was willing to at least pay the current blob gasprice
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
    }

    // there must be at least one blob
    if blobs.is_empty() {
        return Err(InvalidTransaction::EmptyBlobs);
    }

    // all versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
    for blob in blobs {
        if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
            return Err(InvalidTransaction::BlobVersionNotSupported);
        }
    }

    // ensure the total blob gas spent is at most equal to the limit
    // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
    if blobs.len() > eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize {
        return Err(InvalidTransaction::TooManyBlobs {
            have: blobs.len(),
            max: eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize,
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
    let common_field = context.tx().common_fields();
    let tx_type = context.tx().tx_type().into();

    let base_fee = if context.cfg().is_base_fee_check_disabled() {
        None
    } else {
        Some(*context.block().basefee())
    };

    match tx_type {
        TransactionType::Legacy => {
            let tx = context.tx().legacy();
            // check chain_id only if it is present in the legacy transaction.
            // EIP-155: Simple replay attack protection
            if let Some(chain_id) = tx.chain_id() {
                if chain_id != context.cfg().chain_id() {
                    return Err(InvalidTransaction::InvalidChainId.into());
                }
            }
            // gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if U256::from(tx.gas_price()) < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee.into());
                }
            }
        }
        TransactionType::Eip2930 => {
            // enabled in BERLIN hardfork
            if !spec_id.is_enabled_in(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported.into());
            }
            let tx = context.tx().eip2930();

            if context.cfg().chain_id() != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            // gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if U256::from(tx.gas_price()) < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee.into());
                }
            }
        }
        TransactionType::Eip1559 => {
            if !spec_id.is_enabled_in(SpecId::LONDON) {
                return Err(InvalidTransaction::Eip1559NotSupported.into());
            }
            let tx = context.tx().eip1559();

            if context.cfg().chain_id() != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;
        }
        TransactionType::Eip4844 => {
            if !spec_id.is_enabled_in(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported.into());
            }
            let tx = context.tx().eip4844();

            if context.cfg().chain_id() != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;

            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                context.block().blob_gasprice().unwrap_or_default(),
            )?;
        }
        TransactionType::Eip7702 => {
            // check if EIP-7702 transaction is enabled.
            if !spec_id.is_enabled_in(SpecId::PRAGUE) {
                return Err(InvalidTransaction::Eip7702NotSupported.into());
            }
            let tx = context.tx().eip7702();

            if context.cfg().chain_id() != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId.into());
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;

            let auth_list_len = tx.authorization_list_len();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list_len == 0 {
                return Err(InvalidTransaction::EmptyAuthorizationList.into());
            }

            // TODO temporary here as newest EIP have removed this check.
            for auth in tx.authorization_list_iter() {
                if auth.is_invalid() {
                    return Err(InvalidTransaction::Eip7702NotSupported.into());
                }
            }
        }
        TransactionType::Custom => {
            // custom transaction type check is not done here.
        }
    };

    // Check if gas_limit is more than block_gas_limit
    if !context.cfg().is_block_gas_limit_disabled()
        && U256::from(common_field.gas_limit()) > *context.block().gas_limit()
    {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock.into());
    }

    // EIP-3860: Limit and meter initcode
    if spec_id.is_enabled_in(SpecId::SHANGHAI) && context.tx().kind().is_create() {
        let max_initcode_size = context.cfg().max_code_size().saturating_mul(2);
        if context.tx().common_fields().input().len() > max_initcode_size {
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
    let tx_type = context.tx().tx_type().into();
    // EIP-3607: Reject transactions from senders with deployed code
    // This EIP is introduced after london but there was no collision in past
    // so we can leave it enabled always
    if !context.cfg().is_eip3607_disabled() {
        let bytecode = &account.info.code.as_ref().unwrap();
        // allow EOAs whose code is a valid delegation designation,
        // i.e. 0xef0100 || address, to continue to originate transactions.
        if !bytecode.is_empty() && !bytecode.is_eip7702() {
            return Err(InvalidTransaction::RejectCallerWithCode.into());
        }
    }

    // Check that the transaction's nonce is correct
    if !context.cfg().is_nonce_check_disabled() {
        let tx = context.tx().common_fields().nonce();
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
    let mut balance_check = U256::from(context.tx().common_fields().gas_limit())
        .checked_mul(U256::from(context.tx().max_fee()))
        .and_then(|gas_cost| gas_cost.checked_add(context.tx().common_fields().value()))
        .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

    if tx_type == TransactionType::Eip4844 {
        let tx = context.tx().eip4844();
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
pub fn validate_initial_tx_gas<TxGetter: TransactionGetter, Error>(
    env: TxGetter,
    spec_id: SpecId,
) -> Result<u64, Error>
where
    Error: From<InvalidTransaction>,
{
    let tx_type = env.tx().tx_type().into();

    let authorization_list_num = if tx_type == TransactionType::Eip7702 {
        env.tx().eip7702().authorization_list_len() as u64
    } else {
        0
    };

    let common_fields = env.tx().common_fields();
    let is_create = env.tx().kind().is_create();
    let input = common_fields.input();
    let access_list = env.tx().access_list();

    let initial_gas_spend = gas::validate_initial_tx_gas(
        spec_id,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > common_fields.gas_limit() {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
    }
    Ok(initial_gas_spend)
}

/// Helper trait that summarizes ValidationHandler requirements from Context.
pub trait EthValidationContext:
    TransactionGetter + BlockGetter + JournalStateGetter + CfgGetter
{
}

impl<T: TransactionGetter + BlockGetter + JournalStateGetter + CfgGetter> EthValidationContext
    for T
{
}

/// Helper trait that summarizes all possible requirements by EthValidation.
pub trait EthValidationError<CTX: JournalStateGetter>:
    From<InvalidTransaction> + From<InvalidHeader> + From<JournalStateGetterDBError<CTX>>
{
}

impl<
        CTX: JournalStateGetter,
        T: From<InvalidTransaction> + From<InvalidHeader> + From<JournalStateGetterDBError<CTX>>,
    > EthValidationError<CTX> for T
{
}
