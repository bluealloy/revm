use crate::{
    handler::mainnet::ExecutionImpl,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{db::Database, EVMError, LatestSpec, Spec},
    CallFrame, Context, CreateFrame, Frame, FrameOrResult, FrameResult,
};
use std::boxed::Box;

use dyn_clone::DynClone;
use revm_interpreter::{CallOutcome, CreateOutcome, InterpreterResult};

/// Handles last frame return handle.
pub trait LastFrameReturnTrait<EXT, DB: Database>: DynClone {
    fn last_frame_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame_result: &mut FrameResult,
    ) -> Result<(), EVMError<DB::Error>>;
}

/// Handle sub call.
pub trait FrameCallTrait<EXT, DB: Database>: DynClone {
    fn call(
        &mut self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>>;
}

/// Handle call return
pub trait FrameCallReturnTrait<EXT, DB: Database>: DynClone {
    fn call_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CallOutcome, EVMError<DB::Error>>;
}

/// Insert call outcome to the parent
pub trait InsertCallOutcomeTrait<EXT, DB: Database>: DynClone {
    fn insert_call_outcome(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> Result<(), EVMError<DB::Error>>;
}

/// Handle creation of new create frame.
pub trait FrameCreateTrait<EXT, DB: Database>: DynClone {
    fn create(
        &mut self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>>;
}

/// Handle create frame return
pub trait FrameCreateReturnTrait<EXT, DB: Database>: DynClone {
    fn create_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CreateOutcome, EVMError<DB::Error>>;
}

/// Insert crate frame outcome to the parent
pub trait InsertCreateOutcomeTrait<EXT, DB: Database>: DynClone {
    fn insert_create_outcome(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> Result<(), EVMError<DB::Error>>;
}

dyn_clone::clone_trait_object!(<EXT,DB> LastFrameReturnTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> FrameCallTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> FrameCallReturnTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> InsertCallOutcomeTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> FrameCreateTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> FrameCreateReturnTrait<EXT,DB>);
dyn_clone::clone_trait_object!(<EXT,DB> InsertCreateOutcomeTrait<EXT,DB>);

/// Handles related to stack frames.
pub struct ExecutionHandler<EXT, DB: Database> {
    /// Handles last frame return, modified gas for refund and
    /// sets tx gas limit.
    pub last_frame_return: Box<dyn LastFrameReturnTrait<EXT, DB>>,
    /// Frame call
    pub call: Box<dyn FrameCallTrait<EXT, DB>>,
    /// Call return
    pub call_return: Box<dyn FrameCallReturnTrait<EXT, DB>>,
    /// Insert call outcome
    pub insert_call_outcome: Box<dyn InsertCallOutcomeTrait<EXT, DB>>,
    /// Frame crate
    pub create: Box<dyn FrameCreateTrait<EXT, DB>>,
    /// Crate return
    pub create_return: Box<dyn FrameCreateReturnTrait<EXT, DB>>,
    /// Insert create outcome.
    pub insert_create_outcome: Box<dyn InsertCreateOutcomeTrait<EXT, DB>>,
    pub phantom: std::marker::PhantomData<(EXT, DB)>,
}

impl<EXT, DB: Database> Default for ExecutionHandler<EXT, DB> {
    fn default() -> Self {
        Self::new::<LatestSpec>()
    }
}

impl<EXT, DB: Database> ExecutionHandler<EXT, DB> {
    /// Creates mainnet ExecutionHandler.
    pub fn new<SPEC: Spec>() -> Self {
        Self {
            last_frame_return: Box::<ExecutionImpl<SPEC>>::default(),
            call: Box::<ExecutionImpl<SPEC>>::default(),
            call_return: Box::<ExecutionImpl<SPEC>>::default(),
            insert_call_outcome: Box::<ExecutionImpl<SPEC>>::default(),
            create: Box::<ExecutionImpl<SPEC>>::default(),
            create_return: Box::<ExecutionImpl<SPEC>>::default(),
            insert_create_outcome: Box::<ExecutionImpl<SPEC>>::default(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<EXT, DB: Database> ExecutionHandler<EXT, DB> {
    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    #[inline]
    pub fn last_frame_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame_result: &mut FrameResult,
    ) -> Result<(), EVMError<DB::Error>> {
        self.last_frame_return
            .last_frame_return(context, frame_result)
    }

    /// Call frame call handler.
    #[inline]
    pub fn call(
        &mut self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        self.call.call(context, inputs)
    }

    /// Call registered handler for call return.
    #[inline]
    pub fn call_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CallOutcome, EVMError<DB::Error>> {
        self.call_return
            .call_return(context, frame, interpreter_result)
    }

    /// Call registered handler for inserting call outcome.
    #[inline]
    pub fn insert_call_outcome(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> Result<(), EVMError<DB::Error>> {
        self.insert_call_outcome
            .insert_call_outcome(context, frame, shared_memory, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn create(
        &mut self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        self.create.create(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn create_return(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> Result<CreateOutcome, EVMError<DB::Error>> {
        self.create_return
            .create_return(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_create_outcome(
        &mut self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> Result<(), EVMError<DB::Error>> {
        self.insert_create_outcome
            .insert_create_outcome(context, frame, outcome)
    }
}
