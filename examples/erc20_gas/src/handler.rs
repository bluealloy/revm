use revm::{
    context::Cfg,
    context_interface::{result::HaltReason, Block, ContextTr, JournalTr, Transaction},
    handler::{
        pre_execution::{calculate_caller_fee, validate_account_nonce_and_code_with_components},
        EvmTr, EvmTrError, FrameResult, FrameTr, Handler,
    },
    interpreter::interpreter_action::FrameInit,
    primitives::{hardfork::SpecId, U256},
    state::EvmState,
};

use crate::{erc_address_storage, TOKEN};

/// Custom handler that implements ERC20 token gas payment.
/// Instead of paying gas in ETH, transactions pay gas using ERC20 tokens.
/// The tokens are transferred from the transaction sender to a treasury address.
#[derive(Debug)]
pub struct Erc20MainnetHandler<EVM, ERROR, FRAME> {
    _phantom: core::marker::PhantomData<(EVM, ERROR, FRAME)>,
}

impl<CTX, ERROR, FRAME> Erc20MainnetHandler<CTX, ERROR, FRAME> {
    /// Creates a new ERC20 gas payment handler
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
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<State = EvmState>>, Frame = FRAME>,
    FRAME: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
    ERROR: EvmTrError<EVM>,
{
    type Evm = EVM;
    type Error = ERROR;
    type HaltReason = HaltReason;

    fn validate_against_state_and_deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), ERROR> {
        let (block, tx, cfg, journal, _, _) = evm.ctx_mut().all_mut();

        // load TOKEN contract
        journal.load_account(TOKEN)?.data.mark_touch();

        // Load caller's account.
        let caller_account = journal.load_account_code(tx.caller())?.data;

        validate_account_nonce_and_code_with_components(&mut caller_account.info, tx, cfg)?;

        // make changes to the account. Account balance stays the same
        caller_account
            .caller_initial_modification(caller_account.info.balance, tx.kind().is_call());

        let account_balance_slot = erc_address_storage(tx.caller());

        // load account balance
        let account_balance = journal.sload(TOKEN, account_balance_slot)?.data;

        let new_balance = calculate_caller_fee(account_balance, tx, block, cfg)?;

        // store deducted balance.
        journal.sstore(TOKEN, account_balance_slot, new_balance)?;

        Ok(())
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        let context = evm.ctx();
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);

        let account_balance_slot = erc_address_storage(caller);

        // load account balance
        let account_balance = context
            .journal_mut()
            .sload(TOKEN, account_balance_slot)?
            .data;

        // reimburse caller
        context.journal_mut().sstore(
            TOKEN,
            account_balance_slot,
            account_balance + U256::from(reimbursement),
        )?;

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
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

        let reward = coinbase_gas_price.saturating_mul(gas.used() as u128);

        let beneficiary_slot = erc_address_storage(beneficiary);
        // load account balance
        let journal = context.journal_mut();
        let beneficiary_balance = journal.sload(TOKEN, beneficiary_slot)?.data;
        // reimburse caller
        journal.sstore(
            TOKEN,
            beneficiary_slot,
            beneficiary_balance + U256::from(reward),
        )?;

        Ok(())
    }
}
