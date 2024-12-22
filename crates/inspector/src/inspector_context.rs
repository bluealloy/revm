
use auto_impl::auto_impl;
use core::mem::MaybeUninit;
use derive_where::derive_where;
use revm::{
    bytecode::opcode::OpCode,
    context::{block::BlockEnv, cfg::CfgEnv, tx::TxEnv, Cfg, JournaledState},
    context_interface::{
        block::BlockSetter,
        journaled_state::{AccountLoad, Eip7702CodeLoad},
        result::EVMError,
        transaction::TransactionSetter,
        Block, BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalStateGetter,
        JournalStateGetterDBError, Transaction, TransactionGetter,
    },
    database_interface::{Database, EmptyDB},
    handler::{
        EthExecution, EthFrame, EthHandler, EthPostExecution, EthPreExecution,
        EthPrecompileProvider, EthValidation, FrameResult,
    },
    handler_interface::{Frame, FrameOrResultGen, PrecompileProvider},
    interpreter::{
        instructions::host::{log, selfdestruct},
        interpreter::{EthInterpreter, InstructionProvider},
        interpreter_types::{Jumps, LoopControl},
        table::{self, CustomInstruction},
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput, Host,
        Instruction, InstructionResult, Interpreter, InterpreterTypes, SStoreResult,
        SelfDestructResult, StateLoad,
    },
    precompile::PrecompileErrors,
    primitives::{Address, Bytes, Log, B256, U256},
    specification::hardfork::SpecId,
    Context, Error, Evm, JournalEntry,
};
use std::{rc::Rc, vec::Vec};

use crate::{GetInspector, Inspector, InspectorCtx};
use crate::journal::{ JournalExt, JournalExtGetter};

/// EVM context contains data that EVM needs for execution.
#[derive_where(Clone, Debug; INSP, BLOCK, CFG, CHAIN, TX, DB,JOURNAL, <DB as Database>::Error)]
pub struct InspectorContext<
    INSP,
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv<SpecId>,
    DB: Database = EmptyDB,
    JOURNAL: Journal<Database = DB> = JournaledState<DB>,
    CHAIN = (),
> {
    pub inner: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
    pub inspector: INSP,
    pub frame_input_stack: Vec<FrameInput>,
}


impl<
        INSP,
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB>,
        CHAIN,
    > InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    pub fn new(inner: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, inspector: INSP) -> Self {
        Self {
            inner,
            inspector,
            frame_input_stack: Vec::new(),
        }
    }
}

impl<
        INSP: GetInspector,
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB>,
        CHAIN,
    > Host for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type BLOCK = BLOCK;
    type TX = TX;
    type CFG = CFG;

    fn tx(&self) -> &Self::TX {
        &self.inner.tx
    }

    fn block(&self) -> &Self::BLOCK {
        &self.inner.block
    }

    fn cfg(&self) -> &Self::CFG {
        &self.inner.cfg
    }

    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        self.inner.block_hash(requested_number)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<AccountLoad> {
        self.inner.load_account_delegated(address)
    }

    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.inner.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<Eip7702CodeLoad<Bytes>> {
        // TODO : Remove duplicated function name.
        <Context<_, _, _, _, _, _> as Host>::code(&mut self.inner, address)
    }

    fn code_hash(&mut self, address: Address) -> Option<Eip7702CodeLoad<B256>> {
        <Context<_, _, _, _, _, _> as Host>::code_hash(&mut self.inner, address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>> {
        self.inner.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.inner.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.inner.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.inner.tstore(address, index, value)
    }

    fn log(&mut self, log: Log) {
        self.inner.log(log);
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.inner.selfdestruct(address, target)
    }
}


impl<INSP, BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> InspectorCtx
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    INSP: GetInspector<
        Inspector: Inspector<
            Context = Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    type IT = EthInterpreter<()>;

    fn step(&mut self, interp: &mut Interpreter<Self::IT>) {
        self.inspector.get_inspector().step(interp, &mut self.inner);
    }

    fn step_end(&mut self, interp: &mut Interpreter<Self::IT>) {
        self.inspector
            .get_inspector()
            .step_end(interp, &mut self.inner);
    }

    fn initialize_interp(&mut self, interp: &mut Interpreter<Self::IT>) {
        self.inspector
            .get_inspector()
            .initialize_interp(interp, &mut self.inner);
    }
    fn inspector_log(&mut self, interp: &mut Interpreter<Self::IT>, log: &Log) {
        self.inspector
            .get_inspector()
            .log(interp, &mut self.inner, log);
    }

    fn frame_start(&mut self, frame_input: &mut FrameInput) -> Option<FrameResult> {
        let insp = self.inspector.get_inspector();
        let context = &mut self.inner;
        match frame_input {
            FrameInput::Call(i) => {
                if let Some(output) = insp.call(context, i) {
                    return Some(FrameResult::Call(output));
                }
            }
            FrameInput::Create(i) => {
                if let Some(output) = insp.create(context, i) {
                    return Some(FrameResult::Create(output));
                }
            }
            FrameInput::EOFCreate(i) => {
                if let Some(output) = insp.eofcreate(context, i) {
                    return Some(FrameResult::EOFCreate(output));
                }
            }
        }
        self.frame_input_stack.push(frame_input.clone());
        None
    }

    fn frame_end(&mut self, frame_output: &mut FrameResult) {
        let insp = self.inspector.get_inspector();
        let context = &mut self.inner;
        let frame_input = self.frame_input_stack.pop().expect("Frame pushed");
        match frame_output {
            FrameResult::Call(outcome) => {
                let FrameInput::Call(i) = frame_input else {
                    panic!("FrameInput::Call expected");
                };
                insp.call_end(context, &i, outcome);
            }
            FrameResult::Create(outcome) => {
                let FrameInput::Create(i) = frame_input else {
                    panic!("FrameInput::Create expected");
                };
                insp.create_end(context, &i, outcome);
            }
            FrameResult::EOFCreate(outcome) => {
                let FrameInput::EOFCreate(i) = frame_input else {
                    panic!("FrameInput::EofCreate expected");
                };
                insp.eofcreate_end(context, &i, outcome);
            }
        }
    }

    fn inspector_selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        self.inspector
            .get_inspector()
            .selfdestruct(contract, target, value)
    }
}

impl<INSP, BLOCK, TX, CFG: Cfg, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> CfgGetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Cfg = CFG;

    fn cfg(&self) -> &Self::Cfg {
        &self.inner.cfg
    }
}

impl<INSP, BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> JournalStateGetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Journal = JOURNAL;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.inner.journaled_state
    }
}

impl<INSP, BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> DatabaseGetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Database = DB;

    fn db(&mut self) -> &mut Self::Database {
        self.inner.journaled_state.db_mut()
    }
}

impl<INSP, BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN>
    ErrorGetter for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Error = EVMError<DB::Error, TX::TransactionError>;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        core::mem::replace(&mut self.inner.error, Ok(())).map_err(EVMError::Database)
    }
}

impl<INSP, BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN>
    TransactionGetter for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.inner.tx
    }
}

impl<INSP, BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN>
    TransactionSetter for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
        self.inner.tx = tx;
    }
}

impl<INSP, BLOCK: Block, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> BlockGetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.inner.block
    }
}

impl<INSP, BLOCK: Block, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> BlockSetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        self.inner.block = block;
    }
}

impl<
        INSP,
        BLOCK: Block,
        TX,
        CFG,
        DB: Database,
        JOURNAL: Journal<Database = DB> + JournalExt,
        CHAIN,
    > JournalExtGetter for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type JournalExt = JOURNAL;

    fn journal_ext(&self) -> &Self::JournalExt {
        &self.inner.journaled_state
    }
}