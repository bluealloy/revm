
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

use crate::{journal::JournalExtGetter, InspectorCtx, InspectorInstructionProvider};

pub struct InspectorEthFrame<CTX, ERROR, PRECOMPILE>
where
    CTX: Host,
{
    // TODO : For now, hardcode the InstructionProvider. But in future this should be configurable as generic parameter.
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
        + JournalExtGetter
        + Host
        + InspectorCtx<IT = EthInterpreter>,
    ERROR: From<JournalStateGetterDBError<CTX>> + From<PrecompileErrors>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;

    fn init_first(
        context: &mut Self::Context,
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
        context: &mut Self::Context,
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

        // TODO : Handle last frame_end. MAKE a separate function for `last_return_result`.

        ret
    }

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        self.eth_frame.run(context)
    }

    fn return_result(
        &mut self,
        context: &mut Self::Context,
        mut result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        context.frame_end(&mut result);
        self.eth_frame.return_result(context, result)
    }
}