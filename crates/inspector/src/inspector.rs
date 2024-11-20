use core::mem::MaybeUninit;
use std::rc::Rc;

use auto_impl::auto_impl;
use derive_where::derive_where;
use revm::{
    bytecode::{opcode::OpCode, Bytecode, EOF_MAGIC_BYTES, EOF_MAGIC_HASH},
    context::{
        default::{block::BlockEnv, tx::TxEnv},
        BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, JournalStateGetter,
        JournalStateGetterDBError, TransactionGetter,
    },
    database_interface::{Database, EmptyDB},
    handler::{EthPrecompileProvider, Frame, PrecompileProvider},
    interpreter::{
        as_u64_saturated,
        instructions::{arithmetic::addmod, host, instruction},
        interpreter::{EthInstructionProvider, EthInterpreter, InstructionProvider},
        interpreter_wiring::{Jumps, MemoryTrait},
        table::{self, CustomInstruction},
        CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, Host, Instruction,
        InterpreterWire, NewFrameAction, NewInterpreter, SStoreResult, SelfDestructResult,
        StateLoad,
    },
    mainnet::{EthHandler, FrameResult},
    precompile::PrecompileErrors,
    primitives::{Address, Bytes, HashSet, Log, B256, BLOCK_HASH_HISTORY, U256},
    specification::hardfork::SpecId,
    context_interface::{
        journaled_state::{AccountLoad, Eip7702CodeLoad},
        result::{EVMError, InvalidTransaction},
        Block, CfgEnv, Transaction,
    },
    Context, Error, Evm, JournaledState,
};

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector {
    type Context;
    type InterpreterWire: InterpreterWire;
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [revm::interpreter::InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(
        &mut self,
        interp: &mut NewInterpreter<Self::InterpreterWire>,
        context: &mut Self::Context,
    ) {
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
    fn step(
        &mut self,
        interp: &mut NewInterpreter<Self::InterpreterWire>,
        context: &mut Self::Context,
    ) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [revm::interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(
        &mut self,
        interp: &mut NewInterpreter<Self::InterpreterWire>,
        context: &mut Self::Context,
    ) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(
        &mut self,
        interp: &mut NewInterpreter<Self::InterpreterWire>,
        context: &mut Self::Context,
        log: &Log,
    ) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [revm::interpreter::InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(
        &mut self,
        context: &mut Self::Context,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
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
    fn call_end(
        &mut self,
        context: &mut Self::Context,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(
        &mut self,
        context: &mut Self::Context,
        inputs: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
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
        context: &mut Self::Context,
        inputs: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when EOF creating is called.
    ///
    /// This can happen from create TX or from EOFCREATE opcode.
    fn eofcreate(
        &mut self,
        context: &mut Self::Context,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut Self::Context,
        inputs: &EOFCreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        let _ = context;
        let _ = inputs;
        outcome
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}

pub struct StepPrintInspector<CTX> {
    _phantom: core::marker::PhantomData<CTX>,
}

impl<CTX> StepPrintInspector<CTX> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX> Inspector for StepPrintInspector<CTX> {
    type Context = CTX;
    type InterpreterWire = EthInterpreter;

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    #[inline]
    fn step(
        &mut self,
        interp: &mut NewInterpreter<Self::InterpreterWire>,
        context: &mut Self::Context,
    ) {
        let opcode = interp.bytecode.opcode();
        let name = OpCode::name_by_op(opcode);

        let gas_remaining = 0; //self.gas_inspector.gas_remaining();

        let memory_size = interp.memory.size();

        println!(
            "depth:{}, PC:{}, gas:{:#x}({}), OPCODE: {:?}({:?})  refund:{:#x}({}) Stack:{:?}, Data size:{}",
            0,
            interp.bytecode.pc(),
            gas_remaining,
            gas_remaining,
            name,
            opcode,
            0, //interp.gas.refunded(),
            0, //interp.gas.refunded(),
            interp.stack.data(),
            memory_size,
        );
    }
}

/// Provides access to an `Inspector` instance.
pub trait GetInspector {
    type Inspector: Inspector;
    /// Returns the associated `Inspector`.
    fn get_inspector(&mut self) -> &mut Self::Inspector;
}

pub trait InspectorCtx {
    type IW: InterpreterWire;

    fn step(&mut self, interp: &mut NewInterpreter<Self::IW>);

    fn step_end(&mut self, interp: &mut NewInterpreter<Self::IW>);
}

impl<INSP: Inspector> GetInspector for INSP {
    type Inspector = INSP;
    #[inline]
    fn get_inspector(&mut self) -> &mut Self::Inspector {
        self
    }
}

/// EVM context contains data that EVM needs for execution.
#[derive_where(Clone, Debug; INSP, BLOCK, SPEC, CHAIN, TX, DB, <DB as Database>::Error)]
pub struct InspectorContext<
    INSP,
    BLOCK = BlockEnv,
    TX = TxEnv,
    SPEC = SpecId,
    DB: Database = EmptyDB,
    CHAIN = (),
> {
    pub inner: Context<BLOCK, TX, SPEC, DB, CHAIN>,
    pub inspector: INSP,
}

impl<INSP: GetInspector, BLOCK: Block, TX: Transaction, SPEC, DB: Database, CHAIN> Host
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type BLOCK = BLOCK;
    type TX = TX;

    fn tx(&self) -> &Self::TX {
        &self.inner.tx
    }

    fn block(&self) -> &Self::BLOCK {
        &self.inner.block
    }

    fn cfg(&self) -> &CfgEnv {
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
        <Context<_, _, _, _, _> as Host>::code(&mut self.inner, address)
    }

    fn code_hash(&mut self, address: Address) -> Option<Eip7702CodeLoad<B256>> {
        <Context<_, _, _, _, _> as Host>::code_hash(&mut self.inner, address)
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

impl<INSP, BLOCK, TX, DB: Database, SPEC, CHAIN> CfgGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Cfg = CfgEnv;

    fn cfg(&self) -> &Self::Cfg {
        &self.inner.cfg
    }
}

impl<INSP, BLOCK, TX, SPEC, DB: Database, CHAIN> InspectorCtx
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
where
    INSP: GetInspector<
        Inspector: Inspector<
            Context = Context<BLOCK, TX, SPEC, DB, CHAIN>,
            InterpreterWire = EthInterpreter,
        >,
    >,
{
    type IW = EthInterpreter<()>;

    fn step(&mut self, interp: &mut NewInterpreter<Self::IW>) {
        self.inspector.get_inspector().step(interp, &mut self.inner);
    }

    fn step_end(&mut self, interp: &mut NewInterpreter<Self::IW>) {
        self.inspector
            .get_inspector()
            .step_end(interp, &mut self.inner);
    }
}

impl<INSP, BLOCK, TX, SPEC, DB: Database, CHAIN> JournalStateGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Journal = JournaledState<DB>;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.inner.journaled_state
    }
}

impl<INSP, BLOCK, TX, SPEC, DB: Database, CHAIN> DatabaseGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Database = DB;

    fn db(&mut self) -> &mut Self::Database {
        &mut self.inner.journaled_state.database
    }
}

impl<INSP, BLOCK, TX: Transaction, SPEC, DB: Database, CHAIN> ErrorGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Error = EVMError<DB::Error, TX::TransactionError>;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        core::mem::replace(&mut self.inner.error, Ok(())).map_err(EVMError::Database)
    }
}

impl<INSP, BLOCK, TX: Transaction, SPEC, DB: Database, CHAIN> TransactionGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.inner.tx
    }
}
impl<INSP, BLOCK: Block, TX, SPEC, DB: Database, CHAIN> BlockGetter
    for InspectorContext<INSP, BLOCK, TX, SPEC, DB, CHAIN>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.inner.block
    }
}

use revm::mainnet::{EthExecution, EthFrame, EthPostExecution, EthPreExecution, EthValidation};

#[derive(Clone)]
pub struct InspectorInstruction<WIRE: InterpreterWire, HOST> {
    pub instruction: fn(&mut NewInterpreter<WIRE>, &mut HOST),
}

impl<WIRE: InterpreterWire, HOST> CustomInstruction for InspectorInstruction<WIRE, HOST>
where
    HOST: InspectorCtx<IW = WIRE>,
{
    type Wire = WIRE;
    type Host = HOST;

    fn exec(&self, interpreter: &mut NewInterpreter<Self::Wire>, host: &mut Self::Host) {
        host.step(interpreter);
        (self.instruction)(interpreter, host);
        host.step_end(interpreter);
    }

    fn from_base(instruction: Instruction<Self::Wire, Self::Host>) -> Self {
        Self {
            instruction: instruction,
        }
    }
}

pub struct InspectorInstructionProvider<WIRE: InterpreterWire, HOST> {
    instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
}

impl<WIRE, HOST> Clone for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterWire,
{
    fn clone(&self) -> Self {
        Self {
            instruction_table: self.instruction_table.clone(),
        }
    }
}

/*
INSPECTOR FEATURES:
- [x] Step/StepEnd (Step/StepEnd are wrapped inside InspectorInstructionProvider)
        * currently limited to mainnet instructions.
- [ ] Initialize
        * Needs EthFrame wrapper.
- [ ] Call/CallEnd
- [ ] Create/CreateEnd
- [ ] EOFCreate/EOFCreateEnd
- [ ] SelfDestruct
- [ ] Log
*/

impl<WIRE, HOST> InstructionProvider for InspectorInstructionProvider<WIRE, HOST>
where
    WIRE: InterpreterWire,
    HOST: Host + InspectorCtx<IW = WIRE>,
{
    type WIRE = WIRE;
    type Host = HOST;

    fn new(_ctx: &mut Self::Host) -> Self {
        // TODO make this configurable. Inspection over same instruction is not useful.
        let main_table = table::make_instruction_table::<WIRE, HOST>();
        let mut table: [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, element) in table.iter_mut().enumerate() {
            let foo = InspectorInstruction {
                instruction: main_table[i],
            };
            *element = MaybeUninit::new(foo);
        }

        /* LOG and Selfdestruct instructions */

        let table =
            unsafe { core::mem::transmute::<_, [InspectorInstruction<WIRE, HOST>; 256]>(table) };
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
        + JournalStateGetter
        + CfgGetter
        + Host
        + InspectorCtx<IW = EthInterpreter>,
    ERROR: From<JournalStateGetterDBError<CTX>> + From<PrecompileErrors>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = NewFrameAction;
    type FrameResult = FrameResult;

    fn init_first(
        cxt: &mut Self::Context,
        frame_action: Self::FrameInit,
    ) -> Result<revm::handler::FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        EthFrame::init_first(cxt, frame_action)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }))
    }

    fn init(
        &self,
        cxt: &mut Self::Context,
        frame_action: Self::FrameInit,
    ) -> Result<revm::handler::FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.eth_frame
            .init(cxt, frame_action)
            .map(|frame| frame.map_frame(|eth_frame| Self { eth_frame }))
    }

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<revm::handler::FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error>
    {
        self.eth_frame.run(context)
    }

    fn return_result(
        &mut self,
        cxt: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        self.eth_frame.return_result(cxt, result)
    }
}

pub type InspCtxType<INSP, DB> = InspectorContext<INSP, BlockEnv, TxEnv, SpecId, DB, ()>;

pub type InspectorMainEvm<DB, INSP> = Evm<
    Error<DB>,
    InspCtxType<INSP, DB>,
    EthHandler<
        InspCtxType<INSP, DB>,
        Error<DB>,
        EthValidation<InspCtxType<INSP, DB>, Error<DB>>,
        EthPreExecution<InspCtxType<INSP, DB>, Error<DB>>,
        EthExecution<
            InspCtxType<INSP, DB>,
            Error<DB>,
            InspectorEthFrame<
                InspCtxType<INSP, DB>,
                Error<DB>,
                EthPrecompileProvider<InspCtxType<INSP, DB>, Error<DB>>,
            >,
        >,
    >,
>;
