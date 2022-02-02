use crate::{
    db::Database,
    instructions::gas,
    machine,
    machine::{Contract, Gas, Machine},
    models::SelfDestructResult,
    return_ok,
    spec::{Spec, SpecId::*},
    subroutine::{Account, State, SubRoutine},
    util, CallContext, CreateScheme, Env, Inspector, Log, Return, TransactOut, TransactTo,
    Transfer, KECCAK_EMPTY,
};
use alloc::vec::Vec;
use bytes::Bytes;
use core::{cmp::min, marker::PhantomData};
use hashbrown::HashMap as Map;
use primitive_types::{H160, H256, U256};
use revm_precompiles::{Precompile, PrecompileOutput, Precompiles};
use sha3::{Digest, Keccak256};

pub struct EVMData<'a, DB> {
    pub env: &'a mut Env,
    pub subroutine: SubRoutine,
    pub db: &'a mut DB,
}

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> {
    data: EVMData<'a, DB>,
    precompiles: Precompiles,
    inspector: &'a mut dyn Inspector<DB>,
    _phantomdata: PhantomData<GSPEC>,
}

pub trait Transact {
    /// Do transaction.
    /// Return Return, Output for call or Address if we are creating contract, gas spend, State that needs to be applied.
    fn transact(&mut self) -> (Return, TransactOut, u64, State, Vec<Log>);
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> Transact
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn transact(&mut self) -> (Return, TransactOut, u64, State, Vec<Log>) {
        let caller = self.data.env.tx.caller;
        let value = self.data.env.tx.value;
        let data = self.data.env.tx.data.clone();
        let gas_limit = self.data.env.tx.gas_limit;
        let exit = |reason: Return| (reason, TransactOut::None, 0, State::new(), Vec::new());

        if GSPEC::enabled(LONDON) {
            if let Some(priority_fee) = self.data.env.tx.gas_priority_fee {
                if priority_fee > self.data.env.tx.gas_price {
                    // or gas_max_fee for eip1559
                    return exit(Return::GasMaxFeeGreaterThanPriorityFee);
                }
            }
            let effective_gas_price = self.data.env.effective_gas_price();
            let basefee = self.data.env.block.basefee;

            // check minimal cost against basefee
            // TODO maybe do this checks when creating evm. We already have all data there
            // or should be move effective_gas_price inside transact fn
            if effective_gas_price < basefee {
                return exit(Return::GasPriceLessThenBasefee);
            }
            // check if priority fee is lower then max fee
        }
        // unusual to be found here, but check if gas_limit is more then block_gas_limit
        if U256::from(gas_limit) > self.data.env.block.gas_limit {
            return exit(Return::CallerGasLimitMoreThenBlock);
        }

        let mut gas = Gas::new(gas_limit);
        // record initial gas cost. if not using gas metering init will return 0
        if !gas.record_cost(self.initialization::<GSPEC>()) {
            return exit(Return::OutOfGas);
        }

        // load acc
        self.inner_load_account(caller);

        // EIP-3607: Reject transactions from senders with deployed code
        // This EIP is introduced after london but there was no colision in past
        // so we can leave it enabled always
        if self.data.subroutine.account(caller).info.code_hash != KECCAK_EMPTY {
            return exit(Return::RejectCallerWithCode);
        }

        // substract gas_limit*gas_price from current account.
        if let Some(payment_value) =
            U256::from(gas_limit).checked_mul(self.data.env.effective_gas_price())
        {
            if !self.data.subroutine.balance_sub(caller, payment_value) {
                return exit(Return::LackOfFundForGasLimit);
            }
        } else {
            return exit(Return::OverflowPayment);
        }

        // check if we have enought balance for value transfer.
        let difference = self.data.env.tx.gas_price - self.data.env.effective_gas_price();
        if difference + value > self.data.subroutine.account(caller).info.balance {
            return exit(Return::OutOfFund);
        }

        // record all as cost;
        let gas_limit = gas.remaining();
        if crate::USE_GAS {
            gas.record_cost(gas_limit);
        }

        // call inner handling of call/create
        let (exit_reason, ret_gas, out) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                self.data.subroutine.inc_nonce(caller);
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

        if crate::USE_GAS {
            gas.reimburse_unspend(&exit_reason, ret_gas);
        }
        match self.finalize::<GSPEC>(caller, &gas) {
            Err(e) => (e, out, gas.spend(), Map::new(), Vec::new()),
            Ok((state, logs)) => (exit_reason, out, gas.spend(), state, logs),
        }
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: &'a mut dyn Inspector<DB>,
        precompiles: Precompiles,
    ) -> Self {
        let mut subroutine = SubRoutine::default();
        if env.cfg.perf_all_precompiles_have_balance {
            // load precompiles without asking db.
            let mut precompile_acc = Vec::new();
            for (add, _) in precompiles.as_slice() {
                precompile_acc.push(*add);
            }
            subroutine.load_precompiles_default(&precompile_acc);
        } else {
            let mut precompile_acc = Map::new();
            for (add, _) in precompiles.as_slice() {
                precompile_acc.insert(*add, db.basic(*add));
            }
            subroutine.load_precompiles(precompile_acc);
        }
        Self {
            data: EVMData {
                env,
                subroutine,
                db,
            },
            precompiles,
            inspector,
            _phantomdata: PhantomData {},
        }
    }

    fn finalize<SPEC: Spec>(
        &mut self,
        caller: H160,
        gas: &Gas,
    ) -> Result<(Map<H160, Account>, Vec<Log>), Return> {
        let coinbase = self.data.env.block.coinbase;
        if crate::USE_GAS {
            let effective_gas_price = self.data.env.effective_gas_price();
            let basefee = self.data.env.block.basefee;
            let max_refund_quotient = if SPEC::enabled(LONDON) { 5 } else { 2 }; // EIP-3529: Reduction in refunds
            let gas_refunded = min(gas.refunded() as u64, gas.spend() / max_refund_quotient);
            self.data.subroutine.balance_add(
                caller,
                effective_gas_price * (gas.remaining() + gas_refunded),
            );
            let coinbase_gas_price = if SPEC::enabled(LONDON) {
                effective_gas_price.saturating_sub(basefee)
            } else {
                effective_gas_price
            };

            self.data.subroutine.load_account(coinbase, self.data.db);
            self.data
                .subroutine
                .balance_add(coinbase, coinbase_gas_price * (gas.spend() - gas_refunded));
        } else {
            // touch coinbase
            self.data.subroutine.load_account(coinbase, self.data.db);
            self.data.subroutine.balance_add(coinbase, U256::zero());
        }
        let (mut new_state, logs) = self.data.subroutine.finalize();
        // precompiles are special case. If there is precompiles in finalized Map that means some balance is
        // added to it, we need now to load precompile address from db and add this amount to it so that we
        // will have sum.
        if self.data.env.cfg.perf_all_precompiles_have_balance {
            for (address, _) in self.precompiles.as_slice() {
                if let Some(precompile) = new_state.get_mut(address) {
                    // we found it.
                    precompile.info.balance += self.data.db.basic(*address).balance;
                }
            }
        }

        Ok((new_state, logs))
    }

    fn inner_load_account(&mut self, caller: H160) -> bool {
        self.data.subroutine.load_account(caller, self.data.db)
    }

    fn initialization<SPEC: Spec>(&mut self) -> u64 {
        let is_create = matches!(self.data.env.tx.transact_to, TransactTo::Create(_));
        let input = &self.data.env.tx.data;
        let access_list = self.data.env.tx.access_list.clone();

        if crate::USE_GAS {
            let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
            let non_zero_data_len = (input.len() as u64 - zero_data_len) as u64;
            let (accessed_accounts, accessed_slots) = {
                if SPEC::enabled(BERLIN) {
                    let mut accessed_slots = 0_u64;
                    let accessed_accounts = access_list.len() as u64;

                    for (address, slots) in access_list {
                        //TODO trace load access_list?
                        self.data.subroutine.load_account(address, self.data.db);
                        accessed_slots += slots.len() as u64;
                        for slot in slots {
                            self.data.subroutine.sload(address, slot, self.data.db);
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
        } else {
            0
        }
    }

    fn create_inner<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        let gas = Gas::new(gas_limit);
        self.load_account(caller);

        // check depth of calls
        if self.data.subroutine.depth() > machine::CALL_STACK_LIMIT {
            return (Return::CallTooDeep, None, gas, Bytes::new());
        }
        // check balance of caller and value. Do this before increasing nonce
        if self.balance(caller).0 < value {
            return (Return::OutOfFund, None, gas, Bytes::new());
        }

        // inc nonce of caller
        let old_nonce = self.data.subroutine.inc_nonce(caller);
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
        let checkpoint = self.data.subroutine.create_checkpoint();

        // create contract account and check for collision
        if !self.data.subroutine.new_contract_acc(
            created_address,
            self.precompiles.contains(&created_address),
            self.data.db,
        ) {
            self.data.subroutine.checkpoint_revert(checkpoint);
            return (Return::CreateCollision, ret, gas, Bytes::new());
        }

        // transfer value to contract address
        if let Err(e) = self
            .data
            .subroutine
            .transfer(caller, created_address, value, self.data.db)
        {
            self.data.subroutine.checkpoint_revert(checkpoint);
            return (e, ret, gas, Bytes::new());
        }
        // inc nonce of contract
        if SPEC::enabled(ISTANBUL) {
            self.data.subroutine.inc_nonce(created_address);
        }
        // create new machine and execute init function
        let contract =
            Contract::new::<SPEC>(Bytes::new(), init_code, created_address, caller, value);
        let mut machine = Machine::new::<SPEC>(contract, gas.limit(), self.data.subroutine.depth());
        if Self::INSPECT {
            self.inspector
                .initialize_machine(&mut machine, &mut self.data, false); // TODO fix is_static
        }
        let exit_reason = machine.run::<Self, SPEC>(self);
        // Host error if present on execution\
        let ret = match exit_reason {
            return_ok!() => {
                let b = Bytes::new();
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let code = machine.return_value();

                // EIP-3541: Reject new contract code starting with the 0xEF byte
                if SPEC::enabled(LONDON) && !code.is_empty() && code.get(0) == Some(&0xEF) {
                    self.data.subroutine.checkpoint_revert(checkpoint);
                    return (Return::CreateContractWithEF, ret, machine.gas, b);
                }

                // TODO maybe create some macro to hide this `if`
                let mut contract_code_size_limit = 0x6000;
                if INSPECT {
                    contract_code_size_limit = self
                        .inspector
                        .override_spec()
                        .eip170_contract_code_size_limit;
                }
                // EIP-170: Contract code size limit
                if SPEC::enabled(SPURIOUS_DRAGON) && code.len() > contract_code_size_limit {
                    self.data.subroutine.checkpoint_revert(checkpoint);
                    return (Return::CreateContractLimit, ret, machine.gas, b);
                }
                if crate::USE_GAS {
                    let gas_for_code = code.len() as u64 * crate::instructions::gas::CODEDEPOSIT;
                    // record code deposit gas cost and check if we are out of gas.
                    if !machine.gas.record_cost(gas_for_code) {
                        self.data.subroutine.checkpoint_revert(checkpoint);
                        return (Return::OutOfGas, ret, machine.gas, b);
                    }
                }
                // if we have enought gas
                self.data.subroutine.checkpoint_commit();
                let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());
                self.data
                    .subroutine
                    .set_code(created_address, code, code_hash);
                (Return::Continue, ret, machine.gas, b)
            }
            _ => {
                self.data.subroutine.checkpoint_revert(checkpoint);
                (exit_reason, ret, machine.gas, machine.return_value())
            }
        };
        ret
    }

    #[allow(clippy::too_many_arguments)]
    fn call_inner<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas_limit: u64,
        context: CallContext,
    ) -> (Return, Gas, Bytes) {
        let mut gas = Gas::new(gas_limit);
        // Load account and get code. Account is now hot.
        let (code, _) = self.code(code_address);

        // check depth
        if self.data.subroutine.depth() > machine::CALL_STACK_LIMIT {
            return (Return::CallTooDeep, gas, Bytes::new());
        }

        // Create subroutine checkpoint
        let checkpoint = self.data.subroutine.create_checkpoint();
        // touch address. For "EIP-158 State Clear" this will erase empty accounts.
        if transfer.value.is_zero() {
            self.load_account(context.address);
            self.data
                .subroutine
                .balance_add(context.address, U256::zero()); // touch the acc
        }

        // transfer value from caller to called account;
        match self.data.subroutine.transfer(
            transfer.source,
            transfer.target,
            transfer.value,
            self.data.db,
        ) {
            Err(e) => {
                self.data.subroutine.checkpoint_revert(checkpoint);
                return (e, gas, Bytes::new());
            }
            Ok((_source_is_cold, _target_is_cold)) => {}
        }

        // call precompiles
        if let Some(precompile) = self.precompiles.get(&code_address) {
            let out = match precompile {
                Precompile::Standard(fun) => fun(input.as_ref(), gas_limit),
                Precompile::Custom(fun) => fun(input.as_ref(), gas_limit),
            };
            match out {
                Ok(PrecompileOutput { output, cost, logs }) => {
                    if !crate::USE_GAS || gas.record_cost(cost) {
                        logs.into_iter().for_each(|l| {
                            self.data.subroutine.log(Log {
                                address: l.address,
                                topics: l.topics,
                                data: l.data,
                            })
                        });
                        self.data.subroutine.checkpoint_commit();
                        (Return::Continue, gas, Bytes::from(output))
                    } else {
                        self.data.subroutine.checkpoint_revert(checkpoint);
                        (Return::OutOfGas, gas, Bytes::new())
                    }
                }
                Err(_e) => {
                    self.data.subroutine.checkpoint_revert(checkpoint); //TODO check if we are discarding or reverting
                    (Return::Precompile, gas, Bytes::new())
                }
            }
        } else {
            // create machine and execute subcall
            let contract = Contract::new_with_context::<SPEC>(input, code, &context);
            let mut machine =
                Machine::new::<SPEC>(contract, gas_limit, self.data.subroutine.depth());
            if Self::INSPECT {
                self.inspector
                    .initialize_machine(&mut machine, &mut self.data, false); // TODO fix is_static
            }
            let exit_reason = machine.run::<Self, SPEC>(self);
            if matches!(exit_reason, return_ok!()) {
                self.data.subroutine.checkpoint_commit();
            } else {
                self.data.subroutine.checkpoint_revert(checkpoint);
            }

            (exit_reason, machine.gas, machine.return_value())
        }
    }
}

impl<'a, GSPEC: Spec, DB: Database + 'a, const INSPECT: bool> Host
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    const INSPECT: bool = INSPECT;
    type DB = DB;

    fn step(&mut self, machine: &mut Machine, is_static: bool) -> Return {
        self.inspector.step(machine, &mut self.data, is_static);
        Return::Continue
    }

    fn step_end(&mut self, _ret: Return, _machine: &mut Machine) -> Return {
        Return::Continue
    }

    fn env(&mut self) -> &mut Env {
        self.data.env
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        self.data.db.block_hash(number)
    }

    fn load_account(&mut self, address: H160) -> (bool, bool) {
        let (is_cold, exists) = self
            .data
            .subroutine
            .load_account_exist(address, self.data.db);
        (is_cold, exists)
    }

    fn balance(&mut self, address: H160) -> (U256, bool) {
        let is_cold = self.inner_load_account(address);
        let balance = self.data.subroutine.account(address).info.balance;
        (balance, is_cold)
    }

    fn code(&mut self, address: H160) -> (Bytes, bool) {
        let (acc, is_cold) = self.data.subroutine.load_code(address, self.data.db);
        (acc.info.code.clone().unwrap(), is_cold)
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: H160) -> (H256, bool) {
        let (acc, is_cold) = self.data.subroutine.load_code(address, self.data.db);
        //asume that all precompiles have some balance
        if acc.filth.is_precompile() && self.data.env.cfg.perf_all_precompiles_have_balance {
            return (KECCAK_EMPTY, is_cold);
        }
        if acc.is_empty() {
            return (H256::zero(), is_cold);
        }

        (acc.info.code_hash, is_cold)
    }

    fn sload(&mut self, address: H160, index: U256) -> (U256, bool) {
        // account is allways hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.data.subroutine.sload(address, index, self.data.db)
    }

    fn sstore(&mut self, address: H160, index: U256, value: U256) -> (U256, U256, U256, bool) {
        self.data
            .subroutine
            .sstore(address, index, value, self.data.db)
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {
        let log = Log {
            address,
            topics,
            data,
        };
        self.data.subroutine.log(log);
    }

    fn selfdestruct(&mut self, address: H160, target: H160) -> SelfDestructResult {
        if INSPECT {
            self.inspector.selfdestruct();
        }
        self.data
            .subroutine
            .selfdestruct(address, target, self.data.db)
    }

    fn create<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
    ) -> (Return, Option<H160>, Gas, Bytes) {
        if INSPECT {
            let (ret, address, gas, out) = self.inspector.create(
                &mut self.data,
                caller,
                &scheme,
                value,
                &init_code,
                gas_limit,
            );
            if ret != Return::Continue {
                return (ret, address, gas, out);
            }
        }
        let (ret, address, gas, out) =
            self.create_inner::<SPEC>(caller, scheme, value, init_code.clone(), gas_limit);
        if INSPECT {
            self.inspector.create_end(
                &mut self.data,
                caller,
                &scheme,
                value,
                &init_code,
                ret,
                address,
                gas_limit,
                gas.remaining(),
                &out,
            );
        }
        (ret, address, gas, out)
    }

    fn call<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas_limit: u64,
        context: CallContext,
    ) -> (Return, Gas, Bytes) {
        if INSPECT {
            let (ret, gas, out) = self.inspector.call(
                &mut self.data,
                code_address,
                &context,
                &transfer,
                &input,
                gas_limit,
                SPEC::IS_STATIC_CALL,
            );
            if ret != Return::Continue {
                return (ret, gas, out);
            }
        }
        let (ret, gas, out) = self.call_inner::<SPEC>(
            code_address,
            transfer.clone(),
            input.clone(),
            gas_limit,
            context.clone(),
        );
        if INSPECT {
            self.inspector.call_end(
                &mut self.data,
                code_address,
                &context,
                &transfer,
                &input,
                gas_limit,
                gas.remaining(),
                ret,
                &out,
                SPEC::IS_STATIC_CALL,
            );
        }
        (ret, gas, out)
    }
}

/// EVM context host.
pub trait Host {
    const INSPECT: bool;

    type DB: Database;

    fn step(&mut self, machine: &mut Machine, is_static: bool) -> Return;
    fn step_end(&mut self, ret: Return, machine: &mut Machine) -> Return;

    fn env(&mut self) -> &mut Env;

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
    fn sload(&mut self, address: H160, index: U256) -> (U256, bool);
    /// Set storage value of address at index. Return if slot is cold/hot access.
    fn sstore(&mut self, address: H160, index: U256, value: U256) -> (U256, U256, U256, bool);
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
    ) -> (Return, Option<H160>, Gas, Bytes);

    /// Invoke a call operation.
    fn call<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Transfer,
        input: Bytes,
        gas: u64,
        context: CallContext,
    ) -> (Return, Gas, Bytes);
}
