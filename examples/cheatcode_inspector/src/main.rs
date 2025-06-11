//! An example that shows how to implement a Foundry-style Solidity test cheatcode inspector.
//!
//! The code below mimics relevant parts of the implementation of the [`transact`](https://book.getfoundry.sh/cheatcodes/transact)
//! and [`rollFork(uint256 forkId, bytes32 transaction)`](https://book.getfoundry.sh/cheatcodes/roll-fork#rollfork) cheatcodes.
//! Both of these cheatcodes initiate transactions from a call step in the cheatcode inspector which is the most
//! advanced cheatcode use-case.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use revm::{
    context::{
        result::InvalidTransaction, BlockEnv, Cfg, CfgEnv, ContextTr, Evm, JournalOutput,
        LocalContext, TxEnv,
    },
    context_interface::{
        journaled_state::{AccountLoad, JournalCheckpoint, TransferError},
        result::EVMError,
        Block, JournalTr, Transaction,
    },
    database::InMemoryDB,
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EthPrecompiles, PrecompileProvider,
    },
    inspector::{inspectors::TracerEip3155, JournalExt},
    interpreter::{
        interpreter::EthInterpreter, CallInputs, CallOutcome, InterpreterResult, SStoreResult,
        SelfDestructResult, StateLoad,
    },
    primitives::{hardfork::SpecId, Address, HashSet, Log, StorageKey, StorageValue, B256, U256},
    state::{Account, Bytecode, EvmState},
    Context, Database, DatabaseCommit, InspectEvm, Inspector, Journal, JournalEntry,
};
use std::{convert::Infallible, fmt::Debug};

/// Backend for cheatcodes.
/// The problematic cheatcodes are only supported in fork mode, so we'll omit the non-fork behavior of the Foundry
/// `Backend`.
#[derive(Clone, Debug)]
struct Backend {
    /// In fork mode, Foundry stores (`JournaledState`, `Database`) pairs for each fork.
    journaled_state: Journal<InMemoryDB>,
    /// Counters to be able to assert that we mutated the object that we expected to mutate.
    method_with_inspector_counter: usize,
    method_without_inspector_counter: usize,
}

impl Backend {
    fn new(spec: SpecId, db: InMemoryDB) -> Self {
        let mut journaled_state = Journal::new(db);
        journaled_state.set_spec_id(spec);
        Self {
            journaled_state,
            method_with_inspector_counter: 0,
            method_without_inspector_counter: 0,
        }
    }
}

impl JournalTr for Backend {
    type Database = InMemoryDB;
    type FinalOutput = JournalOutput;

    fn new(database: InMemoryDB) -> Self {
        Self::new(SpecId::default(), database)
    }

    fn db_ref(&self) -> &Self::Database {
        self.journaled_state.db_ref()
    }

    fn db(&mut self) -> &mut Self::Database {
        self.journaled_state.db()
    }

    fn sload(
        &mut self,
        address: Address,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error> {
        self.journaled_state.sload(address, key)
    }

    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error> {
        self.journaled_state.sstore(address, key, value)
    }

    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue {
        self.journaled_state.tload(address, key)
    }

    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue) {
        self.journaled_state.tstore(address, key, value)
    }

    fn log(&mut self, log: Log) {
        self.journaled_state.log(log)
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, Infallible> {
        self.journaled_state.selfdestruct(address, target)
    }

    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) -> Result<(), <Self::Database as Database>::Error> {
        self.journaled_state
            .warm_account_and_storage(address, storage_keys)
    }

    fn warm_account(&mut self, address: Address) {
        self.journaled_state
            .warm_preloaded_addresses
            .insert(address);
    }

    fn warm_precompiles(&mut self, addresses: HashSet<Address>) {
        self.journaled_state.warm_precompiles(addresses)
    }

    fn precompile_addresses(&self) -> &HashSet<Address> {
        self.journaled_state.precompile_addresses()
    }

    fn set_spec_id(&mut self, spec_id: SpecId) {
        self.journaled_state.set_spec_id(spec_id);
    }

    fn touch_account(&mut self, address: Address) {
        self.journaled_state.touch_account(address);
    }

    fn transfer(
        &mut self,
        from: Address,
        to: Address,
        balance: U256,
    ) -> Result<Option<TransferError>, Infallible> {
        self.journaled_state.transfer(from, to, balance)
    }

    fn inc_account_nonce(&mut self, address: Address) -> Result<Option<u64>, Infallible> {
        Ok(self.journaled_state.inc_nonce(address))
    }

    fn load_account(&mut self, address: Address) -> Result<StateLoad<&mut Account>, Infallible> {
        self.journaled_state.load_account(address)
    }

    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, Infallible> {
        self.journaled_state.load_account_code(address)
    }

    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, Infallible> {
        self.journaled_state.load_account_delegated(address)
    }

    fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256) {
        self.journaled_state.set_code_with_hash(address, code, hash);
    }

    fn code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<revm::primitives::Bytes>, <Self::Database as Database>::Error> {
        self.journaled_state.code(address)
    }

    fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<B256>, <Self::Database as Database>::Error> {
        self.journaled_state.code_hash(address)
    }

    fn clear(&mut self) {
        self.journaled_state.clear();
    }

    fn checkpoint(&mut self) -> JournalCheckpoint {
        self.journaled_state.checkpoint()
    }

    fn checkpoint_commit(&mut self) {
        self.journaled_state.checkpoint_commit()
    }

    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        self.journaled_state.checkpoint_revert(checkpoint)
    }

    fn create_account_checkpoint(
        &mut self,
        caller: Address,
        address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        self.journaled_state
            .create_account_checkpoint(caller, address, balance, spec_id)
    }

    /// Returns call depth.
    #[inline]
    fn depth(&self) -> usize {
        self.journaled_state.depth()
    }

    fn finalize(&mut self) -> Self::FinalOutput {
        self.journaled_state.finalize()
    }
}

impl JournalExt for Backend {
    fn logs(&self) -> &[Log] {
        self.journaled_state.logs()
    }

    fn journal(&self) -> &[JournalEntry] {
        self.journaled_state.journal()
    }

    fn evm_state(&self) -> &EvmState {
        self.journaled_state.evm_state()
    }

    fn evm_state_mut(&mut self) -> &mut EvmState {
        self.journaled_state.evm_state_mut()
    }
}

/// Used in Foundry to provide extended functionality to cheatcodes.
/// The methods are called from the `Cheatcodes` inspector.
trait DatabaseExt: JournalTr {
    /// Mimics `DatabaseExt::transact`
    /// See `commit_transaction` for the generics
    fn method_that_takes_inspector_as_argument<
        InspectorT,
        BlockT,
        TxT,
        CfgT,
        InstructionProviderT,
        PrecompileT,
    >(
        &mut self,
        env: Env<BlockT, TxT, CfgT>,
        inspector: InspectorT,
    ) -> anyhow::Result<()>
    where
        InspectorT: Inspector<Context<BlockT, TxT, CfgT, InMemoryDB, Backend>, EthInterpreter>,
        BlockT: Block,
        TxT: Transaction,
        CfgT: Cfg,
        InstructionProviderT: InstructionProvider<
                Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                InterpreterTypes = EthInterpreter,
            > + Default,
        PrecompileT: PrecompileProvider<
                Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                Output = InterpreterResult,
            > + Default;

    /// Mimics `DatabaseExt::roll_fork_to_transaction`
    fn method_that_constructs_inspector<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>(
        &mut self,
        env: Env<BlockT, TxT, CfgT>,
    ) -> anyhow::Result<()>
    where
        BlockT: Block,
        TxT: Transaction,
        CfgT: Cfg,
        InstructionProviderT: InstructionProvider<
                Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                InterpreterTypes = EthInterpreter,
            > + Default,
        PrecompileT: PrecompileProvider<
                Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                Output = InterpreterResult,
            > + Default;
}

impl DatabaseExt for Backend {
    fn method_that_takes_inspector_as_argument<
        InspectorT,
        BlockT,
        TxT,
        CfgT,
        InstructionProviderT,
        PrecompileT,
    >(
        &mut self,
        env: Env<BlockT, TxT, CfgT>,
        inspector: InspectorT,
    ) -> anyhow::Result<()>
    where
        InspectorT: Inspector<Context<BlockT, TxT, CfgT, InMemoryDB, Backend>, EthInterpreter>,
        BlockT: Block,
        TxT: Transaction,
        CfgT: Cfg,
        InstructionProviderT: InstructionProvider<
                Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                InterpreterTypes = EthInterpreter,
            > + Default,
        PrecompileT: PrecompileProvider<
                Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                Output = InterpreterResult,
            > + Default,
    {
        commit_transaction::<InspectorT, BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>(
            self, env, inspector,
        )?;
        self.method_with_inspector_counter += 1;
        Ok(())
    }

    fn method_that_constructs_inspector<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>(
        &mut self,
        env: Env<BlockT, TxT, CfgT>,
    ) -> anyhow::Result<()>
    where
        BlockT: Block,
        TxT: Transaction,
        CfgT: Cfg,
        InstructionProviderT: InstructionProvider<
                Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                InterpreterTypes = EthInterpreter,
            > + Default,
        PrecompileT: PrecompileProvider<
                Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
                Output = InterpreterResult,
            > + Default,
    {
        let inspector = TracerEip3155::new(Box::new(std::io::sink()));
        commit_transaction::<
            // Generic interpreter types are not supported yet in the `Evm` implementation
            TracerEip3155,
            BlockT,
            TxT,
            CfgT,
            InstructionProviderT,
            PrecompileT,
        >(self, env, inspector)?;

        self.method_without_inspector_counter += 1;
        Ok(())
    }
}

/// An REVM inspector that intercepts calls to the cheatcode address and executes them with the help of the
/// `DatabaseExt` trait.
#[derive(Clone, Default)]
struct Cheatcodes<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT> {
    call_count: usize,
    phantom: core::marker::PhantomData<(BlockT, TxT, CfgT, InstructionProviderT, PrecompileT)>,
}

impl<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>
    Cheatcodes<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>
where
    BlockT: Block + Clone,
    TxT: Transaction + Clone,
    CfgT: Cfg + Clone,
    InstructionProviderT: InstructionProvider<
            Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            InterpreterTypes = EthInterpreter,
        > + Default,
    PrecompileT: PrecompileProvider<
            Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            Output = InterpreterResult,
        > + Default,
{
    fn apply_cheatcode(
        &mut self,
        context: &mut Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
    ) -> anyhow::Result<()> {
        // We cannot avoid cloning here, because we need to mutably borrow the context to get the journal.
        let block = context.block.clone();
        let tx = context.tx.clone();
        let cfg = context.cfg.clone();

        // `transact` cheatcode would do this
        context
            .journal()
            .method_that_takes_inspector_as_argument::<_, _, _, _, InstructionProviderT, PrecompileT>(
                Env {
                    block: block.clone(),
                    tx: tx.clone(),
                    cfg: cfg.clone(),
                },
                self,
            )?;

        // `rollFork(bytes32 transaction)` cheatcode would do this
        context
            .journal()
            .method_that_constructs_inspector::<_, _, _, InstructionProviderT, PrecompileT>(
                Env { block, tx, cfg },
            )?;
        Ok(())
    }
}

impl<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>
    Inspector<Context<BlockT, TxT, CfgT, InMemoryDB, Backend>>
    for Cheatcodes<BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>
where
    BlockT: Block + Clone,
    TxT: Transaction + Clone,
    CfgT: Cfg + Clone,
    InstructionProviderT: InstructionProvider<
            Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            InterpreterTypes = EthInterpreter,
        > + Default,
    PrecompileT: PrecompileProvider<
            Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            Output = InterpreterResult,
        > + Default,
{
    /// Note that precompiles are no longer accessible via `EvmContext::precompiles`.
    fn call(
        &mut self,
        context: &mut Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
        _inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        self.call_count += 1;
        // Don't apply cheatcodes recursively.
        if self.call_count == 1 {
            // Instead of calling unwrap here, we would want to return an appropriate call outcome based on the result
            // in a real project.
            self.apply_cheatcode(context).unwrap();
        }
        None
    }
}

/// EVM environment
#[derive(Clone, Debug)]
struct Env<BlockT, TxT, CfgT> {
    block: BlockT,
    tx: TxT,
    cfg: CfgT,
}

impl Env<BlockEnv, TxEnv, CfgEnv> {
    fn mainnet() -> Self {
        // `CfgEnv` is non-exhaustive, so we need to set the field after construction.
        let mut cfg = CfgEnv::default();
        cfg.disable_nonce_check = true;

        Self {
            block: BlockEnv::default(),
            tx: TxEnv::default(),
            cfg,
        }
    }
}

/// Executes a transaction and runs the inspector using the `Backend` as the state.
/// Mimics `commit_transaction` <https://github.com/foundry-rs/foundry/blob/25cc1ac68b5f6977f23d713c01ec455ad7f03d21/crates/evm/core/src/backend/mod.rs#L1931>
fn commit_transaction<InspectorT, BlockT, TxT, CfgT, InstructionProviderT, PrecompileT>(
    backend: &mut Backend,
    env: Env<BlockT, TxT, CfgT>,
    inspector: InspectorT,
) -> Result<(), EVMError<Infallible, InvalidTransaction>>
where
    InspectorT: Inspector<Context<BlockT, TxT, CfgT, InMemoryDB, Backend>, EthInterpreter>,
    BlockT: Block,
    TxT: Transaction,
    CfgT: Cfg,
    InstructionProviderT: InstructionProvider<
            Context = Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            InterpreterTypes = EthInterpreter,
        > + Default,
    PrecompileT: PrecompileProvider<
            Context<BlockT, TxT, CfgT, InMemoryDB, Backend>,
            Output = InterpreterResult,
        > + Default,
{
    // Create new journaled state and backend with the same DB and journaled state as the original for the transaction.
    // This new backend and state will be discarded after the transaction is done and the changes are applied to the
    // original backend.
    // Mimics https://github.com/foundry-rs/foundry/blob/25cc1ac68b5f6977f23d713c01ec455ad7f03d21/crates/evm/core/src/backend/mod.rs#L1950-L1953
    let new_backend = backend.clone();

    let context = Context {
        tx: env.tx,
        block: env.block,
        cfg: env.cfg,
        journaled_state: new_backend,
        chain: (),
        local: LocalContext::default(),
        error: Ok(()),
    };

    let mut evm = Evm::new_with_inspector(
        context,
        inspector,
        InstructionProviderT::default(),
        PrecompileT::default(),
    );
    let result = evm.inspect_replay()?;

    // Persist the changes to the original backend.
    backend.journaled_state.database.commit(result.state);
    update_state(
        &mut backend.journaled_state.inner.state,
        &mut backend.journaled_state.database,
    )?;

    Ok(())
}

/// Mimics <https://github.com/foundry-rs/foundry/blob/25cc1ac68b5f6977f23d713c01ec455ad7f03d21/crates/evm/core/src/backend/mod.rs#L1968>
/// Omits persistent accounts (accounts that should be kept persistent when switching forks) for simplicity.
fn update_state<DB: Database>(state: &mut EvmState, db: &mut DB) -> Result<(), DB::Error> {
    for (addr, acc) in state.iter_mut() {
        acc.info = db.basic(*addr)?.unwrap_or_default();
        for (key, val) in acc.storage.iter_mut() {
            val.present_value = db.storage(*addr, *key)?;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let backend = Backend::new(SpecId::default(), InMemoryDB::default());
    let mut inspector = Cheatcodes::<
        BlockEnv,
        TxEnv,
        CfgEnv,
        EthInstructions<EthInterpreter, Context<BlockEnv, TxEnv, CfgEnv, InMemoryDB, Backend>>,
        EthPrecompiles,
    >::default();
    let env = Env::mainnet();

    let context = Context {
        tx: env.tx,
        block: env.block,
        cfg: env.cfg,
        journaled_state: backend,
        chain: (),
        local: LocalContext::default(),
        error: Ok(()),
    };

    let mut evm = Evm::new_with_inspector(
        context,
        &mut inspector,
        EthInstructions::default(),
        EthPrecompiles::default(),
    );
    evm.inspect_replay()?;

    // Sanity check
    assert_eq!(evm.inspector.call_count, 2);
    assert_eq!(evm.journaled_state.method_with_inspector_counter, 1);
    assert_eq!(evm.journaled_state.method_without_inspector_counter, 1);

    Ok(())
}
