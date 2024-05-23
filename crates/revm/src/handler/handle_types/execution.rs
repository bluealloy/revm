use crate::{
    frame::EOFCreateFrame,
    handler::mainnet,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{db::Database, ChainSpec, EVMError, Spec},
    CallFrame, Context, CreateFrame, Frame, FrameOrResult, FrameResult,
};
use std::{boxed::Box, sync::Arc};

use revm_interpreter::{
    CallOutcome, CreateOutcome, EOFCreateInput, EOFCreateOutcome, InterpreterResult,
};

/// Handles first frame return handle.
pub type LastFrameReturnHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            &mut FrameResult,
        ) -> Result<(), EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle sub call.
pub type FrameCallHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<CallInputs>,
        ) -> Result<FrameOrResult, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle call return
pub type FrameCallReturnHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<CallFrame>,
            InterpreterResult,
        ) -> Result<CallOutcome, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCallOutcomeHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            &mut Frame,
            &mut SharedMemory,
            CallOutcome,
        ) -> Result<(), EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle sub create.
pub type FrameCreateHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<CreateInputs>,
        ) -> Result<FrameOrResult, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle create return
pub type FrameCreateReturnHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<CreateFrame>,
            InterpreterResult,
        ) -> Result<CreateOutcome, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCreateOutcomeHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            &mut Frame,
            CreateOutcome,
        ) -> Result<(), EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle EOF sub create.
pub type FrameEOFCreateHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<EOFCreateInput>,
        ) -> Result<FrameOrResult, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handle EOF create return
pub type FrameEOFCreateReturnHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Box<EOFCreateFrame>,
            InterpreterResult,
        ) -> Result<EOFCreateOutcome, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Insert EOF crate outcome to the parent
pub type InsertEOFCreateOutcomeHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            &mut Frame,
            EOFCreateOutcome,
        ) -> Result<(), EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Handles related to stack frames.
pub struct ExecutionHandler<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Handles last frame return, modified gas for refund and
    /// sets tx gas limit.
    pub last_frame_return: LastFrameReturnHandle<'a, ChainSpecT, EXT, DB>,
    /// Frame call
    pub call: FrameCallHandle<'a, ChainSpecT, EXT, DB>,
    /// Call return
    pub call_return: FrameCallReturnHandle<'a, ChainSpecT, EXT, DB>,
    /// Insert call outcome
    pub insert_call_outcome: InsertCallOutcomeHandle<'a, ChainSpecT, EXT, DB>,
    /// Frame crate
    pub create: FrameCreateHandle<'a, ChainSpecT, EXT, DB>,
    /// Crate return
    pub create_return: FrameCreateReturnHandle<'a, ChainSpecT, EXT, DB>,
    /// Insert create outcome.
    pub insert_create_outcome: InsertCreateOutcomeHandle<'a, ChainSpecT, EXT, DB>,
    /// Frame EOFCreate
    pub eofcreate: FrameEOFCreateHandle<'a, ChainSpecT, EXT, DB>,
    /// EOFCreate return
    pub eofcreate_return: FrameEOFCreateReturnHandle<'a, ChainSpecT, EXT, DB>,
    /// Insert EOFCreate outcome.
    pub insert_eofcreate_outcome: InsertEOFCreateOutcomeHandle<'a, ChainSpecT, EXT, DB>,
}

impl<'a, ChainSpecT: ChainSpec, EXT: 'a, DB: Database + 'a>
    ExecutionHandler<'a, ChainSpecT, EXT, DB>
{
    /// Creates mainnet ExecutionHandler.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            last_frame_return: Arc::new(mainnet::last_frame_return::<ChainSpecT, SPEC, EXT, DB>),
            call: Arc::new(mainnet::call::<ChainSpecT, SPEC, EXT, DB>),
            call_return: Arc::new(mainnet::call_return::<ChainSpecT, EXT, DB>),
            insert_call_outcome: Arc::new(mainnet::insert_call_outcome),
            create: Arc::new(mainnet::create::<ChainSpecT, SPEC, EXT, DB>),
            create_return: Arc::new(mainnet::create_return::<ChainSpecT, SPEC, EXT, DB>),
            insert_create_outcome: Arc::new(mainnet::insert_create_outcome),
            eofcreate: Arc::new(mainnet::eofcreate::<ChainSpecT, SPEC, EXT, DB>),
            eofcreate_return: Arc::new(mainnet::eofcreate_return::<ChainSpecT, SPEC, EXT, DB>),
            insert_eofcreate_outcome: Arc::new(mainnet::insert_eofcreate_outcome),
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> ExecutionHandler<'a, ChainSpecT, EXT, DB> {
    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    #[inline]
    pub fn last_frame_return(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame_result: &mut FrameResult,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.last_frame_return)(context, frame_result)
    }

    /// Call frame call handler.
    #[inline]
    pub fn call(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        inputs: Box<CallInputs>,
    ) -> Result<FrameOrResult, EVMError<ChainSpecT, DB::Error>> {
        (self.call)(context, inputs.clone())
    }

    /// Call registered handler for call return.
    #[inline]
    pub fn call_return(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CallOutcome, EVMError<ChainSpecT, DB::Error>> {
        (self.call_return)(context, frame, interpreter_result)
    }

    /// Call registered handler for inserting call outcome.
    #[inline]
    pub fn insert_call_outcome(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.insert_call_outcome)(context, frame, shared_memory, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn create(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> Result<FrameOrResult, EVMError<ChainSpecT, DB::Error>> {
        (self.create)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn create_return(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CreateOutcome, EVMError<ChainSpecT, DB::Error>> {
        (self.create_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_create_outcome(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.insert_create_outcome)(context, frame, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn eofcreate(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        inputs: Box<EOFCreateInput>,
    ) -> Result<FrameOrResult, EVMError<ChainSpecT, DB::Error>> {
        (self.eofcreate)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn eofcreate_return(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: Box<EOFCreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<EOFCreateOutcome, EVMError<ChainSpecT, DB::Error>> {
        (self.eofcreate_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_eofcreate_outcome(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        frame: &mut Frame,
        outcome: EOFCreateOutcome,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.insert_eofcreate_outcome)(context, frame, outcome)
    }
}
