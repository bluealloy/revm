use core::cell::RefCell;
use std::rc::Rc;

use crate::handler::{wires::Frame, FrameOrResultGen};
use context::{
    BlockGetter, Context, Frame as FrameData, FrameResult, JournalStateGetter, TransactionGetter,
};
use interpreter::{Host, InternalResult, NewFrameAction, SharedMemory};
use specification::hardfork::{Spec, SpecId};
use wiring::{
    result::{EVMError, EVMErrorWiring},
    EvmWiring,
};

pub struct EthFrame<EvmWiring, CTX, FORK> {
    _phantom: std::marker::PhantomData<(CTX, FORK, EvmWiring)>,
    data: FrameData,
    // This is worth making as a generic type.
    shared_memory: Rc<RefCell<SharedMemory>>,
}

impl<EvmWiring, CTX, FORK> EthFrame<EvmWiring, CTX, FORK> {
    pub fn new(data: FrameData) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            data,
            shared_memory: Rc::new(RefCell::new(SharedMemory::new())),
        }
    }
}

pub trait HostTemp: TransactionGetter + BlockGetter + JournalStateGetter {}

impl<CTX: HostTemp, FORK: Spec, EvmWiringT: EvmWiring> Frame for EthFrame<EvmWiringT, CTX, FORK> {
    type Context = Context<EvmWiringT>;

    type Error = EVMErrorWiring<EvmWiringT>;

    type FrameInit = NewFrameAction;

    type FrameResult = FrameResult;

    fn init(
        &self,
        frame_init: Self::FrameInit,
        ctx: &mut Context<EvmWiringT>,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.shared_memory.borrow_mut().new_context();
        let mut ret = match frame_init {
            NewFrameAction::Call(inputs) => ctx.evm.make_call_frame(&inputs).map(Into::into),
            NewFrameAction::Create(inputs) => ctx
                .evm
                .make_create_frame(FORK::SPEC_ID, &inputs)
                .map(Into::into),
            NewFrameAction::EOFCreate(inputs) => ctx
                .evm
                .make_eofcreate_frame(FORK::SPEC_ID, &inputs)
                .map(Into::into),
        };
        if let Ok(FrameOrResultGen::Frame(frame)) = &mut ret {
            frame.shared_memory = self.shared_memory.clone();
        }
        ret
    }

    fn run(
        &mut self,
        instructions: (),
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        todo!()

        /*
        /// Return frame result from this
                let ctx = &mut self.context;
                   FrameOrResult::Result(match returned_frame {
                       Frame::Call(frame) => {
                           // return_call
                           FrameResult::Call(exec.call_return(ctx, frame, result)?)
                       }
                       Frame::Create(frame) => {
                           // return_create
                           FrameResult::Create(exec.create_return(ctx, frame, result)?)
                       }
                       Frame::EOFCreate(frame) => {
                           // return_eofcreate
                           FrameResult::EOFCreate(exec.eofcreate_return(ctx, frame, result)?)
                       }
                   })
        */
    }

    fn return_result(
        &mut self,
        ctx: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        self.shared_memory.borrow_mut().free_context();
        ctx.evm.take_error().map_err(EVMError::Database)?;

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                // return_call
                let mut shared_memory = self.shared_memory.borrow_mut();
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_call_outcome(&mut shared_memory, outcome);
            }
            FrameResult::Create(outcome) => {
                // return_create
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_create_outcome(outcome);
            }
            FrameResult::EOFCreate(outcome) => {
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_eofcreate_outcome(outcome);
            }
        }

        Ok(())
    }
}
