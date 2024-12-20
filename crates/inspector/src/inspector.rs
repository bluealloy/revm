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
        Block, BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalDBError,
        JournalGetter, Transaction, TransactionGetter,
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

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<CTX, INTR: InterpreterTypes> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [revm::interpreter::InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    #[inline]
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [revm::interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: &Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [revm::interpreter::InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a call to a contract has concluded.
    ///
    /// The returned [CallOutcome] is used as the result of the call.
    ///
    /// This allows the inspector to modify the given `result` before returning it.
    #[inline]
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    #[inline]
    fn create_end(
        &mut self,
        context: &mut CTX,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when EOF creating is called.
    ///
    /// This can happen from create TX or from EOFCREATE opcode.
    fn eofcreate(
        &mut self,
        context: &mut CTX,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut CTX,
        inputs: &EOFCreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}

/// Provides access to an `Inspector` instance.
pub trait GetInspector<CTX, INTR: InterpreterTypes> {
    /// Returns the associated `Inspector`.
    fn get_inspector(&mut self) -> &mut impl Inspector<CTX, INTR>;
}

pub trait InspectorCtx {
    type IT: InterpreterTypes;

    fn step(&mut self, interp: &mut Interpreter<Self::IT>);
    fn step_end(&mut self, interp: &mut Interpreter<Self::IT>);
    fn initialize_interp(&mut self, interp: &mut Interpreter<Self::IT>);
    fn frame_start(&mut self, frame_input: &mut FrameInput) -> Option<FrameResult>;
    fn frame_end(&mut self, frame_output: &mut FrameResult);
    fn inspector_selfdestruct(&mut self, contract: Address, target: Address, value: U256);
    fn inspector_log(&mut self, interp: &mut Interpreter<Self::IT>, log: &Log);
}

impl<CTX, INTR: InterpreterTypes, INSP: Inspector<CTX, INTR>> GetInspector<CTX, INTR> for INSP {
    #[inline]
    fn get_inspector(&mut self) -> &mut impl Inspector<CTX, INTR> {
        self
    }
}

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
        INSP: GetInspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
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
        // TODO remove duplicated function name.
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
    INSP: GetInspector<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>, EthInterpreter>,
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

impl<INSP, BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> JournalGetter
    for InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Journal = JOURNAL;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.inner.journaled_state
    }

    fn journal_ref(&self) -> &Self::Journal {
        &self.inner.journaled_state
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

impl<BLOCK: Block, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB> + JournalExt, CHAIN>
    JournalExtGetter for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type JournalExt = JOURNAL;

    fn journal_ext(&self) -> &Self::JournalExt {
        &self.journaled_state
    }
}

#[derive(Clone)]
pub struct InspectorInstruction<IT: InterpreterTypes, HOST> {
    pub instruction: fn(&mut Interpreter<IT>, &mut HOST),
}

impl<IT: InterpreterTypes, HOST> CustomInstruction for InspectorInstruction<IT, HOST>
where
    HOST: InspectorCtx<IT = IT>,
{
    type Wire = IT;
    type Host = HOST;

    fn exec(&self, interpreter: &mut Interpreter<Self::Wire>, host: &mut Self::Host) {
        // SAFETY: as the PC was already incremented we need to subtract 1 to preserve the
        // old Inspector behavior.
        interpreter.bytecode.relative_jump(-1);

        // Call step.
        host.step(interpreter);
        if interpreter.control.instruction_result() != InstructionResult::Continue {
            return;
        }

        // Reset PC to previous value.
        interpreter.bytecode.relative_jump(1);

        // Execute instruction.
        (self.instruction)(interpreter, host);

        // Call step_end.
        host.step_end(interpreter);
    }

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self {
        Self { instruction }
    }
}

pub struct InspectorInstructionProvider<WIRE: InterpreterTypes, HOST> {
    instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
}

impl<WIRE, HOST> Clone for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

pub trait JournalExt {
    fn logs(&self) -> &[Log];

    fn last_journal(&self) -> &[JournalEntry];
}

impl<DB: Database> JournalExt for JournaledState<DB> {
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    fn last_journal(&self) -> &[JournalEntry] {
        self.journal.last().expect("Journal is never empty")
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait JournalExtGetter {
    type JournalExt: JournalExt;

    fn journal_ext(&self) -> &Self::JournalExt;
}

impl<WIRE, HOST> InstructionProvider for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterTypes,
    HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
{
    type WIRE = WIRE;
    type Host = HOST;

    fn new(_context: &mut Self::Host) -> Self {
        let main_table = table::make_instruction_table::<WIRE, HOST>();
        let mut table: [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, element) in table.iter_mut().enumerate() {
            let function = InspectorInstruction {
                instruction: main_table[i],
            };
            *element = MaybeUninit::new(function);
        }

        let mut table = unsafe {
            core::mem::transmute::<
                [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256],
                [InspectorInstruction<WIRE, HOST>; 256],
            >(table)
        };

        // inspector log wrapper

        fn inspector_log<CTX: Host + JournalExtGetter + InspectorCtx>(
            interpreter: &mut Interpreter<<CTX as InspectorCtx>::IT>,
            context: &mut CTX,
            prev: Instruction<<CTX as InspectorCtx>::IT, CTX>,
        ) {
            prev(interpreter, context);

            if interpreter.control.instruction_result() == InstructionResult::Continue {
                let last_log = context.journal_ext().logs().last().unwrap().clone();
                context.inspector_log(interpreter, &last_log);
            }
        }

        /* LOG and Selfdestruct instructions */
        table[OpCode::LOG0.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<0, HOST>);
            },
        };
        table[OpCode::LOG1.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<1, HOST>);
            },
        };
        table[OpCode::LOG2.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<2, HOST>);
            },
        };
        table[OpCode::LOG3.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<3, HOST>);
            },
        };
        table[OpCode::LOG4.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                inspector_log(interp, context, log::<4, HOST>);
            },
        };

        table[OpCode::SELFDESTRUCT.as_usize()] = InspectorInstruction {
            instruction: |interp, context| {
                selfdestruct::<Self::WIRE, HOST>(interp, context);
                if interp.control.instruction_result() == InstructionResult::SelfDestruct {
                    match context.journal_ext().last_journal().last() {
                        Some(JournalEntry::AccountDestroyed {
                            address,
                            target,
                            had_balance,
                            ..
                        }) => {
                            context.inspector_selfdestruct(*address, *target, *had_balance);
                        }
                        Some(JournalEntry::BalanceTransfer {
                            from, to, balance, ..
                        }) => {
                            context.inspector_selfdestruct(*from, *to, *balance);
                        }
                        _ => {}
                    }
                }
            },
        };

        Self {
            instruction_table: Rc::new(table),
        }
    }

    fn table(&mut self) -> &[impl CustomInstruction<Wire = Self::WIRE, Host = Self::Host>; 256] {
        self.instruction_table.as_ref()
    }
}

pub struct InspectorEthFrame<CTX, ERROR, PRECOMPILE>
where
    CTX: Host,
{
    /// TODO for now hardcode the InstructionProvider. But in future this should be configurable
    /// as generic parameter.
    pub eth_frame: EthFrame<
        CTX,
        ERROR,
        EthInterpreter<()>,
        PRECOMPILE,
        InspectorInstructionProvider<EthInterpreter<()>, CTX>,
    >,
}

impl<CTX, ERROR, PRECOMPILE> Frame for InspectorEthFrame<CTX, ERROR, PRECOMPILE>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalGetter
        + CfgGetter
        + JournalExtGetter
        + Host
        + InspectorCtx<IT = EthInterpreter>,
    ERROR: From<JournalDBError<CTX>> + From<PrecompileErrors>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;

    fn init_first(
        context: &mut CTX,
        mut frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(FrameOrResultGen::Result(output));
        }
        let mut ret = EthFrame::init_first(context, frame_input)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }));

        match &mut ret {
            Ok(FrameOrResultGen::Result(res)) => {
                context.frame_end(res);
            }
            Ok(FrameOrResultGen::Frame(frame)) => {
                context.initialize_interp(&mut frame.eth_frame.interpreter);
            }
            _ => (),
        }

        ret
    }

    fn init(
        &self,
        context: &mut CTX,
        mut frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        if let Some(output) = context.frame_start(&mut frame_input) {
            return Ok(FrameOrResultGen::Result(output));
        }
        let mut ret = self
            .eth_frame
            .init(context, frame_input)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }));

        if let Ok(FrameOrResultGen::Frame(frame)) = &mut ret {
            context.initialize_interp(&mut frame.eth_frame.interpreter);
        }

        // TODO handle last frame_end. MAKE a separate function for `last_return_result`.

        ret
    }

    fn run(
        &mut self,
        context: &mut CTX,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        self.eth_frame.run(context)
    }

    fn return_result(
        &mut self,
        context: &mut CTX,
        mut result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(&mut result);
        self.eth_frame.return_result(context, result)
    }
}

pub type InspCtxType<
    INSP,
    DB,
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    JOURNAL = JournaledState<DB>,
> = InspectorContext<INSP, BLOCK, TX, CFG, DB, JOURNAL, ()>;

pub type InspectorMainEvm<
    DB,
    INSP,
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    JOURNAL = JournaledState<DB>,
> = Evm<
    Error<DB>,
    InspCtxType<INSP, DB, BLOCK, TX, CFG, JOURNAL>,
    EthHandler<
        InspCtxType<INSP, DB, BLOCK, TX, CFG, JOURNAL>,
        Error<DB>,
        EthValidation<InspCtxType<INSP, DB, BLOCK, TX, CFG, JOURNAL>, Error<DB>>,
        EthPreExecution<InspCtxType<INSP, DB, BLOCK, TX, CFG, JOURNAL>, Error<DB>>,
        InspectorEthExecution<InspCtxType<INSP, DB, BLOCK, TX, CFG, JOURNAL>, Error<DB>>,
    >,
>;

/// Function to create Inspector Handler.
pub fn inspector_handler<CTX: Host, ERROR, PRECOMPILE>() -> InspectorHandler<CTX, ERROR, PRECOMPILE>
{
    EthHandler::new(
        EthValidation::new(),
        EthPreExecution::new(),
        EthExecution::<_, _, InspectorEthFrame<_, _, PRECOMPILE>>::new(),
        EthPostExecution::new(),
    )
}

/// Composed type for Inspector Execution handler.
pub type InspectorEthExecution<CTX, ERROR, PRECOMPILE = EthPrecompileProvider<CTX, ERROR>> =
    EthExecution<CTX, ERROR, InspectorEthFrame<CTX, ERROR, PRECOMPILE>>;

/// Composed type for Inspector Handler.
pub type InspectorHandler<CTX, ERROR, PRECOMPILE> = EthHandler<
    CTX,
    ERROR,
    EthValidation<CTX, ERROR>,
    EthPreExecution<CTX, ERROR>,
    InspectorEthExecution<CTX, ERROR, PRECOMPILE>,
>;
