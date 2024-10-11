use crate::{
    builder::{HandlerStage, RwasmBuilder, SetGenericStage},
    db::EmptyDB,
    interpreter::{
        CallInputs,
        CallOutcome,
        CreateInputs,
        CreateOutcome,
        Gas,
        Host,
        InstructionResult,
    },
    primitives::{
        Address,
        BlockEnv,
        Bytecode,
        Bytes,
        CfgEnv,
        EVMError,
        EVMResult,
        EnvWithHandlerCfg,
        ExecutionResult,
        HandlerCfg,
        Log,
        LogData,
        ResultAndState,
        SpecId,
        TransactTo,
        TxEnv,
        B256,
        U256,
    },
    Context,
    ContextWithHandlerCfg,
    Database,
    DatabaseCommit,
    EvmContext,
    FrameResult,
    Handler,
    JournalEntry,
};
use core::{cell::RefCell, fmt, ops::Deref};
use fluentbase_core::{blended::BlendedRuntime, helpers::exit_code_from_evm_error};
use fluentbase_runtime::RuntimeContext;
use fluentbase_sdk::{
    runtime::RuntimeContextWrapper,
    Account,
    AccountStatus,
    BlockContext,
    CallPrecompileResult,
    ContextFreeNativeAPI,
    ContractContext,
    DestroyedAccountResult,
    ExitCode,
    IsColdAccess,
    JournalCheckpoint,
    NativeAPI,
    SovereignAPI,
    SovereignStateResult,
    TxContext,
    F254,
};
use revm_interpreter::StateLoad;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Rwasm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler is a component of the of EVM that contains all the logic. Handler contains
    /// specification id and it different depending on the specified fork.
    pub handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
}

impl<EXT, DB> fmt::Debug for Rwasm<'_, EXT, DB>
where
    EXT: fmt::Debug,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EXT, DB: Database + DatabaseCommit> Rwasm<'_, EXT, DB> {
    /// Commit the changes to the database.
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.db.commit(state);
        Ok(result)
    }
}

impl<'a> Rwasm<'a, (), EmptyDB> {
    /// Returns evm builder with an empty database and empty external context.
    pub fn builder() -> RwasmBuilder<'a, SetGenericStage, (), EmptyDB> {
        RwasmBuilder::default()
    }
}

impl<'a, EXT, DB: Database> Rwasm<'a, EXT, DB> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<EXT, DB>,
        handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    ) -> Rwasm<'a, EXT, DB> {
        context.evm.journaled_state.set_spec_id(handler.cfg.spec_id);
        Rwasm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> RwasmBuilder<'a, HandlerStage, EXT, DB> {
        RwasmBuilder::<'a, HandlerStage, EXT, DB>::new(self)
    }
}

impl<EXT, DB: Database> Rwasm<'_, EXT, DB> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> SpecId {
        self.handler.cfg.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balances to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post-execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)
            .map_err(|e| {
                self.clear();
                e
            })?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Pre verify transaction inner.
    #[inline]
    fn preverify_transaction_inner(&mut self) -> Result<u64, EVMError<DB::Error>> {
        self.handler.validation().env(&self.context.evm.env)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;
        Ok(initial_gas_spend)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self.preverify_transaction_inner().map_err(|e| {
            self.clear();
            e
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of handler configuration
    #[inline]
    pub fn handler_cfg(&self) -> &HandlerCfg {
        &self.handler.cfg
    }

    /// Returns the reference of Env configuration
    #[inline]
    pub fn cfg(&self) -> &CfgEnv {
        &self.context.env().cfg
    }

    /// Returns the mutable reference of Env configuration
    #[inline]
    pub fn cfg_mut(&mut self) -> &mut CfgEnv {
        &mut self.context.evm.env.cfg
    }

    /// Returns the reference of transaction
    #[inline]
    pub fn tx(&self) -> &TxEnv {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut TxEnv {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &DB {
        &self.context.evm.db
    }

    /// Returns the mutable reference of a database
    #[inline]
    pub fn db_mut(&mut self) -> &mut DB {
        &mut self.context.evm.db
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &BlockEnv {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut BlockEnv {
        &mut self.context.evm.env.block
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(&mut self, spec_id: SpecId) {
        self.handler.modify_spec_id(spec_id);
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// Returns database and [`EnvWithHandlerCfg`].
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(self) -> (DB, EnvWithHandlerCfg) {
        (
            self.context.evm.inner.db,
            EnvWithHandlerCfg {
                env: self.context.evm.inner.env,
                handler_cfg: self.handler.cfg,
            },
        )
    }

    /// Returns [Context] and [HandlerCfg].
    #[inline]
    pub fn into_context_with_handler_cfg(self) -> ContextWithHandlerCfg<EXT, DB> {
        ContextWithHandlerCfg::new(self.context, self.handler.cfg)
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // load precompiles
        let precompiles = pre_exec.load_precompiles();
        ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.gas_limit - initial_gas_spend;

        // load an EVM loader account to access storage slots
        // let (evm_storage, _) = ctx.evm.load_account(PRECOMPILE_EVM)?;
        // evm_storage.info.nonce = 1;
        // ctx.evm.touch(&PRECOMPILE_EVM);

        let mut result = match ctx.evm.env.tx.transact_to.clone() {
            TransactTo::Call(_) => {
                let inputs = CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap();
                let result = self.call_inner(inputs)?;
                FrameResult::Call(result)
            }
            TransactTo::Create => {
                let inputs = CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap();
                let result = self.create_inner(inputs)?;
                FrameResult::Create(result)
            }
        };

        let ctx = &mut self.context;

        // handle output of call/create calls.
        self.handler
            .execution()
            .last_frame_return(ctx, &mut result)?;

        let post_exec = self.handler.post_execution();
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, result.gas())?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, result.gas())?;
        // Returns output of transaction.
        post_exec.output(ctx, result)
    }

    /// EVM create opcode for both initial CREATE and CREATE2 opcodes.
    fn create_inner(
        &mut self,
        create_inputs: Box<CreateInputs>,
    ) -> Result<CreateOutcome, EVMError<DB::Error>> {
        let runtime_context = RuntimeContext::default()
            .with_depth(0u32)
            .with_fuel_limit(create_inputs.gas_limit);
        let native_sdk = RuntimeContextWrapper::new(runtime_context);
        let mut sdk = RwasmDbWrapper::new(&mut self.context.evm, native_sdk);

        let result = BlendedRuntime::new(&mut sdk).create(create_inputs);
        Ok(result)
    }

    /// Main contract call of the EVM.
    fn call_inner(
        &mut self,
        call_inputs: Box<CallInputs>,
    ) -> Result<CallOutcome, EVMError<DB::Error>> {
        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if call_inputs.call_value() == U256::ZERO {
            self.context.evm.load_account(call_inputs.target_address)?;
            self.context
                .evm
                .journaled_state
                .touch(&call_inputs.target_address);
        }

        let runtime_context = RuntimeContext::default()
            .with_depth(0u32)
            .with_fuel_limit(call_inputs.gas_limit);
        let native_sdk = RuntimeContextWrapper::new(runtime_context);
        let mut sdk = RwasmDbWrapper::new(&mut self.context.evm, native_sdk);

        let result = BlendedRuntime::new(&mut sdk).call(call_inputs);
        Ok(result)
    }
}

pub struct RwasmDbWrapper<'a, API: NativeAPI, DB: Database> {
    evm_context: RefCell<&'a mut EvmContext<DB>>,
    native_sdk: API,
    block_context: BlockContext,
    tx_context: TxContext,
}

impl<'a, API: NativeAPI, DB: Database> RwasmDbWrapper<'a, API, DB> {
    pub fn new(
        evm_context: &'a mut EvmContext<DB>,
        native_sdk: API,
    ) -> RwasmDbWrapper<'a, API, DB> {
        let block_context = BlockContext::from(evm_context.env.deref());
        let tx_context = TxContext::from(evm_context.env.deref());
        RwasmDbWrapper {
            evm_context: RefCell::new(evm_context),
            native_sdk,
            block_context,
            tx_context,
        }
    }
}

impl<'a, API: NativeAPI, DB: Database> ContextFreeNativeAPI for RwasmDbWrapper<'a, API, DB> {
    fn keccak256(data: &[u8]) -> B256 {
        API::keccak256(data)
    }

    fn sha256(data: &[u8]) -> B256 {
        API::sha256(data)
    }

    fn poseidon(data: &[u8]) -> F254 {
        API::poseidon(data)
    }

    fn poseidon_hash(fa: &F254, fb: &F254, fd: &F254) -> F254 {
        API::poseidon_hash(fa, fb, fd)
    }

    fn ec_recover(digest: &B256, sig: &[u8; 64], rec_id: u8) -> [u8; 65] {
        API::ec_recover(digest, sig, rec_id)
    }

    fn debug_log(message: &str) {
        API::debug_log(message)
    }
}

impl<'a, API: NativeAPI, DB: Database> SovereignAPI for RwasmDbWrapper<'a, API, DB> {
    fn native_sdk(&self) -> &impl NativeAPI {
        &self.native_sdk
    }

    fn block_context(&self) -> &BlockContext {
        &self.block_context
    }

    fn tx_context(&self) -> &TxContext {
        &self.tx_context
    }

    fn contract_context(&self) -> Option<&ContractContext> {
        None
    }

    fn checkpoint(&self) -> JournalCheckpoint {
        let mut ctx = self.evm_context.borrow_mut();
        let (a, b) = ctx.journaled_state.checkpoint().into();
        JournalCheckpoint(a, b)
    }

    fn commit(&mut self) -> SovereignStateResult {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.checkpoint_commit();
        SovereignStateResult::default()
    }

    fn rollback(&mut self, checkpoint: JournalCheckpoint) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state
            .checkpoint_revert((checkpoint.0, checkpoint.1).into());
    }

    fn write_account(&mut self, account: Account, status: AccountStatus) {
        let mut ctx = self.evm_context.borrow_mut();
        // load account with this address from journaled state
        let StateLoad {
            data: db_account, ..
        } = ctx
            .load_account_with_code(account.address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let old_nonce = db_account.info.nonce;
        // copy all account info fields
        db_account.info.balance = account.balance;
        db_account.info.nonce = account.nonce;
        db_account.info.code_hash = account.code_hash;
        // if this is an account deployment, then mark is as created (needed for SELFDESTRUCT)
        if status == AccountStatus::NewlyCreated {
            db_account.mark_created();
            let last_journal = ctx.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::AccountCreated {
                address: account.address,
            });
        }
        // if nonce has changed, then inc nonce as well
        if account.nonce - old_nonce == 1 {
            let last_journal = ctx.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::NonceChange {
                address: account.address,
            });
        }
        // mark an account as touched
        ctx.journaled_state.touch(&account.address);
    }

    fn destroy_account(&mut self, address: &Address, target: &Address) -> DestroyedAccountResult {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .selfdestruct(*address, *target)
            .map_err(|_| "unexpected EVM self destruct error")
            .unwrap();
        DestroyedAccountResult {
            had_value: result.had_value,
            target_exists: result.target_exists,
            is_cold: result.is_cold,
            previously_destroyed: result.previously_destroyed,
        }
    }

    fn account(&self, address: &Address) -> (Account, bool) {
        let mut ctx = self.evm_context.borrow_mut();
        let StateLoad {
            data: account,
            is_cold,
        } = ctx
            .load_account(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let mut account = Account::from(account.info.clone());
        account.address = *address;
        (account, is_cold)
    }

    fn account_committed(&self, _address: &Address) -> (Account, IsColdAccess) {
        todo!()
    }

    fn write_preimage(&mut self, address: Address, hash: B256, preimage: Bytes) {
        let mut ctx = self.evm_context.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .load_account(address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if account.info.code_hash == hash {
            ctx.journaled_state
                .set_code_with_hash(address, Bytecode::new_raw(preimage), hash);
            return;
        }
        // calculate preimage address
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if !preimage_account.is_empty() {
            assert_eq!(
                preimage_account.info.code_hash, hash,
                "unexpected preimage hash"
            );
            return;
        }
        // set default preimage account fields
        preimage_account.info.nonce = 1;
        preimage_account.info.code_hash = hash;
        // write preimage as a bytecode for the account
        ctx.journaled_state
            .set_code_with_hash(preimage_address, Bytecode::new_raw(preimage), hash);
        // // remember code hash
        // ctx.sstore(
        //     PRECOMPILE_EVM,
        //     U256::from_le_bytes(address.into_word().0),
        //     U256::from_le_bytes(hash.0),
        // )
        // .map_err(|_| panic!("database error"))
        // .unwrap();
    }

    fn preimage(&self, address: &Address, hash: &B256) -> Option<Bytes> {
        let mut ctx = self.evm_context.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .load_account_with_code(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if account.info.code_hash == *hash {
            return account.info.code.as_ref().map(|v| v.original_bytes());
        }
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        preimage_account
            .info
            .code
            .as_ref()
            .map(|v| v.original_bytes())
    }

    fn preimage_size(&self, address: &Address, hash: &B256) -> u32 {
        let mut ctx = self.evm_context.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .load_account_with_code(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if account.info.code_hash == *hash {
            return account
                .info
                .code
                .as_ref()
                .map(|v| v.len() as u32)
                .unwrap_or_default();
        }
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        preimage_account
            .info
            .code
            .as_ref()
            .map(|v| v.len() as u32)
            .unwrap_or_default()
    }

    fn write_storage(&mut self, address: Address, slot: U256, value: U256) -> IsColdAccess {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .sstore(address, slot, value)
            .map_err(|_| panic!("failed to update storage slot"))
            .unwrap();
        result.is_cold
    }

    fn storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.evm_context.borrow_mut();
        let load_result = ctx
            .load_account_delegated(*address)
            .unwrap_or_else(|_| panic!("internal storage error"));
        if load_result.is_empty {
            return (U256::ZERO, load_result.is_cold);
        }
        let state_load = ctx
            .sload(*address, *slot)
            .ok()
            .expect("failed to read storage slot");
        (state_load.data, state_load.is_cold)
    }

    fn committed_storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.evm_context.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .load_account(*address)
            .map_err(|_| panic!("failed to load account"))
            .unwrap();
        if account.is_created() {
            return (U256::ZERO, true);
        }
        let value = ctx
            .db
            .storage(*address, *slot)
            .ok()
            .expect("failed to read storage slot");
        (value, true)
    }

    fn write_transient_storage(&mut self, address: Address, index: U256, value: U256) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.tstore(address, index, value);
    }

    fn transient_storage(&self, address: &Address, index: &U256) -> U256 {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.tload(*address, *index)
    }

    fn write_log(&mut self, address: Address, data: Bytes, topics: Vec<B256>) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.log(Log {
            address,
            data: LogData::new_unchecked(topics, data),
        });
    }

    //noinspection RsBorrowChecker
    fn precompile(
        &self,
        address: &Address,
        input: &Bytes,
        gas: u64,
    ) -> Option<CallPrecompileResult> {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .call_precompile(&address, input, Gas::new(gas))
            .unwrap_or(None)?;
        Some(CallPrecompileResult {
            output: result.output,
            exit_code: exit_code_from_evm_error(result.result),
            gas_remaining: result.gas.remaining(),
            gas_refund: result.gas.refunded(),
        })
    }

    fn is_precompile(&self, address: &Address) -> bool {
        let ctx = self.evm_context.borrow_mut();
        ctx.journaled_state
            .warm_preloaded_addresses
            .contains(address)
    }

    fn transfer(
        &mut self,
        from: &mut Account,
        to: &mut Account,
        value: U256,
    ) -> Result<(), ExitCode> {
        Account::transfer(from, to, value)?;
        let mut ctx = self.evm_context.borrow_mut();
        ctx.transfer(&from.address, &to.address, value)
            .map_err(|_| panic!("unexpected EVM transfer error"))
            .unwrap()
            .and_then(|err| -> Option<InstructionResult> {
                panic!(
                    "it seems there is an account balance mismatch between ECL and REVM: {:?}",
                    err
                );
            });
        Ok(())
    }
}
