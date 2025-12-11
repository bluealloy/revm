//! Monad handler implementation.
//!
//! Key differences from Ethereum:
//! - Gas is charged based on gas_limit, not gas_used (no refunds)
use revm::{
    context_interface::{result::HaltReason, Block, Cfg, ContextTr, JournalTr, Transaction},
    handler::{evm::FrameTr, handler::EvmTrError, EvmTr, FrameResult, Handler, MainnetHandler},
    interpreter::interpreter_action::FrameInit,
    primitives::{hardfork::SpecId, U256},
    state::EvmState,
};

/// Monad handler extends [`Handler`] with Monad-specific gas handling.
///
/// Key difference: Gas is charged based on gas_limit rather than gas_used.
/// This is a DOS-prevention measure for Monad's asynchronous execution.

#[derive(Debug, Clone)]
pub struct MonadHandler<EVM, ERROR, FRAME> {
    /// Mainnet handler allows us to use functions from the mainnet handler inside monad handler.
    /// So we dont duplicate the logic
    pub mainnet: MainnetHandler<EVM, ERROR, FRAME>,
}

impl<EVM, ERROR, FRAME> MonadHandler<EVM, ERROR, FRAME> {
    /// Create a new Monad handler.
    pub fn new() -> Self {
        Self {
            mainnet: MainnetHandler::default(),
        }
    }
}

impl<EVM, ERROR, FRAME> Default for MonadHandler<EVM, ERROR, FRAME> {
    fn default() -> Self {
        Self::new()
    }
}

impl<EVM, ERROR, FRAME> Handler for MonadHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<State = EvmState>>, Frame = FRAME>,
    ERROR: EvmTrError<EVM>,
    FRAME: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
{
    type Evm = EVM;
    type Error = ERROR;
    type HaltReason = HaltReason;

    // Disable gas refunds
    fn refund(
        &self,
        _evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
        _eip7702_refund: i64,
    ) {
        exec_result.gas_mut().set_refund(0);
    }

    // Don't reimburse caller
    fn reimburse_caller(
        &self,
        _evm: &mut Self::Evm,
        _exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    // Pay full gas_limit to beneficiary
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        _exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        // a modified version of post_execution::reward_beneficiary() to charge based on gas_limit() not gas.used()
        let ctx = evm.ctx();

        let gas_limit = ctx.tx().gas_limit();
        let basefee = ctx.block().basefee() as u128;
        let effective_gas_price = ctx.tx().effective_gas_price(basefee);

        let coinbase_gas_price = if ctx.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let reward = coinbase_gas_price * gas_limit as u128;
        let beneficiary = ctx.block().beneficiary();

        ctx.journal_mut()
            .balance_incr(beneficiary, U256::from(reward))?;

        Ok(())
    }
}
