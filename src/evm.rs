use crate::collection::{vec::Vec, Map};
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::{
    db::Database,
    error::{ExitError, ExitReason, ExitSucceed},
    machine::{Contract, Gas, Machine, Stack},
    opcode::OpCode,
    spec::{NotStaticSpec, Spec},
    subroutine::{Account, State, SubRoutine},
    util, CallContext, CreateScheme, GlobalEnv, Log, Transfer,
};
use bytes::Bytes;

pub struct EVM<'a, DB: Database> {
    db: &'a mut DB,
    global_env: GlobalEnv,
    subroutine: SubRoutine,
    precompiles: Map<H160, ()>,
}

impl<'a, DB: Database> EVM<'a, DB> {
    pub fn new(db: &'a mut DB, global_env: GlobalEnv) -> Self {
        Self {
            db,
            global_env,
            subroutine: SubRoutine::new(),
            precompiles: Map::new(),
        }
    }

    pub fn finalize(&mut self) -> Map<H160, Account> {
        self.subroutine.finalize()
    }

    pub fn call<SPEC: Spec>(
        &mut self,
        caller: H160,
        address: H160,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, Bytes, Gas, State) {
        // TODO calculate gascost
        //let transaction_cost = gasometer::call_transaction_cost(&data, &access_list);

        
        let gas_used_init = self.initialization::<SPEC>(&data, false, access_list);
        if gas_limit < gas_used_init {
            return (ExitReason::Error(ExitError::OutOfGas),Bytes::new(),Gas::default(),State::default())
        }
        let gas_limit = gas_limit-gas_used_init;

        self.subroutine.load_account(caller, self.db);
        self.subroutine.inc_nonce(caller);

        let context = CallContext {
            caller,
            address,
            apparent_value: value,
        };

        let (exit, mut gas, bytes) = self.call_inner::<SPEC>(
            address,
            Some(Transfer {
                source: caller,
                target: address,
                value,
            }),
            data,
            gas_limit,
            false,
            context,
        );
        gas.used += gas_used_init;
        (exit, bytes, gas, self.subroutine.finalize())
    }

    pub fn create<SPEC: Spec + NotStaticSpec>(
        &mut self,
        caller: H160,
        value: U256,
        init_code: Bytes,
        create_scheme: CreateScheme,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, Option<H160>, Gas, State) {

        let gas_used_init = self.initialization::<SPEC>(&init_code, true, access_list);
        if gas_limit < gas_used_init {
            return (ExitReason::Error(ExitError::OutOfGas),None,Gas::default(),State::default())
        }
        let gas_limit = gas_limit-gas_used_init;

        let (exit_reason, address, mut gas, _) =
            self.create_inner::<SPEC>(caller, create_scheme, value, init_code, gas_limit);

        gas.used += gas_used_init;
        (exit_reason, address, gas, self.subroutine.finalize())
    }

    fn initialization<SPEC: Spec>(
        &mut self,
        input: &Bytes,
        is_create: bool,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> u64 {
        let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
        let non_zero_data_len = (input.len() as u64 - zero_data_len) as u64;
        let accessed_accounts = access_list.len() as u64;
        let mut accessed_slots = 0 as u64;

        for (address, slots) in access_list {
            self.subroutine.load_account(address, self.db);
            accessed_slots += slots.len() as u64;
            for slot in slots {
                self.subroutine.sload(address, slot, self.db);
            }
        }

        let transact = if is_create {
            SPEC::GAS_TRANSACTION_CREATE
        } else {
            SPEC::GAS_TRANSACTION_CALL
        };

        transact
            + zero_data_len * SPEC::GAS_TRANSACTION_ZERO_DATA
            + non_zero_data_len * SPEC::GAS_TRANSACTION_NON_ZERO_DATA
            + accessed_accounts * SPEC::GAS_ACCESS_LIST_ADDRESS
            + accessed_slots * SPEC::GAS_ACCESS_LIST_STORAGE_KEY
    }

    fn create_inner<SPEC: Spec>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
    ) -> (ExitReason, Option<H160>, Gas, Bytes) {
        //TODO PRECOMPILES

        // set caller/contract_add/precompiles as hot access. Probably can be removed. Acc shold be allready hot.
        self.subroutine.load_account(caller, self.db);

        // trace call
        self.trace_call();
        // check depth of calls
        if self.subroutine.depth() > SPEC::CALL_STACK_LIMIT {
            return (
                ExitError::CallTooDeep.into(),
                None,
                Gas::default(),
                Bytes::new(),
            );
        }

        // check balance of caller and value
        if self.balance(caller).0 < value {
            return (
                ExitError::OutOfFund.into(),
                None,
                Gas::default(),
                Bytes::new(),
            );
        }

        let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());
        // create address
        let address = match scheme {
            CreateScheme::Create => {
                util::create_address(caller, self.subroutine.account(caller).info.nonce)
            }
            CreateScheme::Create2 { salt } => util::create2_address(caller, code_hash, salt),
        };
        // TODO wtf is l64 gas reduction. Check spec. Return gas and set gas_limit
        // inc nonce of caller
        self.subroutine.inc_nonce(caller);
        // enter into subroutine
        let checkpoint = self.subroutine.create_checkpoint();
        // TODO check for code colision by checking nonce and code of created address
        // TODO reset storage to be sure that we dont need to ask db for storage
        if self.subroutine.new_contract_acc(address, self.db) {
            self.subroutine.checkpoint_discard(checkpoint);
            return (
                ExitError::CreateCollision.into(),
                None,
                Gas::default(),
                Bytes::new(),
            );
        }

        // transfer value to contract address
        if let Err(e) = self.subroutine.transfer(caller, address, value, self.db) {
            let _ = self.subroutine.checkpoint_revert(checkpoint);
            return (ExitReason::Error(e), None, Gas::default(), Bytes::new());
        }
        // inc nonce of contract
        if SPEC::CREATE_INCREASE_NONCE {
            self.subroutine.inc_nonce(address);
        }
        // create new machine and execute init function
        let contract = Contract::new(Bytes::new(), init_code, address, caller, value);
        let mut machine = Machine::new(contract, gas_limit);
        let exit_reason = machine.run::<Self, SPEC>(self);
        // handler error if present on execution
        match exit_reason {
            ExitReason::Succeed(_) => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let code = machine.return_value();
                if let Some(limit) = SPEC::CREATE_CONTRACT_LIMIT {
                    if code.len() > limit {
                        // TODO reduce gas and return
                        self.subroutine.checkpoint_discard(checkpoint);
                        return (
                            ExitError::CreateContractLimit.into(),
                            None,
                            machine.gas,
                            Bytes::new(),
                        );
                    }
                }
                let gas_for_code = code.len() as u64 * crate::opcode::gas::CODEDEPOSIT;
                // record code deposit gas cost and check if we are out of gas.
                if !machine.gas.record_cost(gas_for_code) {
                    self.subroutine.checkpoint_discard(checkpoint);
                    (
                        ExitReason::Error(ExitError::OutOfGas),
                        None,
                        Gas::default(),
                        Bytes::new(),
                    )
                } else {
                    println!("SM created: {:?}", address);
                    // if we have enought gas, set code and do checkpoint comit.
                    self.subroutine.set_code(address, code, code_hash);
                    self.subroutine.checkpoint_commit(checkpoint);
                    (
                        ExitReason::Succeed(ExitSucceed::Returned),
                        Some(address),
                        machine.gas,
                        Bytes::new(),
                    )
                }
            }
            ExitReason::Revert(revert) => {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                (
                    ExitReason::Revert(revert),
                    None,
                    machine.gas,
                    machine.return_value(),
                )
            }
            ExitReason::Error(_) | ExitReason::Fatal(_) => {
                let _ = self.subroutine.checkpoint_discard(checkpoint);
                (
                    exit_reason.clone(),
                    None,
                    machine.gas,
                    machine.return_value(),
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_inner<SPEC: Spec>(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Bytes,
        gas_limit: u64,
        take_stipend: bool,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes) {
        // call trace_opcode.

        // wtf is l64  calculate it here and set gas
        let mut gas_limit: u64 = gas_limit;

        // Check stipend and if we are transfering some value

        if let Some(transfer) = transfer.as_ref() {
            if take_stipend && transfer.value != U256::zero() {
                gas_limit = gas_limit.saturating_add(SPEC::CALL_STIPEND);
            }
        }

        // get code that we want to call
        let (code, _) = self.code(code_address);
        // Create subroutine checkpoint
        let checkpoint = self.subroutine.create_checkpoint();
        // TODO touch address
        // self.subroutine.touch(context.address);
        self.subroutine.load_account(context.address, self.db);
        // check depth of calls
        if self.subroutine.depth() > SPEC::CALL_STACK_LIMIT {
            return (ExitError::CallTooDeep.into(), Gas::default(), Bytes::new());
        }

        // transfer value from caller to called address;
        if let Some(transfer) = transfer {
            if let Err(e) =
                self.subroutine
                    .transfer(transfer.source, transfer.target, transfer.value, self.db)
            {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                return (ExitReason::Error(e), Gas::default(), Bytes::new());
            }
        }
        // TODO check if we are calling PRECOMPILES and call it here and return.
        // create machine and execute it
        let contract = Contract::new(
            input,
            code,
            context.address,
            context.caller,
            context.apparent_value,
        );
        let mut machine = Machine::new(contract, gas_limit);
        let exit_reason = machine.run::<Self, SPEC>(self);
        //let gas = machine.gas;
        match exit_reason {
            ExitReason::Succeed(_) => {
                let _ = self.subroutine.checkpoint_commit(checkpoint);
                (exit_reason, machine.gas, machine.return_value())
            }
            ExitReason::Revert(_) => {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                (exit_reason, machine.gas, machine.return_value())
            }
            ExitReason::Error(_) | ExitReason::Fatal(_) => {
                let _ = self.subroutine.checkpoint_discard(checkpoint);
                (exit_reason, machine.gas, machine.return_value())
            }
        }
    }
}

impl<'a, DB: Database> Handler for EVM<'a, DB> {
    fn global_env(&self) -> &GlobalEnv {
        &self.global_env
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }

    fn balance(&mut self, address: H160) -> (U256, bool) {
        let is_cold = self.subroutine.load_account(address, self.db);
        let balance = self.subroutine.account(address).info.balance;
        (balance, is_cold)
    }

    fn code(&mut self, address: H160) -> (Bytes, bool) {
        let (acc, is_cold) = self.subroutine.load_code(address, self.db);
        (acc.info.code.clone().unwrap(), is_cold)
    }

    fn sload(&mut self, address: H160, index: H256) -> (H256, bool) {
        // account is allways hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.subroutine.sload(address, index, self.db)
    }

    // TODO check return value, should it be is_cold or maybe original value
    fn sstore(&mut self, address: H160, index: H256, value: H256) -> (H256, H256, H256, bool) {
        self.subroutine.sstore(address, index, value, self.db)
    }

    fn original_storage(&mut self, address: H160, index: H256) -> H256 {
        self.db.original_storage(address, index).unwrap_or_default()
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {
        let log = Log {
            address,
            topics,
            data,
        };
        self.subroutine.log(log);
    }

    fn selfdestruct(
        &mut self,
        _address: H160,
        _target: H160,
    ) -> Result<SelfDestructResult, ExitError> {
        Ok(SelfDestructResult {
            value: U256::from(10),
            is_cold: false,
            exists: true,
            previously_destroyed: false,
        })
        // TODO
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
        transfer: Option<Transfer>,
        input: Bytes,
        gas: u64,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes) {
        self.call_inner::<SPEC>(code_address, transfer, input, gas, true, context)
    }
}

impl<'a, DB: Database> Tracing for EVM<'a, DB> {}
impl<'a, DB: Database> ExtHandler for EVM<'a, DB> {}

pub struct SelfDestructResult {
    pub value: U256,
    pub is_cold: bool,
    pub exists: bool,
    pub previously_destroyed: bool,
}
/// EVM context handler.
pub trait Handler {
    /// Get global const context of evm execution
    fn global_env(&self) -> &GlobalEnv;

    /// Get environmental block hash.
    fn block_hash(&mut self, number: U256) -> H256;
    /// Get balance of address.
    fn balance(&mut self, address: H160) -> (U256, bool);
    /// Get code of address.
    fn code(&mut self, address: H160) -> (Bytes, bool);
    /// Get storage value of address at index.
    fn sload(&mut self, address: H160, index: H256) -> (H256, bool);
    /// Set storage value of address at index. Return if slot is cold/hot access.
    fn sstore(&mut self, address: H160, index: H256, value: H256) -> (H256, H256, H256, bool);
    /// Get original storage value of address at index.
    fn original_storage(&mut self, address: H160, index: H256) -> H256;
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn selfdestruct(
        &mut self,
        address: H160,
        target: H160,
    ) -> Result<SelfDestructResult, ExitError>;
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
        transfer: Option<Transfer>,
        input: Bytes,
        gas: u64,
        context: CallContext,
    ) -> (ExitReason, Gas, Bytes);
}

pub trait Tracing {
    fn trace_opcode(&mut self, _contract: &Contract, opcode: OpCode, _stack: &Stack) {
        println!(
            "Opcode:{:?} ({:?})",
            opcode,
            opcode as u8,
            //stack.data()
        );
    }
    fn trace_call(&mut self) {}
}

pub trait ExtHandler: Handler + Tracing {
    /// Get code size of address.
    fn code_size(&mut self, address: H160) -> (U256, bool) {
        let (code, is_cold) = self.code(address);
        (U256::from(code.len()), is_cold)
    }
    /// Get code hash of address.
    fn code_hash(&mut self, address: H160) -> (H256, bool) {
        let (code, is_cold) = self.code(address);
        if code.is_empty() {
            return (H256::default(), true); // TODO check is_cold
        }
        (H256::from_slice(&Keccak256::digest(code.as_ref())), is_cold)
    }

    /// Get the gas price value.
    fn gas_price(&self) -> U256 {
        self.global_env().gas_price
    }
    /// Get execution origin.
    fn origin(&self) -> H160 {
        self.global_env().origin
    }
    /// Get environmental block number.
    fn block_number(&self) -> U256 {
        self.global_env().block_number
    }
    /// Get environmental coinbase.
    fn block_coinbase(&self) -> H160 {
        self.global_env().block_coinbase
    }
    /// Get environmental block timestamp.
    fn block_timestamp(&self) -> U256 {
        self.global_env().block_timestamp
    }
    /// Get environmental block difficulty.
    fn block_difficulty(&self) -> U256 {
        self.global_env().block_difficulty
    }
    /// Get environmental gas limit.
    fn block_gas_limit(&self) -> U256 {
        self.global_env().block_gas_limit
    }
    /// Get environmental chain ID.
    fn chain_id(&self) -> U256 {
        self.global_env().chain_id
    }
}
