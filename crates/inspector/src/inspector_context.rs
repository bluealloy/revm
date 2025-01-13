use revm::{
    context_interface::{
        block::BlockSetter, journaled_state::AccountLoad, transaction::TransactionSetter,
        BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, JournalGetter,
        PerformantContextAccess, TransactionGetter,
    },
    database_interface::Database,
    handler::FrameResult,
    interpreter::{
        interpreter::EthInterpreter, FrameInput, Host, Interpreter, SStoreResult,
        SelfDestructResult, StateLoad,
    },
    primitives::{Address, Bytes, Log, B256, U256},
};
use std::vec::Vec;

use crate::{journal::JournalExtGetter, GetInspector, Inspector, InspectorCtx};

/// EVM context contains data that EVM needs for execution.
#[derive(Clone, Debug)]
pub struct InspectorContext<INSP, DB, CTX>
where
    CTX: DatabaseGetter<Database = DB>,
{
    pub inspector: INSP,
    pub inner: CTX,
    pub frame_input_stack: Vec<FrameInput>,
}

impl<INSP, DB, CTX> InspectorContext<INSP, DB, CTX>
where
    CTX: BlockGetter
        + TransactionGetter
        + CfgGetter
        + DatabaseGetter<Database = DB>
        + JournalGetter
        + ErrorGetter
        + Host,
{
    pub fn new(inner: CTX, inspector: INSP) -> Self {
        Self {
            inner,
            inspector,
            frame_input_stack: Vec::new(),
        }
    }
}

impl<INSP: GetInspector<CTX, EthInterpreter>, DB, CTX> Host for InspectorContext<INSP, DB, CTX>
where
    CTX: Host + DatabaseGetter<Database = DB>,
{
    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        self.inner.block_hash(requested_number)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.inner.load_account_delegated(address)
    }

    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.inner.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.inner.code(address)
    }

    fn code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.inner.code_hash(address)
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

impl<INSP, DB, CTX> InspectorCtx for InspectorContext<INSP, DB, CTX>
where
    INSP: GetInspector<CTX, EthInterpreter>,
    CTX: DatabaseGetter<Database = DB>,
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

impl<INSP, DB, CTX> CfgGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: CfgGetter + DatabaseGetter<Database = DB>,
{
    type Cfg = <CTX as CfgGetter>::Cfg;

    fn cfg(&self) -> &Self::Cfg {
        self.inner.cfg()
    }
}

impl<INSP, DB, CTX> JournalGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: JournalGetter + DatabaseGetter<Database = DB>,
    DB: Database,
{
    type Journal = <CTX as JournalGetter>::Journal;

    fn journal(&mut self) -> &mut Self::Journal {
        self.inner.journal()
    }

    fn journal_ref(&self) -> &Self::Journal {
        self.inner.journal_ref()
    }
}

impl<INSP, DB, CTX> DatabaseGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: DatabaseGetter<Database = DB>,
    DB: Database,
{
    type Database = <CTX as DatabaseGetter>::Database;

    fn db(&mut self) -> &mut Self::Database {
        self.inner.db()
    }

    fn db_ref(&self) -> &Self::Database {
        self.inner.db_ref()
    }
}

impl<INSP, DB, CTX> ErrorGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: ErrorGetter + JournalGetter<Database = DB>,
{
    type Error = <CTX as ErrorGetter>::Error;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        self.inner.take_error()
    }
}

impl<INSP, DB, CTX> TransactionGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: TransactionGetter + DatabaseGetter<Database = DB>,
{
    type Transaction = <CTX as TransactionGetter>::Transaction;

    fn tx(&self) -> &Self::Transaction {
        self.inner.tx()
    }
}

impl<INSP, DB, CTX> TransactionSetter for InspectorContext<INSP, DB, CTX>
where
    CTX: TransactionSetter + DatabaseGetter<Database = DB>,
{
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
        self.inner.set_tx(tx);
    }
}

impl<INSP, DB, CTX> BlockGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: BlockGetter + DatabaseGetter<Database = DB>,
{
    type Block = <CTX as BlockGetter>::Block;

    fn block(&self) -> &Self::Block {
        self.inner.block()
    }
}

impl<INSP, DB, CTX> BlockSetter for InspectorContext<INSP, DB, CTX>
where
    CTX: BlockSetter + DatabaseGetter<Database = DB>,
{
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        self.inner.set_block(block);
    }
}

impl<INSP, DB, CTX> JournalExtGetter for InspectorContext<INSP, DB, CTX>
where
    CTX: JournalExtGetter + DatabaseGetter<Database = DB>,
{
    type JournalExt = <CTX as JournalExtGetter>::JournalExt;

    fn journal_ext(&self) -> &Self::JournalExt {
        self.inner.journal_ext()
    }
}

impl<INSP, DB: Database, CTX> PerformantContextAccess for InspectorContext<INSP, DB, CTX>
where
    CTX: PerformantContextAccess<Error = DB::Error> + DatabaseGetter<Database = DB>,
{
    type Error = <CTX as PerformantContextAccess>::Error;

    fn load_access_list(&mut self) -> Result<(), Self::Error> {
        self.inner.load_access_list()
    }
}
