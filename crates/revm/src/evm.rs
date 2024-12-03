use context::{block::BlockEnv, tx::TxEnv, CfgEnv, Context};
use context_interface::{
    block::BlockSetter,
    journaled_state::JournaledState,
    result::{
        EVMError, ExecutionResult, HaltReason, InvalidHeader, InvalidTransaction, ResultAndState,
    },
    transaction::TransactionSetter,
    BlockGetter, CfgGetter, DatabaseGetter, ErrorGetter, JournalStateGetter,
    JournalStateGetterDBError, Transaction, TransactionGetter,
};
use database_interface::{Database, DatabaseCommit};
use handler::{EthHandler, FrameResult};
use handler_interface::{
    ExecutionHandler, Frame, FrameOrResultGen, Handler, PostExecutionHandler, PreExecutionHandler,
    ValidationHandler,
};
use interpreter::Host;
use precompile::PrecompileErrors;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

use crate::{exec::EvmCommit, EvmExec};

/// Main EVM structure
pub struct Evm<ERROR, CTX = Context, HANDLER = EthHandler<CTX, ERROR>> {
    pub context: CTX,
    pub handler: HANDLER,
    pub _error: core::marker::PhantomData<fn() -> ERROR>,
}

impl<ERROR, CTX, HANDLER> Evm<ERROR, CTX, HANDLER> {
    pub fn new(context: CTX, handler: HANDLER) -> Self {
        Self {
            context,
            handler,
            _error: core::marker::PhantomData,
        }
    }
}

impl<ERROR, CTX, VAL, PREEXEC, EXEC, POSTEXEC> EvmCommit
    for Evm<ERROR, CTX, EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>>
where
    CTX: TransactionSetter
        + BlockSetter
        + JournalStateGetter
        + CfgGetter
        + DatabaseGetter<Database: Database + DatabaseCommit>
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<
            Journal: JournaledState<
                FinalOutput = (EvmState, Vec<Log>),
                Database = <CTX as DatabaseGetter>::Database,
            >,
        > + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationHandler<Context = CTX, Error = ERROR>,
    PREEXEC: PreExecutionHandler<Context = CTX, Error = ERROR>,
    EXEC: ExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        Frame: Frame<FrameResult = FrameResult>,
    >,
    POSTEXEC: PostExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        // TODO make output generics
        Output = ResultAndState<HaltReason>,
    >,
{
    type CommitOutput = Result<ExecutionResult<HaltReason>, ERROR>;

    fn exec_commit(&mut self) -> Self::CommitOutput {
        let res = self.transact();
        res.map(|r| {
            self.context.db().commit(r.state);
            r.result
        })
    }
}

impl<ERROR, CTX, VAL, PREEXEC, EXEC, POSTEXEC> EvmExec
    for Evm<ERROR, CTX, EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>>
where
    CTX: TransactionSetter
        + BlockSetter
        + JournalStateGetter
        + CfgGetter
        + DatabaseGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<
            Journal: JournaledState<
                FinalOutput = (EvmState, Vec<Log>),
                Database = <CTX as DatabaseGetter>::Database,
            >,
        > + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationHandler<Context = CTX, Error = ERROR>,
    PREEXEC: PreExecutionHandler<Context = CTX, Error = ERROR>,
    EXEC: ExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        Frame: Frame<FrameResult = FrameResult>,
    >,
    POSTEXEC: PostExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        // TODO make output generics
        Output = ResultAndState<HaltReason>,
    >,
{
    type Transaction = <CTX as TransactionGetter>::Transaction;

    type Block = <CTX as BlockGetter>::Block;

    type Output = Result<ResultAndState<HaltReason>, ERROR>;

    fn set_block(&mut self, block: Self::Block) {
        self.context.set_block(block);
    }

    fn set_tx(&mut self, tx: Self::Transaction) {
        self.context.set_tx(tx);
    }

    fn exec(&mut self) -> Self::Output {
        self.transact()
    }
}

/// Mainnet Error.
pub type Error<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

/// Mainnet Contexts.
pub type EthContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB, ()>;

/// Mainnet EVM type.
pub type MainEvm<DB> = Evm<Error<DB>, EthContext<DB>>;

impl<ERROR, CTX, VAL, PREEXEC, EXEC, POSTEXEC>
    Evm<ERROR, CTX, EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + DatabaseGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<
            Journal: JournaledState<
                FinalOutput = (EvmState, Vec<Log>),
                Database = <CTX as DatabaseGetter>::Database,
            >,
        > + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationHandler<Context = CTX, Error = ERROR>,
    PREEXEC: PreExecutionHandler<Context = CTX, Error = ERROR>,
    EXEC: ExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        Frame: Frame<FrameResult = FrameResult>,
    >,
    POSTEXEC: PostExecutionHandler<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        Output = ResultAndState<HaltReason>,
    >,
{
    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balance to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> Result<(), ERROR> {
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> Result<ResultAndState<HaltReason>, ERROR> {
        let initial_gas_spend = self
            .handler
            .validation()
            .validate_initial_tx_gas(&self.context)
            .inspect_err(|_| {
                self.clear();
            })?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Pre verify transaction inner.
    #[inline]
    fn preverify_transaction_inner(&mut self) -> Result<u64, ERROR> {
        self.handler.validation().validate_env(&self.context)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .validate_initial_tx_gas(&self.context)?;
        self.handler
            .validation()
            .validate_tx_against_state(&mut self.context)?;
        Ok(initial_gas_spend)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> Result<ResultAndState<HaltReason>, ERROR> {
        let initial_gas_spend = self.preverify_transaction_inner().inspect_err(|_| {
            self.clear();
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(
        &mut self,
        initial_gas_spend: u64,
    ) -> Result<ResultAndState<HaltReason>, ERROR> {
        let context = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(context)?;

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(context)?;

        let gas_limit = context.tx().common_fields().gas_limit() - initial_gas_spend;

        // apply EIP-7702 auth list.
        let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(context)? as i64;

        // start execution

        //let instructions = self.handler.take_instruction_table();
        let exec = self.handler.execution();

        // create first frame action
        let first_frame = exec.init_first_frame(context, gas_limit)?;
        let frame_result = match first_frame {
            FrameOrResultGen::Frame(frame) => exec.run(context, frame)?,
            FrameOrResultGen::Result(result) => result,
        };

        let mut exec_result = exec.last_frame_result(context, frame_result)?;

        let post_exec = self.handler.post_execution();
        // calculate final refund and add EIP-7702 refund to gas.
        post_exec.refund(context, &mut exec_result, eip7702_gas_refund);
        // Reimburse the caller
        post_exec.reimburse_caller(context, &mut exec_result)?;
        // Reward beneficiary
        post_exec.reward_beneficiary(context, &mut exec_result)?;
        // Returns output of transaction.
        post_exec.output(context, exec_result)
    }
}

/*

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        handler::mainnet::{EthExecution, EthPostExecution, EthPreExecution, EthValidation},
        EvmHandler,
    };
    use bytecode::{
        opcode::{PUSH1, SSTORE},
        Bytecode,
    };
    use core::{fmt::Debug, hash::Hash};
    use database::BenchmarkDB;
    use database_interface::Database;
    use interpreter::table::InstructionTables;
    use primitives::{address, TxKind, U256};
    use specification::{
        eip7702::{Authorization, RecoveredAuthorization, Signature},
        hardfork::{Spec, SpecId},
        spec_to_generic,
    };
    use transaction::TransactionType;
    use context_interface::{
        default::{self, block::BlockEnv, Env, TxEnv},
        result::{EVMErrorWiring, HaltReason},
        EthereumWiring, EvmWiring as InnerEvmWiring,
    };

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct CEthereumWiring<'a, DB: Database, EXT> {
        phantom: core::marker::PhantomData<&'a (DB, EXT)>,
    }

    impl<'a, DB: Database, EXT: Debug> InnerEvmWiring for CEthereumWiring<'a, DB, EXT> {
        type Database = DB;
        type ExternalContext = EXT;
        type ChainContext = ();
        type Block = default::block::BlockEnv;
        type Transaction = &'a default::TxEnv;
        type Hardfork = SpecId;
        type HaltReason = HaltReason;
    }

    impl<'a, DB: Database, EXT: Debug> EvmWiring for CEthereumWiring<'a, DB, EXT> {
        fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
        where
            DB: Database,
            'a: 'evm,
        {
            spec_to_generic!(
                hardfork,
                EvmHandler {
                    spec_id: hardfork,
                    //instruction_table: InstructionTables::new_plain::<SPEC>(),
                    registers: Vec::new(),
                    pre_execution:
                        EthPreExecution::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                            SPEC::SPEC_ID
                        ),
                    validation: EthValidation::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                    post_execution: EthPostExecution::<
                        Context<Self>,
                        EVMErrorWiring<Self>,
                        HaltReason,
                    >::new_boxed(SPEC::SPEC_ID),
                    execution: EthExecution::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                }
            )
        }
    }

    //pub type DefaultEthereumWiring = EthereumWiring<EmptyDB, ()>;

    #[test]
    fn sanity_tx_ref() {
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let mut tx = TxEnv::default();
        tx.tx_type = TransactionType::Eip7702;
        tx.gas_limit = 100_000;
        tx.authorization_list = vec![RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                address: delegate,
                nonce: 0,
            }
            .into_signed(Signature::test_signature()),
            Some(auth),
        )]
        .into();
        tx.caller = caller;
        tx.transact_to = TxKind::Call(auth);

        let mut tx2 = TxEnv::default();
        tx2.tx_type = TransactionType::Legacy;
        // nonce was bumped from 0 to 1
        tx2.nonce = 1;

        let mut evm = EvmBuilder::new_with(
            BenchmarkDB::default(),
            (),
            Env::boxed(CfgEnv::default(), BlockEnv::default(), &tx),
            CEthereumcontext_interface::handler(SpecId::LATEST),
        )
        .build();

        let _ = evm.transact().unwrap();

        let mut evm = evm
            .modify()
            .modify_tx_env(|t| {
                *t = &tx2;
            })
            .build();

        let _ = evm.transact().unwrap();
    }

    #[test]
    fn sanity_eip7702_tx() {
        let delegate = address!("0000000000000000000000000000000000000000");
        let caller = address!("0000000000000000000000000000000000000001");
        let auth = address!("0000000000000000000000000000000000000100");

        let bytecode = Bytecode::new_legacy([PUSH1, 0x01, PUSH1, 0x01, SSTORE].into());

        let mut evm = Evm::<EthereumWiring<BenchmarkDB, ()>>::builder()
            .with_spec_id(SpecId::PRAGUE)
            .with_db(BenchmarkDB::new_bytecode(bytecode))
            .with_default_ext_context()
            .modify_tx_env(|tx| {
                tx.tx_type = TransactionType::Eip7702;
                tx.gas_limit = 100_000;
                tx.authorization_list = vec![RecoveredAuthorization::new_unchecked(
                    Authorization {
                        chain_id: U256::from(1),
                        address: delegate,
                        nonce: 0,
                    }
                    .into_signed(Signature::test_signature()),
                    Some(auth),
                )]
                .into();
                tx.caller = caller;
                tx.transact_to = TxKind::Call(auth);
            })
            .build();

        let ok = evm.transact().unwrap();

        let auth_acc = ok.state.get(&auth).unwrap();
        assert_eq!(auth_acc.info.code, Some(Bytecode::new_eip7702(delegate)));
        assert_eq!(auth_acc.info.nonce, 1);
        assert_eq!(
            auth_acc.storage.get(&U256::from(1)).unwrap().present_value,
            U256::from(1)
        );
    }
}

*/
