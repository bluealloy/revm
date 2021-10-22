use core::{cmp::min, marker::PhantomData};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use super::precompiles::{PrecompileOutput, Precompiles};
use crate::{
    collection::{vec::Vec, Map},
    db::Database,
    error::{ExitError, ExitReason, ExitSucceed},
    machine,
    machine::{Contract, Gas, Machine},
    models::SelfDestructResult,
    opcode::gas,
    spec::{Spec, SpecId::*},
    subroutine::{Account, State, SubRoutine},
    util, CallContext, CreateScheme, ExitRevert, GlobalEnv, Inspector, Log, TransactOut,
    TransactTo, Transfer, EVM,
};
use bytes::Bytes;

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> {
    db: &'a mut DB,
    global_env: GlobalEnv,
    subroutine: SubRoutine,
    precompiles: Precompiles,
    inspector: &'a mut dyn Inspector,
    _phantomdata: PhantomData<GSPEC>,
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVM for EVMImpl<'a, GSPEC, DB, INSPECT> {
    fn transact(
        &mut self,
        caller: H160,
        transact_to: TransactTo,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, TransactOut, u64, State) {
        let mut gas = Gas::new(gas_limit);
        //If there is no initial gas, should we take whatever is present?
        if !gas.record_cost(self.initialization::<GSPEC>(
            &data,
            matches!(transact_to, TransactTo::Create(_)),
            access_list,
        )) {
            return (
                ExitError::OutOfGas.into(),
                TransactOut::None,
                0,
                State::new(),
            );
        }

        // load acc
        self.inner_load_account(caller);

        // substract gas_limit*gas_price from current account.
        let payment_value = U256::from(gas_limit) * self.global_env.gas_price;
        if !self.subroutine.balance_sub(caller, payment_value) {
            return (
                ExitError::OutOfFund.into(),
                TransactOut::None,
                0,
                State::new(),
            );
        }

        // record all as cost;
        let gas_limit = gas.remaining();
        gas.record_cost(gas_limit);

        // call inner handling of call/create
        let (exit_reason, ret_gas, out) = match transact_to {
            TransactTo::Call(address) => {
                self.subroutine.inc_nonce(caller);
                let context = CallContext {
                    caller,
                    address,
                    apparent_value: value,
                };
                let (exit, gas, bytes) = self.call_inner::<GSPEC>(
                    address,
                    Transfer {
                        source: caller,
                        target: address,
                        value,
                    },
                    data,
                    gas_limit,
                    context,
                );
                (exit, gas, TransactOut::Call(bytes))
            }
            TransactTo::Create(scheme) => {
                let (exit, address, ret_gas, bytes) =
                    self.create_inner::<GSPEC>(caller, scheme, value, data, gas_limit);
                (exit, ret_gas, TransactOut::Create(bytes, address))
            }
        };

        gas.reimburse_unspend(&exit_reason, ret_gas);
        match self.finalize::<GSPEC>(caller, &gas) {
            Err(e) => (e, out, gas.spend(), Map::new()),
            Ok(state) => (exit_reason, out, gas.spend(), state),
        }
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    pub fn new(
        db: &'a mut DB,
        global_env: GlobalEnv,
        inspector: &'a mut dyn Inspector,
        precompiles: Precompiles,
    ) -> Self {
        let mut precompile_acc = Map::new();
        for add in precompiles.addresses() {
            precompile_acc.insert(add.clone(), db.basic(add.clone()));
        }
        Self {
            db,
            global_env,
            subroutine: SubRoutine::new(precompile_acc),
            precompiles,
            inspector,
            _phantomdata: PhantomData {},
        }
    }

    fn finalize<SPEC: Spec>(
        &mut self,
        caller: H160,
        gas: &Gas,
    ) -> Result<Map<H160, Account>, ExitReason> {
        let gas_price = self.global_env.gas_price;
        let coinbase = self.global_env.block_coinbase;
        let basefee = self.global_env.block_basefee;
        let max_refund_quotient = if SPEC::enabled(LONDON) { 5 } else { 2 }; // EIP-3529: Reduction in refunds
        let gas_refunded = min(gas.refunded() as u64, gas.spend() / max_refund_quotient);
        println!(
            "\n spend:{}, refunded:{}, gas_price:{}, basefee:{:?}",
            gas.spend(),
            gas_refunded,
            gas_price,
            basefee,
        );
        self.subroutine
            .balance_add(caller, gas_price * (gas.remaining() + gas_refunded));
        let coinbase_gas_price = if SPEC::enabled(LONDON) {
            gas_price.saturating_sub(basefee.unwrap_or_default())
        } else {
            gas_price
        };
        if coinbase_gas_price != U256::zero() {
            self.subroutine.load_account(coinbase, self.db);
            self.subroutine
                .balance_add(coinbase, coinbase_gas_price * (gas.spend() - gas_refunded));
        }
        Ok(self.subroutine.finalize())
    }

    fn inner_load_account(&mut self, caller: H160) -> bool {
        let is_cold = self.subroutine.load_account(caller, self.db);
        if INSPECT && is_cold {
            self.inspector.load_account(&caller);
        }
        is_cold
    }

    fn initialization<SPEC: Spec>(
        &mut self,
        input: &Bytes,
        is_create: bool,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> u64 {
        for &ward_acc in self.precompiles.addresses().iter() {
            //TODO trace load precompiles?
            self.subroutine.load_account(ward_acc, self.db);
        }

        let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
        let non_zero_data_len = (input.len() as u64 - zero_data_len) as u64;
        let (accessed_accounts, accessed_slots) = {
            if SPEC::enabled(ISTANBUL) {
                let mut accessed_slots = 0 as u64;
                let accessed_accounts = access_list.len() as u64;

                for (address, slots) in access_list {
                    //TODO trace load access_list?
                    self.subroutine.load_account(address, self.db);
                    accessed_slots += slots.len() as u64;
                    for slot in slots {
                        self.subroutine.sload(address, slot, self.db);
                    }
                }
                (accessed_accounts, accessed_slots)
            } else {
                (0, 0)
            }
        };

        let transact = if is_create {
            if SPEC::enabled(HOMESTEAD) {
                // EIP-2: Homestead Hard-fork Changes
                53000
            } else {
                21000
            }
        } else {
            21000
        };

        // EIP-2028: Transaction data gas cost reduction
        let gas_transaction_non_zero_data = if SPEC::enabled(ISTANBUL) { 16 } else { 68 };

        transact
            + zero_data_len * gas::TRANSACTION_ZERO_DATA
            + non_zero_data_len * gas_transaction_non_zero_data
            + accessed_accounts * gas::ACCESS_LIST_ADDRESS
            + accessed_slots * gas::ACCESS_LIST_STORAGE_KEY
    }

    fn create_inner<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
    ) -> (ExitReason, Option<H160>, Gas, Bytes) {
        //println!("create depth:{}",self.subroutine.depth());
        let gas = Gas::new(gas_limit);
        self.load_account(caller);

        // check depth of calls
        if self.subroutine.depth() > machine::CALL_STACK_LIMIT {
            return (ExitRevert::CallTooDeep.into(), None, gas, Bytes::new());
        }
        // check balance of caller and value. Do this before increasing nonce
        if self.balance(caller).0 < value {
            return (ExitRevert::OutOfFund.into(), None, gas, Bytes::new());
        }

        // inc nonce of caller
        let old_nonce = self.subroutine.inc_nonce(caller);
        // create address
        let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());
        let created_address = match scheme {
            CreateScheme::Create => util::create_address(caller, old_nonce),
            CreateScheme::Create2 { salt } => util::create2_address(caller, code_hash, salt),
        };
        let ret = Some(created_address);

        // load account so that it will be hot
        self.load_account(created_address);

        // enter into subroutine
        let checkpoint = self.subroutine.create_checkpoint();

        // create contract account and check for collision
        if !self
            .subroutine
            .new_contract_acc(created_address, self.precompiles.addresses(), self.db)
        {
            self.subroutine.checkpoint_revert(checkpoint);
            return (ExitError::CreateCollision.into(), ret, gas, Bytes::new());
        }

        // transfer value to contract address
        if let Err(e) = self
            .subroutine
            .transfer(caller, created_address, value, self.db)
        {
            self.subroutine.checkpoint_revert(checkpoint);
            return (e.into(), ret, gas, Bytes::new());
        }
        // inc nonce of contract
        if SPEC::enabled(ISTANBUL) {
            self.subroutine.inc_nonce(created_address);
        }
        // create new machine and execute init function
        let contract = Contract::new(Bytes::new(), init_code, created_address, caller, value);
        let mut machine = Machine::new::<SPEC>(contract, gas.limit(), self.subroutine.depth());
        let exit_reason = machine.run::<Self, SPEC>(self);
        // handler error if present on execution\
        match exit_reason {
            ExitReason::Succeed(_) => {
                let b = Bytes::new();
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let code = machine.return_value();

                // EIP-3541: Reject new contract code starting with the 0xEF byte
                if SPEC::enabled(LONDON) && !code.is_empty() && code.get(0) == Some(&0xEF) {
                    self.subroutine.checkpoint_revert(checkpoint);
                    return (ExitError::CreateContractWithEF.into(), ret, machine.gas, b);
                }

                // EIP-170: Contract code size limit
                if SPEC::enabled(SPURIOUS_DRAGON) && code.len() > 0x6000 {
                    self.subroutine.checkpoint_revert(checkpoint);
                    return (ExitError::CreateContractLimit.into(), ret, machine.gas, b);
                }
                let gas_for_code = code.len() as u64 * crate::opcode::gas::CODEDEPOSIT;
                // record code deposit gas cost and check if we are out of gas.
                if !machine.gas.record_cost(gas_for_code) {
                    self.subroutine.checkpoint_revert(checkpoint);
                    (ExitError::OutOfGas.into(), ret, machine.gas, b)
                } else {
                    // if we have enought gas
                    self.subroutine.checkpoint_commit();
                    self.subroutine.set_code(created_address, code, code_hash);
                    (ExitSucceed::Returned.into(), ret, machine.gas, b)
                }
            }
            _ => {
                self.subroutine.checkpoint_revert(checkpoint);
                (exit_reason, ret, machine.gas, machine.return_value())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_inner<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas_limit: u64,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes) {
        let mut gas = Gas::new(gas_limit);
        // Load account and get code. Account is now hot.
        let (code, _) = self.code(code_address);

        // check depth
        if self.subroutine.depth() > machine::CALL_STACK_LIMIT {
            return (ExitRevert::CallTooDeep.into(), gas, Bytes::new());
        }

        // Create subroutine checkpoint
        let checkpoint = self.subroutine.create_checkpoint();
        //touch address. For "EIP-158 State Clear" this will erase empty accounts.
        if transfer.value == U256::zero() {
            self.load_account(context.address);
            self.subroutine.balance_add(context.address, U256::zero()); // touch the acc
        }

        // transfer value from caller to called account;
        match self
            .subroutine
            .transfer(transfer.source, transfer.target, transfer.value, self.db)
        {
            Err(e) => {
                self.subroutine.checkpoint_revert(checkpoint);
                return (e.into(), gas, Bytes::new());
            }
            Ok((source_is_cold, target_is_cold)) => {
                if INSPECT && source_is_cold {
                    self.inspector.load_account(&transfer.source);
                }
                if INSPECT && target_is_cold {
                    self.inspector.load_account(&transfer.target);
                }
            }
        }

        // call precompiles
        if let Some(precompile) = self.precompiles.get_fun(&code_address) {
            match (precompile)(input.as_ref(), gas_limit, &context, SPEC::IS_STATIC_CALL) {
                Ok(PrecompileOutput { output, cost, logs }) => {
                    if gas.record_cost(cost) {
                        logs.into_iter().for_each(|l| self.subroutine.log(l));
                        self.subroutine.checkpoint_commit();
                        (ExitSucceed::Returned.into(), gas, Bytes::from(output))
                    } else {
                        self.subroutine.checkpoint_revert(checkpoint);
                        (ExitError::OutOfGas.into(), gas, Bytes::new())
                    }
                }
                Err(e) => {
                    self.subroutine.checkpoint_revert(checkpoint); //TODO check if we are discarding or reverting
                    (ExitReason::Error(e), gas, Bytes::new())
                }
            }
        } else {
            // create machine and execute subcall
            let contract = Contract::new_with_context(input, code, &context);
            let mut machine = Machine::new::<SPEC>(contract, gas_limit, self.subroutine.depth());
            let exit_reason = machine.run::<Self, SPEC>(self);
            if matches!(exit_reason, ExitReason::Succeed(_)) {
                self.subroutine.checkpoint_commit();
            } else {
                self.subroutine.checkpoint_revert(checkpoint);
            }

            (exit_reason, machine.gas, machine.return_value())
        }
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> Handler
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    const INSPECT: bool = INSPECT;

    fn env(&self) -> &GlobalEnv {
        &self.global_env
    }

    fn inspect(&mut self) -> &mut dyn Inspector {
        self.inspector
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }

    fn load_account(&mut self, address: H160) -> (bool, bool) {
        let (is_cold, exists) = self.subroutine.load_account_exist(address, self.db);
        if INSPECT && is_cold {
            self.inspector.load_account(&address);
        }
        (is_cold, exists)
    }

    fn balance(&mut self, address: H160) -> (U256, bool) {
        let is_cold = self.inner_load_account(address);
        let balance = self.subroutine.account(address).info.balance;
        (balance, is_cold)
    }

    fn code(&mut self, address: H160) -> (Bytes, bool) {
        let (acc, is_cold) = self.subroutine.load_code(address, self.db);
        if INSPECT && is_cold {
            self.inspector.load_account(&address);
        }
        (acc.info.code.clone().unwrap_or_default(), is_cold)
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: H160) -> (H256, bool) {
        let (acc, is_cold) = self.subroutine.load_code(address, self.db);
        if INSPECT && is_cold {
            self.inspector.load_account(&address);
        }
        if acc.is_empty() {
            return (H256::zero(), is_cold);
        }

        (
            H256::from_slice(
                Keccak256::digest(&acc.info.code.clone().unwrap_or_default()).as_slice(),
            ),
            is_cold,
        )
    }

    fn sload(&mut self, address: H160, index: H256) -> (H256, bool) {
        // account is allways hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.subroutine.sload(address, index, self.db)
    }

    fn sstore(&mut self, address: H160, index: H256, value: H256) -> (H256, H256, H256, bool) {
        self.subroutine.sstore(address, index, value, self.db)
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {
        let log = Log {
            address,
            topics,
            data,
        };
        self.subroutine.log(log);
    }

    fn selfdestruct(&mut self, address: H160, target: H160) -> SelfDestructResult {
        let res = self.subroutine.selfdestruct(address, target, self.db);
        if INSPECT && res.is_cold {
            self.inspector.load_account(&target);
        }
        res
    }

    fn create<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas: u64,
    ) -> (ExitReason, Option<H160>, Gas, Bytes) {
        self.create_inner::<SPEC>(caller, scheme, value, init_code, gas)
    }

    fn call<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas: u64,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes) {
        self.call_inner::<SPEC>(code_address, transfer, input, gas, context)
    }
}

/// EVM context handler.
pub trait Handler {
    const INSPECT: bool;
    /// Get global const context of evm execution
    fn env(&self) -> &GlobalEnv;

    fn inspect(&mut self) -> &mut dyn Inspector;

    /// load account. Returns (is_cold,is_new_account)
    fn load_account(&mut self, address: H160) -> (bool, bool);
    /// Get environmental block hash.
    fn block_hash(&mut self, number: U256) -> H256;
    /// Get balance of address.
    fn balance(&mut self, address: H160) -> (U256, bool);
    /// Get code of address.
    fn code(&mut self, address: H160) -> (Bytes, bool);
    /// Get code hash of address.
    fn code_hash(&mut self, address: H160) -> (H256, bool);
    /// Get storage value of address at index.
    fn sload(&mut self, address: H160, index: H256) -> (H256, bool);
    /// Set storage value of address at index. Return if slot is cold/hot access.
    fn sstore(&mut self, address: H160, index: H256, value: H256) -> (H256, H256, H256, bool);
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn selfdestruct(&mut self, address: H160, target: H160) -> SelfDestructResult;
    /// Invoke a create operation.
    fn create<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas: u64,
    ) -> (ExitReason, Option<H160>, Gas, Bytes);

    /// Invoke a call operation.
    fn call<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas: u64,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes);
}
