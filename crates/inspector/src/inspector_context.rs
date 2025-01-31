use crate::{journal::JournalExtGetter, GetInspector, Inspector, InspectorCtx};
use auto_impl::auto_impl;
use revm::{
    context_interface::{
        block::BlockSetter, transaction::TransactionSetter, BlockGetter, CfgGetter, DatabaseGetter,
        ErrorGetter, JournalGetter, PerformantContextAccess, TransactionGetter,
    },
    database_interface::Database,
    handler::{handler::EthContext, FrameResult},
    interpreter::{interpreter::EthInterpreter, FrameInput, Host, Interpreter},
    primitives::{Address, Log, U256},
};
use std::vec::Vec;

/// EVM context contains data that EVM needs for execution.
#[derive(Clone, Debug)]
pub struct InspectorContext<INSP, CTX> {
    pub inspector: INSP,
    pub inner: CTX,
    pub frame_input_stack: Vec<FrameInput>,
}

#[auto_impl(&mut, Box)]
pub trait InspectorInnerCtx {
    type Context;
    fn inner_ctx(&mut self) -> &mut Self::Context;
}

impl<CTX, INSP> InspectorInnerCtx for InspectorContext<INSP, &mut CTX> {
    type Context = CTX;

    fn inner_ctx(&mut self) -> &mut Self::Context {
        &mut self.inner
    }
}

impl<INSP: Inspector<CTX, EthInterpreter>, CTX: EthContext + DatabaseGetter> EthContext
    for InspectorContext<INSP, CTX>
{
}

impl<INSP, CTX> InspectorContext<INSP, CTX>
where
    CTX: BlockGetter
        + TransactionGetter
        + CfgGetter
        + DatabaseGetter
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

impl<INSP, CTX> InspectorCtx for InspectorContext<INSP, CTX>
where
    INSP: GetInspector<CTX, EthInterpreter>,
    CTX: DatabaseGetter,
{
    type IT = EthInterpreter;

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
        let Some(frame_input) = self.frame_input_stack.pop() else {
            // case where call returns immediately will not push to call stack.
            return;
        };
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

impl<INSP, CTX> CfgGetter for InspectorContext<INSP, CTX>
where
    CTX: CfgGetter + DatabaseGetter,
{
    type Cfg = <CTX as CfgGetter>::Cfg;

    fn cfg(&self) -> &Self::Cfg {
        self.inner.cfg()
    }
}

impl<INSP, CTX> JournalGetter for InspectorContext<INSP, CTX>
where
    CTX: JournalGetter + DatabaseGetter,
{
    type Journal = <CTX as JournalGetter>::Journal;

    fn journal(&mut self) -> &mut Self::Journal {
        self.inner.journal()
    }

    fn journal_ref(&self) -> &Self::Journal {
        self.inner.journal_ref()
    }
}

impl<INSP: GetInspector<CTX, EthInterpreter>, CTX> Host for InspectorContext<INSP, CTX>
where
    CTX: Host + DatabaseGetter,
{
    fn set_error(&mut self, error: <CTX::Database as Database>::Error) {
        self.inner.set_error(error);
    }
}

impl<INSP, CTX> DatabaseGetter for InspectorContext<INSP, CTX>
where
    CTX: DatabaseGetter,
{
    type Database = <CTX as DatabaseGetter>::Database;

    fn db(&mut self) -> &mut Self::Database {
        self.inner.db()
    }

    fn db_ref(&self) -> &Self::Database {
        self.inner.db_ref()
    }
}

impl<INSP, CTX> ErrorGetter for InspectorContext<INSP, CTX>
where
    CTX: ErrorGetter + JournalGetter,
{
    type Error = <CTX as ErrorGetter>::Error;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        self.inner.take_error()
    }
}

impl<INSP, CTX> TransactionGetter for InspectorContext<INSP, CTX>
where
    CTX: TransactionGetter + DatabaseGetter,
{
    type Transaction = <CTX as TransactionGetter>::Transaction;

    fn tx(&self) -> &Self::Transaction {
        self.inner.tx()
    }
}

impl<INSP, CTX> TransactionSetter for InspectorContext<INSP, CTX>
where
    CTX: TransactionSetter + DatabaseGetter,
{
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
        self.inner.set_tx(tx);
    }
}

impl<INSP, CTX> BlockGetter for InspectorContext<INSP, CTX>
where
    CTX: BlockGetter + DatabaseGetter,
{
    type Block = <CTX as BlockGetter>::Block;

    fn block(&self) -> &Self::Block {
        self.inner.block()
    }
}

impl<INSP, CTX> BlockSetter for InspectorContext<INSP, CTX>
where
    CTX: BlockSetter + DatabaseGetter,
{
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        self.inner.set_block(block);
    }
}

impl<INSP, CTX> JournalExtGetter for InspectorContext<INSP, CTX>
where
    CTX: JournalExtGetter + DatabaseGetter,
{
    type JournalExt = <CTX as JournalExtGetter>::JournalExt;

    fn journal_ext(&self) -> &Self::JournalExt {
        self.inner.journal_ext()
    }
}

impl<INSP, CTX> PerformantContextAccess for InspectorContext<INSP, CTX>
where
    CTX: PerformantContextAccess + DatabaseGetter,
{
    type Error = <CTX as PerformantContextAccess>::Error;

    fn load_access_list(&mut self) -> Result<(), Self::Error> {
        self.inner.load_access_list()
    }
}
