use std::convert::Infallible;

use revm::{primitives::{address, keccak256, Address, U256}, state::EvmStorageSlot};
use alloy_provider::{network::Ethereum, RootProvider};
use alloy_transport_http::Http;
use database::{AlloyDB, CacheDB};
use reqwest::Client;
use revm::{
    context_interface::{
        result::InvalidTransaction, transaction::Eip4844Tx, Block, JournalStateGetter, JournalStateGetterDBError, Transaction, TransactionGetter, TransactionType
    },
    database_interface::WrapDatabaseAsync,
    handler::{EthPreExecution, EthPreExecutionContext, EthPreExecutionError},
    handler_interface::PreExecutionHandler,
    Context, Database,
};
use alloy_sol_types::{sol, SolValue};


sol! {
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function balanceOf(address owner) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}



type AlloyCacheDB =
    CacheDB<WrapDatabaseAsync<AlloyDB<Http<Client>, Ethereum, RootProvider<Http<Client>>>>>;

// Constants
const TOKEN: Address = address!("1234567890123456789012345678901234567890");
const TREASURY: Address = address!("0000000000000000000000000000000000000001");
const ERC20_TRANSFER_SIGNATURE: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb]; // keccak256("transfer(address,uint256)")[:4]



#[derive(Debug)]
pub enum Erc20PreExecutionError {
   Whatever
}

impl From<Infallible> for Erc20PreExecutionError {
    fn from(_: Infallible) -> Self {
        Self::Whatever
    }
}

impl From<InvalidTransaction> for Erc20PreExecutionError {
    fn from(_: InvalidTransaction) -> Self {
        Self::Whatever
    }
}




struct Erc20PreExecution {
    inner: EthPreExecution<Context, Erc20PreExecutionError>
}

impl Erc20PreExecution {
     fn new() -> Self {
        Self {
            inner: EthPreExecution::new()
        }
    }
}

impl PreExecutionHandler for Erc20PreExecution {
    type Context = Context;
    type Error = Erc20PreExecutionError;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        self.inner.load_accounts(context)
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        self.inner.apply_eip7702_auth_list(context)
    }

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let basefee = context.block.basefee();
        let blob_price = U256::from(context.block.blob_gasprice().unwrap_or_default());
        let effective_gas_price = context.tx().effective_gas_price(*basefee);
        
        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.

        let mut gas_cost = U256::from(context.tx().common_fields().gas_limit())
            .saturating_mul(effective_gas_price);

    
         // EIP-4844
         if context.tx().tx_type() == TransactionType::Eip4844 {
            let blob_gas = U256::from(context.tx().eip4844().total_blob_gas());
            gas_cost = gas_cost.saturating_add(blob_price.saturating_mul(blob_gas));
        }

        // Get the balance slot for the caller
        let caller = context.tx().common_fields().caller();
        let balance_slot: U256 = keccak256((caller, U256::from(3)).abi_encode()).into();

        let token_account = context.journal().load_account(TOKEN)?.data;

        let storage_value  = token_account.storage.get(&balance_slot).expect("Balance slot not found").present_value();

        if storage_value < gas_cost {
            return Err(Erc20PreExecutionError::Whatever);
        }

        // Subtract the gas cost from the caller's balance
        let new_balance = storage_value.saturating_sub(gas_cost);

        token_account.storage.insert(balance_slot, EvmStorageSlot::new_changed(storage_value, new_balance));

        // We could add the gas cost to the treasury's balance
        let treasury_account = context.journal().load_account(TREASURY)?.data;
        let treasury_balance_slot: U256 = keccak256((TREASURY, U256::from(3)).abi_encode()).into();
        let treasury_balance = treasury_account.storage.get(&treasury_balance_slot).expect("Treasury balance slot not found").present_value();
        let new_treasury_balance = treasury_balance.saturating_add(gas_cost);
        treasury_account.storage.insert(treasury_balance_slot, EvmStorageSlot::new_changed(treasury_balance, new_treasury_balance));

        Ok(())
    }
}



fn main() {
    println!("Hello, world!");
}
