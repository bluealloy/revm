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



#[derive(Debug, Default)]
pub enum Erc20PreExecutionError {
    #[default]
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



#[derive(Default)]
struct Erc20PreExecution {
    inner: EthPreExecution<Context, Erc20PreExecutionError>
}

impl Erc20PreExecution {
     fn new() -> Self {
        Self {
            inner: EthPreExecution::default()
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

        token_operation(context, caller, caller, gas_cost)?;

        Ok(())
    }
}



fn token_operation(context: &mut Context, sender: Address, recipient: Address, amount: U256) -> Result<(), Erc20PreExecutionError> {
    let token_account = context.journal().load_account(TOKEN)?.data;
   
    let sender_balance_slot: U256 = keccak256((sender, U256::from(3)).abi_encode()).into();
    let sender_balance = token_account.storage.get(&sender_balance_slot).expect("Balance slot not found").present_value();

   
    if sender_balance < amount {
        return Err(Erc20PreExecutionError::Whatever);
    }
    // Subtract the amount from the sender's balance
    let sender_new_balance = sender_balance.saturating_sub(amount);
    token_account.storage.insert(sender_balance_slot, EvmStorageSlot::new_changed(sender_balance, sender_new_balance));

    // Add the amount to the recipient's balance
    let recipient_balance_slot: U256 = keccak256((recipient, U256::from(3)).abi_encode()).into();
    let recipient_balance = token_account.storage.get(&recipient_balance_slot).expect("To balance slot not found").present_value();
    let recipient_new_balance = recipient_balance.saturating_add(amount);
    token_account.storage.insert(recipient_balance_slot, EvmStorageSlot::new_changed(recipient_balance, recipient_new_balance));

    Ok(())
}


fn main() {
    println!("Hello, world!");
}