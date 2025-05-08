use revm::{
    context::{Cfg, JournalOutput},
    context_interface::{
        result::{HaltReason, InvalidTransaction},
        Block, ContextTr, JournalTr, Transaction,
    },
    handler::{
        pre_execution::validate_account_nonce_and_code, EvmTr, EvmTrError, Frame, FrameResult,
        Handler,
    },
    interpreter::FrameInput,
    primitives::{hardfork::SpecId, U256},
};

use crate::{erc_address_storage, token_operation, TOKEN, TREASURY};

pub struct Erc20MainnetHandler<EVM, ERROR, FRAME> {
    _phantom: core::marker::PhantomData<(EVM, ERROR, FRAME)>,
}

impl<CTX, ERROR, FRAME> Erc20MainnetHandler<CTX, ERROR, FRAME> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM, ERROR, FRAME> Default for Erc20MainnetHandler<EVM, ERROR, FRAME> {
    fn default() -> Self {
        Self::new()
    }
}

impl<EVM, ERROR, FRAME> Handler for Erc20MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>>,
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
    ERROR: EvmTrError<EVM>,
{
    type Evm = EVM;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = HaltReason;

    fn validate_against_state_and_deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), ERROR> {
        let context = evm.ctx();
        let basefee = context.block().basefee() as u128;
        let blob_price = context.block().blob_gasprice().unwrap_or_default();
        let is_balance_check_disabled = context.cfg().is_balance_check_disabled();
        let is_eip3607_disabled = context.cfg().is_eip3607_disabled();
        let is_nonce_check_disabled = context.cfg().is_nonce_check_disabled();
        let caller = context.tx().caller();
        let value = context.tx().value();

        let (tx, journal) = context.tx_journal();

        // Load caller's account.
        let caller_account = journal.load_account_code(tx.caller())?.data;

        validate_account_nonce_and_code(
            &mut caller_account.info,
            tx.nonce(),
            tx.kind().is_call(),
            is_eip3607_disabled,
            is_nonce_check_disabled,
        )?;

        // Touch account so we know it is changed.
        caller_account.mark_touch();

        let max_balance_spending = tx.max_balance_spending()?;
        let effective_balance_spending = tx
            .effective_balance_spending(basefee, blob_price)
            .expect("effective balance is always smaller than max balance so it can't overflow");

        let account_balance_slot = erc_address_storage(tx.caller());
        let account_balance = context
            .journal()
            .sload(TOKEN, account_balance_slot)
            .map(|v| v.data)
            .unwrap_or_default();

        if account_balance < max_balance_spending && !is_balance_check_disabled {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(max_balance_spending),
                balance: Box::new(account_balance),
            }
            .into());
        };

        // Check if account has enough balance for `gas_limit * max_fee`` and value transfer.
        // Transfer will be done inside `*_inner` functions.
        if is_balance_check_disabled {
            // ignore balance check.
            // TODO add transfer value to the erc20 slot.
        } else if max_balance_spending > account_balance {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(max_balance_spending),
                balance: Box::new(account_balance),
            }
            .into());
        } else {
            // subtracting max balance spending with value that is going to be deducted later in the call.
            let gas_balance_spending = effective_balance_spending - value;

            token_operation::<EVM::Context, ERROR>(
                context,
                caller,
                TREASURY,
                gas_balance_spending,
            )?;
        }

        Ok(())
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);
        token_operation::<EVM::Context, ERROR>(
            context,
            TREASURY,
            caller,
            U256::from(reimbursement),
        )?;

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let tx = context.tx();
        let beneficiary = context.block().beneficiary();
        let basefee = context.block().basefee() as u128;
        let effective_gas_price = tx.effective_gas_price(basefee);
        let gas = exec_result.gas();

        let coinbase_gas_price = if context.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let reward =
            coinbase_gas_price.saturating_mul((gas.spent() - gas.refunded() as u64) as u128);
        token_operation::<EVM::Context, ERROR>(context, TREASURY, beneficiary, U256::from(reward))?;

        Ok(())
    }
}
