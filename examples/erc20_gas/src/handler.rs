use revm::{
    context::Cfg,
    context_interface::{result::HaltReason, Block, ContextTr, JournalTr, Transaction},
    handler::{
        pre_execution::{
            caller_touch_and_change, deduct_caller_balance_with_components,
            validate_account_nonce_and_code_with_components,
        },
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
        let (tx, block, cfg, journal) = evm.ctx_mut().tx_block_cfg_journal_mut();
        journal.load_account(TOKEN)?.data.mark_touch();

        // load account erc20 balance
        let account_balance_slot = erc_address_storage(tx.caller());
        let account_balance = journal
            .sload(TOKEN, account_balance_slot)
            .map(|v| v.data)
            .unwrap_or_default();

        // Load caller's account.
        let caller_account = journal.load_account_code(tx.caller())?.data;

        // validate account nonce and code
        validate_account_nonce_and_code_with_components(&mut caller_account.info, tx, cfg)?;

        let new_balance = deduct_caller_balance_with_components(account_balance, tx, block, cfg)?;

        // make changes to the account. Account balance stays the same
        caller_touch_and_change(
            caller_account,
            caller_account.info.balance,
            tx.kind().is_call(),
        );

        // set new balance
        journal.sstore(TOKEN, account_balance_slot, new_balance)?;

        Ok(())
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        let (block, tx, _, journal, _, _) = evm.ctx().all_mut();
        let effective_gas_price = tx.effective_gas_price(block.basefee() as u128);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);

        let slot = erc_address_storage(tx.caller());

        // reimburse the caller
        let recipient_balance = journal.sload(TOKEN, slot)?.data;
        let recipient_new_balance = recipient_balance.saturating_add(U256::from(reimbursement));
        journal.sstore(TOKEN, slot, recipient_new_balance)?;

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

        let recipient_balance_slot = erc_address_storage(beneficiary);
        let recipient_balance = context
            .journal_mut()
            .sload(TOKEN, recipient_balance_slot)?
            .data;

        let recipient_new_balance = recipient_balance.saturating_add(U256::from(reward));
        context
            .journal_mut()
            .sstore(TOKEN, recipient_balance_slot, recipient_new_balance)?;

        Ok(())
    }
}
