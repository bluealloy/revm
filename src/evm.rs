use std::{
    marker::PhantomData,
    process::{exit, ExitStatus},
};

use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::{
    db::Database,
    error::{ExitError, ExitReason, ExitSucceed},
    machine::{Contract, Stack},
    opcode::OpCode,
    spec::Spec,
    subroutine::SubRoutine,
    util, AccountInfo, CallContext, CreateScheme, GlobalEnv, Log, Machine, Transfer,
};
use bytes::Bytes;

pub struct EVM<'a, SPEC: Spec, DB: Database> {
    db: &'a mut DB,
    global_env: GlobalEnv,
    subroutine: SubRoutine,
    gas: U256,
    phantomdata: PhantomData<SPEC>,
    is_static: bool,
}

pub enum CreateType {
    Create,
    Create2(H256), // H256 is salt
}

impl<'a, SPEC: Spec, DB: Database> EVM<'a, SPEC, DB> {
    pub fn new(db: &'a mut DB, global_env: GlobalEnv) -> Self {
        let gas = global_env.gas_limit.clone();
        Self {
            db,
            global_env,
            subroutine: SubRoutine::new(),
            gas,
            phantomdata: PhantomData,
            is_static: false,
        }
    }

    pub fn call(
        &mut self,
        caller: H160,
        address: H160,
        value: U256,
        data: Bytes,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> (ExitReason, Bytes) {
        // TODO calculate gascost
        //let transaction_cost = gasometer::call_transaction_cost(&data, &access_list);

        self.load_access_list(access_list);

        self.subroutine.load_account(caller, self.db);
        self.subroutine.inc_nonce(caller);

        let context = CallContext {
            caller,
            address,
            apparent_value: value,
        };

        self.call_inner(
            address,
            Some(Transfer {
                source: caller,
                target: address,
                value,
            }),
            data,
            Some(gas_limit),
            false,
            false,
            false,
            context,
        )
    }

    pub fn create(
        &mut self,
        caller: H160,
        value: U256,
        init_code: Bytes,
        create_scheme: CreateScheme,
        gas_limit: u64,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> ExitReason {
        //TODO calculate transacition cost and add it to gasometer
        // load access_list items into subrutine

        self.load_access_list(access_list);

        let (exit_reason, _, _) = self.create_inner(
            caller,
            create_scheme,
            value,
            init_code,
            Some(gas_limit),
            false,
        );

        exit_reason
    }
    
    fn load_access_list(&mut self, access_list: Vec<(H160, Vec<H256>)>) {

    }

    fn create_inner(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        target_gas: Option<u64>,
        take_l64: bool,
    ) -> (ExitReason, Option<H160>, Bytes) {
        //todo!()

        // set caller/contract_add/precompiles as hot access. PRobably can be removed. Acc shold be allready hot.
        self.subroutine.load_account(caller, self.db);

        // trace call
        self.trace_call();
        // check depth of calls
        if self.subroutine.depth() > SPEC::call_stack_limit {
            return (ExitError::CallTooDeep.into(), None, Bytes::new());
        }

        // check balance of caller and value
        if self.balance(caller).0 < value {
            return (ExitError::OutOfFund.into(), None, Bytes::new());
        }
        
        let code_hash = H256::from_slice(Keccak256::digest(&init_code).as_slice());
        // create address
        let address = match scheme {
            CreateScheme::Create => {
                util::create_address(caller, self.subroutine.account(caller).info.nonce)
            }
            CreateScheme::Create2 { salt } => {
                util::create2_address(caller, code_hash, salt)
            }
        };
        // TODO wtf is l64 gas reduction. Check spec. Return gas and set gas_limit
        // inc nonce of caller
        self.subroutine.inc_nonce(caller);
        // enter into subroutine
        let checkpoint = self.subroutine.create_checkpoint();
        // TODO check for code colision by checking nonce and code of created address
        // TODO reset storage to be sure that we dont need to ask db for storage

        // transfer value to contract address
        if let Err(e) = self.subroutine.transfer(caller, address, value,self.db) {
            let _ = self.subroutine.checkpoint_revert(checkpoint);
            return (ExitReason::Error(e), None, Bytes::new());
        }
        // inc nonce of contract
        if SPEC::create_increase_nonce {
            self.subroutine.inc_nonce(address);
        }
        // create new machine and execute init function
        let contract = Contract::new(Bytes::new(), init_code, address, caller, value);
        let mut machine = Machine::new(contract);
        let exit_reason = machine.run::<Self, SPEC>(self);
        // handler error if present on execution
        match exit_reason {
            ExitReason::Succeed(s) => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let code = machine.return_value();
                if let Some(limit) = SPEC::create_contract_limit {
                    if code.len() > limit {
                        // TODO reduce gas and return
                        self.subroutine.checkpoint_discard(checkpoint);
                        return (ExitError::CreateContractLimit.into(), None, Bytes::new());
                    }
                }
                // TODO check gas used and revert if we overspend
                println!("SM created: {:?}",address);
                let acc = self.subroutine.account_mut(address);
                acc.info.code = Some(code);
                acc.info.code_hash = Some(code_hash);

                self.subroutine.checkpoint_commit(checkpoint);
                (
                    ExitReason::Succeed(ExitSucceed::Returned),
                    Some(address),
                    Bytes::new(),
                )
            }
            ExitReason::Revert(revert) => {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                (ExitReason::Revert(revert), None, machine.return_value())
            }
            ExitReason::Error(_) | ExitReason::Fatal(_) => {
                let _ = self.subroutine.checkpoint_discard(checkpoint);
                (exit_reason.clone(), None, machine.return_value())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_inner(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Bytes,
        target_gas: Option<u64>,
        is_static: bool,
        take_l64: bool,
        take_stipend: bool,
        context: CallContext,
    ) -> (ExitReason, Bytes) {
        // call trace_opcode.

        // wtf is l64  calculate it here and set gas
        let mut gas_limit: u64 = 0;

        // Check stipend and if we are transfering some value

        if let Some(transfer) = transfer.as_ref() {
            if take_stipend && transfer.value != U256::zero() {
                gas_limit = gas_limit.saturating_add(SPEC::call_stipend);
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
        if self.subroutine.depth() > SPEC::call_stack_limit {
            return (ExitError::CallTooDeep.into(), Bytes::new());
        }

        // transfer value from caller to called address;
        if let Some(transfer) = transfer {
            if let Err(e) =
                self.subroutine
                    .transfer(transfer.source, transfer.target, transfer.value,self.db)
            {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                return (ExitReason::Error(e), Bytes::new());
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
        let mut machine = Machine::new(contract);
        let exit_reason = machine.run::<Self, SPEC>(self);
        match exit_reason {
            ExitReason::Succeed(_) => {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                (exit_reason, machine.return_value())
            }
            ExitReason::Revert(revert) => {
                let _ = self.subroutine.checkpoint_revert(checkpoint);
                (exit_reason, machine.return_value())
            }
            ExitReason::Error(_) | ExitReason::Fatal(_) => {
                let _ = self.subroutine.checkpoint_discard(checkpoint);
                (exit_reason, machine.return_value())
            }
        }
    }
}

impl<'a, SPEC: Spec, DB: Database> Handler for EVM<'a, SPEC, DB> {
    fn global_env(&self) -> &GlobalEnv {
        &self.global_env
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }

    fn balance(&mut self, address: H160) -> (U256, bool) {
        let (acc, is_cold) = self.subroutine.load_account(address, self.db);
        (acc.info.balance, is_cold)
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
    fn sstore(&mut self, address: H160, index: H256, value: H256) {
        self.subroutine.sstore(address, index, value, self.db);
    }

    fn original_storage(&mut self, address: H160, index: H256) -> H256 {
        self.db.original_storage(address, index).unwrap_or_default()
    }

    // TODO Used for selfdestruct gas calculation. Should probably be merged with delete call
    // fn deleted(&self, address: H160) -> bool {
    //     true
    // }

    // This two next functions should be removed. THhis information should be passed when asking needed data/account/slot
    // fn is_cold(&self, address: H160) -> bool {
    //     true
    // }

    // fn is_cold_storage(&self, address: H160, index: H256) -> bool {
    //     true
    // }

    fn gas_left(&self) -> U256 {
        self.gas
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {
        let log = Log {
            address,
            topics,
            data,
        };
        self.subroutine.log(log);
    }

    fn selfdestruct<const CALL_TRACE: bool>(
        &mut self,
        address: H160,
        target: H160,
    ) -> Result<(), ExitError> {
        Ok(())
    }

    fn create<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        target_gas: Option<u64>,
    ) -> (ExitReason, Option<H160>, Bytes) {
        self.create_inner(caller, scheme, value, init_code, target_gas, true)
    }

    fn call<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Bytes,
        target_gas: Option<u64>,
        is_static: bool,
        context: CallContext,
    ) -> (ExitReason, Bytes) {
        self.call_inner(
            code_address,
            transfer,
            input,
            target_gas,
            is_static,
            true,
            true,
            context,
        )
    }
}

impl<'a, SPEC: Spec, DB: Database> Tracing for EVM<'a, SPEC, DB> {}
impl<'a, SPEC: Spec, DB: Database> ExtHandler for EVM<'a, SPEC, DB> {}

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
    fn sstore(&mut self, address: H160, index: H256, value: H256);
    /// Get original storage value of address at index.
    fn original_storage(&mut self, address: H160, index: H256) -> H256;

    // Check whether an address exists.
    //fn exists(&self, address: H160) -> bool;
    // Check whether an address has already been deleted. Should be merged with selfdestruct mark_delete call
    // fn deleted(&self, address: H160) -> bool;

    /// Get the gas left value. It contacts gasometer
    fn gas_left(&self) -> U256;
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn selfdestruct<const CALL_TRACE: bool>(
        &mut self,
        address: H160,
        target: H160,
    ) -> Result<(), ExitError>;
    /// Invoke a create operation.
    fn create<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        target_gas: Option<u64>,
    ) -> (ExitReason, Option<H160>, Bytes);

    /// Invoke a call operation.
    fn call<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Bytes,
        target_gas: Option<u64>,
        is_static: bool,
        context: CallContext,
    ) -> (ExitReason, Bytes);
}

pub trait Tracing {
    fn trace_opcode(&mut self, contract: &Contract, opcode: OpCode, stack: &Stack) {
        println!("Opcode:{:?} ({:?}), stack:{:?}",opcode, opcode as u8,stack.data());
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
