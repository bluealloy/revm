use crate::{
    frame::EOFCreateFrame,
    handler::mainnet,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{db::Database, EVMResultGeneric, Spec},
    CallFrame, Context, CreateFrame, EvmWiring, Frame, FrameOrResult, FrameResult,
};
use revm_interpreter::{
    opcode::InstructionTables, CallOutcome, CreateOutcome, EOFCreateInputs, InterpreterAction,
    InterpreterResult,
};
use std::{boxed::Box, sync::Arc};

/// Handles first frame return handle.
pub type LastFrameReturnHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            &mut FrameResult,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Executes a single frame. Errors can be returned in the EVM context.
pub type ExecuteFrameHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Frame,
            &mut SharedMemory,
            &InstructionTables<'_, Context<EvmWiringT, EXT, DB>>,
            &mut Context<EvmWiringT, EXT, DB>,
        ) -> EVMResultGeneric<InterpreterAction, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle sub call.
pub type FrameCallHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<CallInputs>,
        ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle call return
pub type FrameCallReturnHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<CallFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CallOutcome, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCallOutcomeHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            &mut Frame,
            &mut SharedMemory,
            CallOutcome,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle sub create.
pub type FrameCreateHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<CreateInputs>,
        ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle create return
pub type FrameCreateReturnHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<CreateFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CreateOutcome, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCreateOutcomeHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            &mut Frame,
            CreateOutcome,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle EOF sub create.
pub type FrameEOFCreateHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<EOFCreateInputs>,
        ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handle EOF create return
pub type FrameEOFCreateReturnHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            Box<EOFCreateFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CreateOutcome, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Insert EOF crate outcome to the parent
pub type InsertEOFCreateOutcomeHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            &mut Frame,
            CreateOutcome,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Handles related to stack frames.
pub struct ExecutionHandler<'a, EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Handles last frame return, modified gas for refund and
    /// sets tx gas limit.
    pub last_frame_return: LastFrameReturnHandle<'a, EvmWiringT, EXT, DB>,
    /// Executes a single frame.
    pub execute_frame: ExecuteFrameHandle<'a, EvmWiringT, EXT, DB>,
    /// Frame call
    pub call: FrameCallHandle<'a, EvmWiringT, EXT, DB>,
    /// Call return
    pub call_return: FrameCallReturnHandle<'a, EvmWiringT, EXT, DB>,
    /// Insert call outcome
    pub insert_call_outcome: InsertCallOutcomeHandle<'a, EvmWiringT, EXT, DB>,
    /// Frame crate
    pub create: FrameCreateHandle<'a, EvmWiringT, EXT, DB>,
    /// Crate return
    pub create_return: FrameCreateReturnHandle<'a, EvmWiringT, EXT, DB>,
    /// Insert create outcome.
    pub insert_create_outcome: InsertCreateOutcomeHandle<'a, EvmWiringT, EXT, DB>,
    /// Frame EOFCreate
    pub eofcreate: FrameEOFCreateHandle<'a, EvmWiringT, EXT, DB>,
    /// EOFCreate return
    pub eofcreate_return: FrameEOFCreateReturnHandle<'a, EvmWiringT, EXT, DB>,
    /// Insert EOFCreate outcome.
    pub insert_eofcreate_outcome: InsertEOFCreateOutcomeHandle<'a, EvmWiringT, EXT, DB>,
}

impl<'a, EvmWiringT: EvmWiring, EXT: 'a, DB: Database + 'a>
    ExecutionHandler<'a, EvmWiringT, EXT, DB>
{
    /// Creates mainnet ExecutionHandler.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            last_frame_return: Arc::new(mainnet::last_frame_return::<EvmWiringT, SPEC, EXT, DB>),
            execute_frame: Arc::new(mainnet::execute_frame::<EvmWiringT, SPEC, EXT, DB>),
            call: Arc::new(mainnet::call::<EvmWiringT, SPEC, EXT, DB>),
            call_return: Arc::new(mainnet::call_return::<EvmWiringT, EXT, DB>),
            insert_call_outcome: Arc::new(mainnet::insert_call_outcome),
            create: Arc::new(mainnet::create::<EvmWiringT, SPEC, EXT, DB>),
            create_return: Arc::new(mainnet::create_return::<EvmWiringT, SPEC, EXT, DB>),
            insert_create_outcome: Arc::new(mainnet::insert_create_outcome),
            eofcreate: Arc::new(mainnet::eofcreate::<EvmWiringT, SPEC, EXT, DB>),
            eofcreate_return: Arc::new(mainnet::eofcreate_return::<EvmWiringT, SPEC, EXT, DB>),
            insert_eofcreate_outcome: Arc::new(mainnet::insert_eofcreate_outcome),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring, EXT, DB: Database> ExecutionHandler<'a, EvmWiringT, EXT, DB> {
    /// Executes single frame.
    #[inline]
    pub fn execute_frame(
        &self,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        instruction_tables: &InstructionTables<'_, Context<EvmWiringT, EXT, DB>>,
        context: &mut Context<EvmWiringT, EXT, DB>,
    ) -> EVMResultGeneric<InterpreterAction, EvmWiringT, DB::Error> {
        (self.execute_frame)(frame, shared_memory, instruction_tables, context)
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    #[inline]
    pub fn last_frame_return(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame_result: &mut FrameResult,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.last_frame_return)(context, frame_result)
    }

    /// Call frame call handler.
    #[inline]
    pub fn call(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        inputs: Box<CallInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, DB::Error> {
        (self.call)(context, inputs)
    }

    /// Call registered handler for call return.
    #[inline]
    pub fn call_return(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CallOutcome, EvmWiringT, DB::Error> {
        (self.call_return)(context, frame, interpreter_result)
    }

    /// Call registered handler for inserting call outcome.
    #[inline]
    pub fn insert_call_outcome(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.insert_call_outcome)(context, frame, shared_memory, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn create(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, DB::Error> {
        (self.create)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn create_return(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CreateOutcome, EvmWiringT, DB::Error> {
        (self.create_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_create_outcome(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.insert_create_outcome)(context, frame, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn eofcreate(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        inputs: Box<EOFCreateInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT, DB::Error> {
        (self.eofcreate)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn eofcreate_return(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: Box<EOFCreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CreateOutcome, EvmWiringT, DB::Error> {
        (self.eofcreate_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_eofcreate_outcome(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.insert_eofcreate_outcome)(context, frame, outcome)
    }
}
