use crate::{token_operation, TREASURY};
use revm::{
    context_interface::{
        result::InvalidHeader, Block, Transaction, TransactionGetter, TransactionType,
    },
    handler::{EthPreExecution, EthPreExecutionContext, EthPreExecutionError},
    handler_interface::PreExecutionHandler,
    precompile::PrecompileErrors,
    primitives::U256,
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

impl<CTX, ERROR> Default for Erc20PreExecution<CTX, ERROR> {
    fn default() -> Self {
        Self::new()
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
        let basefee = context.block().basefee() as u128;
        let blob_price = context.block().blob_gasprice().unwrap_or_default();
        let effective_gas_price = context.tx().effective_gas_price(basefee);

        let mut gas_cost = (context.tx().gas_limit() as u128).saturating_mul(effective_gas_price);

        if context.tx().tx_type() == TransactionType::Eip4844 {
            let blob_gas = context.tx().total_blob_gas() as u128;
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        let caller = context.tx().caller();
        token_operation::<CTX, ERROR>(context, caller, TREASURY, U256::from(gas_cost))?;

        Ok(())
    }
}
