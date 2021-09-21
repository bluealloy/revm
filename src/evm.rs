use std::{marker::PhantomData, process::ExitStatus};

use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

use crate::{
    error::{ExitError, ExitReason, ExitSucceed},
    opcode::OpCode,
    spec::Spec,
    machine::Stack,
    db::Database,
    subrutine::SubRutine,
    Basic, Context, CreateScheme, GlobalContext, Log, Machine, Transfer,
};
use bytes::Bytes;

pub struct EVM<'a, SPEC: Spec> {
    db: &'a mut dyn Database,
    global_context: GlobalContext,
    subrutine: SubRutine,
    gas: U256,
    phantomdata: PhantomData<SPEC>,
}

impl<'a, SPEC: Spec> EVM<'a, SPEC> {
    pub fn new(db: &'a mut dyn Database, global_context: GlobalContext) -> Self {
        let gas = global_context.gas_limit.clone();
        Self {
            db,
            global_context,
            subrutine: SubRutine::new(),
            gas,
            phantomdata: PhantomData,
        }
    }

    /// Get the create address from given scheme.
    pub fn create_address(&mut self, scheme: CreateScheme) -> H160 {
        match scheme {
            CreateScheme::Create2 {
                caller,
                code_hash,
                salt,
            } => {
                let mut hasher = Keccak256::new();
                hasher.update(&[0xff]);
                hasher.update(&caller[..]);
                hasher.update(&salt[..]);
                hasher.update(&code_hash[..]);
                H256::from_slice(hasher.finalize().as_slice()).into()
            }
            CreateScheme::Legacy { caller } => {
                let (nonce, _) = self.balance(caller);
                let mut stream = rlp::RlpStream::new_list(2);
                stream.append(&caller);
                stream.append(&nonce);
                H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
            }
            CreateScheme::Fixed(naddress) => naddress,
        }
    }

    fn create_inner(
        &mut self,
        caller: H160,
        scheme: CreateScheme,
        value: U256,
        init_code: Vec<u8>,
        target_gas: Option<u64>,
        take_l64: bool,
    ) -> (ExitReason, Option<H160>, Vec<u8>) {
        //todo!()

        // TODO set caller/contract_add/precompiles as hot access

        // trace call
        self.trace_call();
        // check depth of calls
        if self.subrutine.depth() > SPEC::call_stack_limit {
            return (ExitError::CallTooDeep.into(), None, Vec::new());
        }

        // check balance of caller and value
        if self.balance(caller).0 < value {
            return (ExitError::OutOfFund.into(), None, Vec::new());
        }
        // create address
        let address = self.create_address(scheme);
        // TODO wtf is l64 gas reduction. Check spec. Return gas and set gas_limit
        // inc nonce of caller
        self.subrutine.inc_nonce(caller);
        // enter into subroutine
        let checkpoint = self.subrutine.create_checkpoint();
        // TODO check for code colision by checking nonce and code of created address
        // TODO reset storage to be sure that we dont overlap anything

        // transfer value to contract address
        if let Err(e) = self.subrutine.transfer(caller, address, value) {
            let _ = self.subrutine.exit_revert(checkpoint);
            return (ExitReason::Error(e), None, Vec::new());
        }
        // inc nonce of contract
        if SPEC::create_increase_nonce {
            self.subrutine.inc_nonce(address);
        }
        // create new machine and execute init function
        let context = Context {
            address,
            caller,
            apparent_value: value,
        };
        let mut machine = Machine::new(init_code, context);
        let res = machine.run::<Self, SPEC>(self);
        // handler error if present on execution
        match res {
            ExitReason::Succeed(s) => {
                // if ok, check contract creation limit and calculate gas deduction on output len.
                let out = machine.return_value();
                if let Some(limit) = SPEC::create_contract_limit {
                    if out.len() > limit {
                        // TODO reduce gas and return
                        self.subrutine.exit_discard(checkpoint);
                        return (ExitError::CreateContractLimit.into(), None, Vec::new());
                    }
                }
                // dummy return TODO proper handling
                
                let e = self.subrutine.exit_commit(checkpoint);
                (
                    ExitReason::Succeed(ExitSucceed::Returned),
                    Some(address),
                    Vec::new(),
                )
            }
            ExitReason::Revert(revert) => {
                let _ = self.subrutine.exit_revert(checkpoint);
                (ExitReason::Revert(revert), None, machine.return_value())
            }
            ExitReason::Error(_) | ExitReason::Fatal(_) => {
                let _ = self.subrutine.exit_discard(checkpoint);
                (res.clone(), None, machine.return_value())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_inner(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Vec<u8>,
        target_gas: Option<u64>,
        is_static: bool,
        take_l64: bool,
        take_stipend: bool,
        context: Context,
    ) -> (ExitReason, Vec<u8>) {
        todo!()
    }
}

impl<'a, SPEC: Spec> Handler for EVM<'a, SPEC> {
    fn global_context(&self) -> &GlobalContext {
        &self.global_context
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }

    fn balance(&mut self, address: H160) -> (U256, bool) {
        if let Some(acc) = self.subrutine.known_account(&address).map(|acc| acc.basic) {
            // set it as hot access
            (acc.balance, false)
        } else {
            // set it as cold access
            (self.db.basic(address).balance, true) //TODO LOAD IT
        }
    }

    fn nonce(&mut self, address: H160) -> (U256, bool) {
        if let Some(acc) = self.subrutine.known_account(&address).map(|acc| acc.basic) {
            // set it as hot access
            (acc.nonce, false)
        } else {
            // set it as cold access
            (self.db.basic(address).nonce, true) //TODO LOAD IT
        }
    }

    fn code(&mut self, address: H160) -> (Bytes, bool) {
        if let Some(acc) = self.subrutine.known_account(&address) {
            // set it as hot access
            (acc.code.clone().unwrap(), false)
        } else {
            // set it as cold access
            (self.db.code(address), true) //TODO LOAD IT
        }
    }

    fn storage(&mut self, address: H160, index: H256) -> (H256, bool) {
        // account is allways hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        if let Some(&slot) = self
            .subrutine
            .known_account(&address)
            .unwrap()
            .storage
            .get(&index)
        {
            // set it as hot access
            (slot, true)
        } else {
            // set it as cold access
            (self.db.storage(address, index), false)
        }
    }

    fn original_storage(&mut self, address: H160, index: H256) -> H256 {
        self.db.original_storage(address, index).unwrap_or_default()
    }

    // TODO Used for selfdestruct gas calculation. Should probably be merged with delete call
    // fn deleted(&self, address: H160) -> bool {
    //     true
    // }

    // This two next functions should be removed. THhis information should be passed when asking needed data/account/slot
    fn is_cold(&self, address: H160) -> bool {
        true
    }

    fn is_cold_storage(&self, address: H160, index: H256) -> bool {
        true
    }

    fn gas_left(&self) -> U256 {
        self.gas
    }

    // TODO check return value, should it be is_cold or maybe original value
    fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<bool, ExitError> {
        self.subrutine.set_storage(address, index, value)
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes) {
        let log = Log {
            address,
            topics,
            data,
        };
        self.subrutine.log(log);
    }

    // DO it later :)
    fn mark_delete<const CALL_TRACE: bool>(
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
        init_code: Vec<u8>,
        target_gas: Option<u64>,
    ) -> (ExitReason, Option<H160>, Vec<u8>) {
        self.create_inner(caller, scheme, value, init_code, target_gas, true)
    }

    fn call<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Vec<u8>,
        target_gas: Option<u64>,
        is_static: bool,
        context: Context,
    ) -> (ExitReason, Vec<u8>) {
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

impl<'a, SPEC: Spec> Tracing for EVM<'a, SPEC> {}
impl<'a, SPEC: Spec> ExtHandler2 for EVM<'a, SPEC> {}
impl<'a, SPEC: Spec> ExtHandler for EVM<'a, SPEC> {}

/// EVM context handler.
pub trait Handler {
    /// Get global const context of evm execution
    fn global_context(&self) -> &GlobalContext;

    /// Get environmental block hash.
    fn block_hash(&mut self, number: U256) -> H256;
    /// Get balance of address.
    fn balance(&mut self, address: H160) -> (U256, bool);
    /// Get balance of address.
    fn nonce(&mut self, address: H160) -> (U256, bool);

    /// Get code of address.
    fn code(&mut self, address: H160) -> (Bytes, bool);
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: H256) -> (H256, bool);
    /// Get original storage value of address at index.
    fn original_storage(&mut self, address: H160, index: H256) -> H256;
    /// Check whether an address exists.
    //fn exists(&self, address: H160) -> bool;
    /// Check whether an address has already been deleted. Should be merged with selfdestruct mark_delete call
    // fn deleted(&self, address: H160) -> bool;
    /// Checks if the address or (address, index) pair has been previously accessed
    /// (or set in `accessed_addresses` / `accessed_storage_keys` via an access list
    /// transaction).
    /// References:
    /// * https://eips.ethereum.org/EIPS/eip-2929
    /// * https://eips.ethereum.org/EIPS/eip-2930
    /// TODO REMOVE THIS STUFF
    fn is_cold(&self, address: H160) -> bool;
    fn is_cold_storage(&self, address: H160, index: H256) -> bool;

    /// Get the gas left value. It contacts gasometer
    fn gas_left(&self) -> U256;
    /// Set storage value of address at index. Return if slot is cold/hot access.
    fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<bool, ExitError>;
    /// Create a log owned by address with given topics and data.
    fn log(&mut self, address: H160, topics: Vec<H256>, data: Bytes);
    /// Mark an address to be deleted, with funds transferred to target.
    fn mark_delete<const CALL_TRACE: bool>(
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
        init_code: Vec<u8>,
        target_gas: Option<u64>,
    ) -> (ExitReason, Option<H160>, Vec<u8>);

    /// Invoke a call operation.
    fn call<const CALL_TRACE: bool, const GAS_TRACE: bool, const OPCODE_TRACE: bool>(
        &mut self,
        code_address: H160,
        transfer: Option<Transfer>,
        input: Vec<u8>,
        target_gas: Option<u64>,
        is_static: bool,
        context: Context,
    ) -> (ExitReason, Vec<u8>);
}

pub trait Tracing {
    fn trace_opcode(&mut self, context: &Context, opcode: OpCode, stack: &Stack) {}
    fn trace_call(&mut self) {}
}

pub trait ExtHandler2: Handler {
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
        self.global_context().gas_price
    }
    /// Get execution origin.
    fn origin(&self) -> H160 {
        self.global_context().origin
    }
    /// Get environmental block number.
    fn block_number(&self) -> U256 {
        self.global_context().block_number
    }
    /// Get environmental coinbase.
    fn block_coinbase(&self) -> H160 {
        self.global_context().block_coinbase
    }
    /// Get environmental block timestamp.
    fn block_timestamp(&self) -> U256 {
        self.global_context().block_timestamp
    }
    /// Get environmental block difficulty.
    fn block_difficulty(&self) -> U256 {
        self.global_context().block_difficulty
    }
    /// Get environmental gas limit.
    fn block_gas_limit(&self) -> U256 {
        self.global_context().block_gas_limit
    }
    /// Get environmental chain ID.
    fn chain_id(&self) -> U256 {
        self.global_context().chain_id
    }
}

// TODO cleanup this mess of traits
pub trait ExtHandler: ExtHandler2 + Tracing {}
