use std::cmp::Ordering;

use revm::{context::Cfg, context_interface::result::{ HaltReason, ResultAndState}, handler::{ EthValidation, FrameResult}, handler_interface::{PostExecutionHandler, ValidationHandler}, primitives::{address, keccak256, Address, U256}, state::EvmStorageSlot};
use alloy_provider::{network::Ethereum, RootProvider};
use alloy_transport_http::Http;
use database::{AlloyDB, CacheDB};
use reqwest::Client;
use revm::{
    context_interface::{
        result::{EVMError, InvalidTransaction},
        transaction::Eip4844Tx,
        Block, JournalStateGetter, JournalStateGetterDBError, Transaction, TransactionGetter, TransactionType
    },
    database_interface::WrapDatabaseAsync,
    handler::{EthPreExecution, EthPostExecution},
    handler_interface::PreExecutionHandler,
    Context,
};
use alloy_sol_types::{sol, SolValue};
use specification::hardfork::SpecId;


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



type Erc20Error = EVMError<JournalStateGetterDBError<Context>, InvalidTransaction>;




struct Erc20PreExecution {
    inner: EthPreExecution<Context, Erc20Error>
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
    type Error = Erc20Error;

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

        token_operation(context, caller, TREASURY, gas_cost)?;

        Ok(())
    }
}



fn token_operation(context: &mut Context, sender: Address, recipient: Address, amount: U256) -> Result<(), Erc20Error> {
    let token_account = context.journal().load_account(TOKEN)?.data;
   
    let sender_balance_slot: U256 = keccak256((sender, U256::from(3)).abi_encode()).into();
    let sender_balance = token_account.storage.get(&sender_balance_slot).expect("Balance slot not found").present_value();

   
    if sender_balance < amount {
        return Err(EVMError::Transaction(InvalidTransaction::MaxFeePerBlobGasNotSupported));
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



struct Erc20PostExecution {
    inner: EthPostExecution<Context, Erc20Error, HaltReason>
}


impl Erc20PostExecution {
    fn new() -> Self {
        Self {
            inner: EthPostExecution::new()
        }
    }
}



impl PostExecutionHandler for Erc20PostExecution {
    type Context = Context;
    type Error = Erc20Error;
    type ExecResult = FrameResult;
    type Output = ResultAndState<HaltReason>;


    fn refund(&self, context: &mut Self::Context, exec_result: &mut Self::ExecResult, eip7702_refund: i64) {
        self.inner.refund(context, exec_result, eip7702_refund)
    }

    fn reimburse_caller(&self, context: &mut Self::Context, exec_result: &mut Self::ExecResult) -> Result<(), Self::Error> {
        let basefee = context.block.basefee();
        let caller = context.tx().common_fields().caller();
        let effective_gas_price = context.tx().effective_gas_price(*basefee);
        let gas = exec_result.gas();

        let reimbursement = effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64);
        token_operation(context, TREASURY, caller, reimbursement).unwrap();


        Ok(())

    }

    fn reward_beneficiary(&self, context: &mut Self::Context, exec_result: &mut Self::ExecResult) -> Result<(), Self::Error> {
        let tx = context.tx();
        let beneficiary = context.block.beneficiary();
        let basefee = context.block.basefee();
        let effective_gas_price = tx.effective_gas_price(*basefee);
        let gas = exec_result.gas();

        let coinbase_gas_price = if context.cfg.spec().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(*basefee)
        } else {
            effective_gas_price
        };

        let reward = coinbase_gas_price * U256::from(gas.spent() - gas.refunded() as u64);
        token_operation(context, TREASURY, *beneficiary, reward).unwrap();
     
        Ok(())
    }

    fn output(&self, context: &mut Self::Context, result: Self::ExecResult) -> Result<Self::Output, Self::Error> {
       self.inner.output(context, result)
    }

    fn clear(&self, context: &mut Self::Context) {
       self.inner.clear(context)
    }
}


struct Erc20Validation {
    inner: EthValidation<Context, Erc20Error>
}


impl Erc20Validation {
    fn new() -> Self {
        Self {
            inner: EthValidation::new()
        }
    }
}


impl ValidationHandler for Erc20Validation {
    type Context = Context;
    type Error = Erc20Error;


    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        self.inner.validate_env(context)
    }

    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        let caller = context.tx().common_fields().caller();
        let caller_nonce = context.journal().load_account(caller)?.data.info.nonce;
        let token_account = context.journal().load_account(TOKEN)?.data.clone();
        
        if !context.cfg.is_nonce_check_disabled() {
            let tx_nonce = context.tx().common_fields().nonce();
            let state_nonce = caller_nonce;
            match tx_nonce.cmp(&state_nonce) {
                Ordering::Less => return Err(EVMError::Transaction(InvalidTransaction::NonceTooLow { tx: tx_nonce, state: state_nonce }.into())),
                Ordering::Greater => return Err(EVMError::Transaction(InvalidTransaction::NonceTooHigh { tx: tx_nonce, state: state_nonce }.into())),
               _ => (),
            }
        }

        // gas_limit * max_fee + value
        let mut balance_check = U256::from(context.tx().common_fields().gas_limit())
            .checked_mul(U256::from(context.tx().max_fee()))
            .and_then(|gas_cost| gas_cost.checked_add(context.tx().common_fields().value()))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        if context.tx().tx_type() == TransactionType::Eip4844 {
            let tx = context.tx().eip4844();
            let data_fee = tx.calc_max_data_fee();
            balance_check = balance_check
                .checked_add(data_fee)
                .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
        }   

        // Get the account balance from token slot
        let account_balance_slot: U256 = keccak256((caller, U256::from(3)).abi_encode()).into();
        let account_balance = token_account.storage.get(&account_balance_slot).expect("Balance slot not found").present_value();

        if account_balance < balance_check && !context.cfg.is_balance_check_disabled() {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(balance_check),
                balance: Box::new(account_balance),
            }
            .into());
        };
     
        Ok(())
    }

    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error> {
        self.inner.validate_initial_tx_gas(context)
    }   
}


fn main() {
    println!("Hello, world!");
}
