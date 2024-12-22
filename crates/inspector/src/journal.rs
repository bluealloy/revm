
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