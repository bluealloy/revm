use crate::{token_operation, TREASURY};
use revm::context_interface::result::InvalidHeader;
use revm::context_interface::transaction::Eip4844Tx;
use revm::context_interface::{Block, Transaction, TransactionGetter};
use revm::handler::{EthPreExecutionContext, EthPreExecutionError};
use revm::precompile::PrecompileErrors;
use revm::{
    context_interface::TransactionType, handler::EthPreExecution,
    handler_interface::PreExecutionHandler, primitives::U256,
};

pub struct Erc20PreExecution<CTX, ERROR> {
    inner: EthPreExecution<CTX, ERROR>,
}

impl<CTX, ERROR> Erc20PreExecution<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            inner: EthPreExecution::new(),
        }
    }
}

impl<CTX, ERROR> PreExecutionHandler for Erc20PreExecution<CTX, ERROR>
where
    CTX: EthPreExecutionContext,
    ERROR: EthPreExecutionError<CTX> + From<InvalidHeader> + From<PrecompileErrors>,
{
    type Context = CTX;
    type Error = ERROR;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        self.inner.load_accounts(context)
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        self.inner.apply_eip7702_auth_list(context)
    }

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let basefee = context.block().basefee();
        let blob_price = U256::from(context.block().blob_gasprice().unwrap_or_default());
        let effective_gas_price = context.tx().effective_gas_price(*basefee);

        let mut gas_cost = U256::from(context.tx().common_fields().gas_limit())
            .saturating_mul(effective_gas_price);

        if context.tx().tx_type().into() == TransactionType::Eip4844 {
            let blob_gas = U256::from(context.tx().eip4844().total_blob_gas());
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        let caller = context.tx().common_fields().caller();
        token_operation::<CTX, ERROR>(context, caller, TREASURY, gas_cost)?;

        Ok(())
    }
}
