// Modules.
pub mod mainnet;
mod wires;

use context::{
    default::{block::BlockEnv, tx::TxEnv},
    BlockGetter, CfgGetter, Context, DatabaseGetter, ErrorGetter, JournalStateGetter,
    JournalStateGetterDBError, TransactionGetter,
};
use database_interface::Database;
// Exports.
use mainnet::{
    EthExecution, EthFrame, EthPostExecution, EthPreExecution, EthValidation, FrameResult,
};
use precompile::PrecompileErrors;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
pub use wires::*;

// Includes.

use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter},
    Host,
};
//use register::{EvmHandler, HandleRegisters};
use std::vec::Vec;
use wiring::{
    journaled_state::JournaledState,
    result::{EVMError, HaltReason, InvalidHeader, InvalidTransaction, ResultAndState},
    Transaction,
};

pub trait Handler {
    type Validation: ValidationWire;
    type PreExecution: PreExecutionWire;
    type Execution: ExecutionWire;
    type PostExecution: PostExecutionWire;

    fn validation(&mut self) -> &mut Self::Validation;
    fn pre_execution(&mut self) -> &mut Self::PreExecution;
    fn execution(&mut self) -> &mut Self::Execution;
    fn post_execution(&mut self) -> &mut Self::PostExecution;
}

/// TODO Halt needs to be generalized.
#[derive(Default)]
pub struct EthHand<
    CTX,
    ERROR,
    VAL = EthValidation<CTX, ERROR>,
    PREEXEC = EthPreExecution<CTX, ERROR>,
    EXEC = EthExecution<CTX, ERROR>,
    POSTEXEC = EthPostExecution<CTX, ERROR, HaltReason>,
> {
    pub validation: VAL,
    pub pre_execution: PREEXEC,
    pub execution: EXEC,
    pub post_execution: POSTEXEC,
    _phantom: std::marker::PhantomData<fn() -> (CTX, ERROR)>,
}

impl<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC> EthHand<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC> {
    pub fn new(
        validation: VAL,
        pre_execution: PREEXEC,
        execution: EXEC,
        post_execution: POSTEXEC,
    ) -> Self {
        Self {
            validation,
            pre_execution,
            execution,
            post_execution,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct CustomEthHand<CTX, ERROR> {
    main_eth_hand: EthHand<CTX, ERROR>,
    execution: EthExecution<CTX, ERROR>,
}

impl<CTX, ERROR> Handler for CustomEthHand<CTX, ERROR>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<Journal: JournaledState<FinalOutput = (EvmState, Vec<Log>)>>
        + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
{
    type Validation = <EthHand<CTX, ERROR> as Handler>::Validation;
    type PreExecution = <EthHand<CTX, ERROR> as Handler>::PreExecution;
    type Execution = <EthHand<CTX, ERROR> as Handler>::Execution;
    type PostExecution = <EthHand<CTX, ERROR> as Handler>::PostExecution;

    fn validation(&mut self) -> &mut Self::Validation {
        self.main_eth_hand.validation()
    }

    fn pre_execution(&mut self) -> &mut Self::PreExecution {
        self.main_eth_hand.pre_execution()
    }

    fn execution(&mut self) -> &mut Self::Execution {
        self.main_eth_hand.execution()
    }

    fn post_execution(&mut self) -> &mut Self::PostExecution {
        self.main_eth_hand.post_execution()
    }
}

impl<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC> Handler
    for EthHand<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<Journal: JournaledState<FinalOutput = (EvmState, Vec<Log>)>>
        + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationWire,
    PREEXEC: PreExecutionWire,
    EXEC: ExecutionWire,
    POSTEXEC: PostExecutionWire,
{
    type Validation = VAL;
    type PreExecution = PREEXEC;
    type Execution = EXEC;
    type PostExecution = POSTEXEC;

    //type InstructionTable = InstructionTables<'static, CTX>;

    fn validation(&mut self) -> &mut Self::Validation {
        &mut self.validation
    }

    fn pre_execution(&mut self) -> &mut Self::PreExecution {
        &mut self.pre_execution
    }

    fn execution(&mut self) -> &mut Self::Execution {
        &mut self.execution
    }

    fn post_execution(&mut self) -> &mut Self::PostExecution {
        &mut self.post_execution
    }
}

//EvmWiring::Hardfork::default();

pub struct GEVM<ERROR, CTX = Context, HAND = EthHand<CTX, ERROR>> {
    pub context: CTX,
    pub handler: HAND,
    pub _error: std::marker::PhantomData<fn() -> ERROR>,
}

pub struct EEVM<ERROR, CTX = Context> {
    pub context: CTX,
    pub handler: EthHand<CTX, ERROR>,
}

pub type GEEVM<DB> = EEVM<EVMError<<DB as Database>::Error, InvalidTransaction>, EthContext<DB>>;

pub type EthContext<DB> = Context<BlockEnv, TxEnv, SpecId, DB, ()>;

pub type NNEW_EVMM<DB> = NEW_EVM<DB, EVMError<<DB as Database>::Error, InvalidTransaction>>;

pub type NEW_EVM<DB, ERROR> = GEVM<
    ERROR,
    EthContext<DB>,
    EthHand<
        EthContext<DB>,
        ERROR,
        EthValidation<EthContext<DB>, ERROR>,
        EthPreExecution<EthContext<DB>, ERROR>,
        EthExecution<
            EthContext<DB>,
            ERROR,
            EthFrame<
                EthContext<DB>,
                ERROR,
                EthInterpreter<()>,
                EthPrecompileProvider<EthContext<DB>, ERROR>,
                EthInstructionProvider<EthInterpreter<()>, EthContext<DB>>,
            >,
        >,
    >,
>;

impl<ERROR, CTX> EEVM<ERROR, CTX>
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
{
    // TODO
    // transact_commit (needs DatabaseCommit requirement)
}

impl<ERROR, CTX, VAL, PREEXEC, EXEC, POSTEXEC>
    GEVM<ERROR, CTX, EthHand<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>>
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
    VAL: ValidationWire<Context = CTX, Error = ERROR>,
    PREEXEC: PreExecutionWire<Context = CTX, Error = ERROR>,
    EXEC: ExecutionWire<
        Context = CTX,
        Error = ERROR,
        ExecResult = FrameResult,
        Frame: Frame<FrameResult = FrameResult>,
    >,
    POSTEXEC: PostExecutionWire<
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
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.tx().common_fields().gas_limit() - initial_gas_spend;

        // apply EIP-7702 auth list.
        let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(ctx)? as i64;

        // start execution

        //let instructions = self.handler.take_instruction_table();
        let exec = self.handler.execution();

        // create first frame action
        let first_frame = exec.init_first_frame(ctx, gas_limit)?;
        let frame_result = match first_frame {
            FrameOrResultGen::Frame(frame) => exec.run(ctx, frame)?,
            FrameOrResultGen::Result(result) => result,
        };

        let mut exec_result = exec.last_frame_result(ctx, frame_result)?;

        //self.handler.set_instruction_table(instructions);

        let post_exec = self.handler.post_execution();
        // calculate final refund and add EIP-7702 refund to gas.
        post_exec.refund(ctx, &mut exec_result, eip7702_gas_refund);
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, &mut exec_result)?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, &mut exec_result)?;
        // Returns output of transaction.
        post_exec.output(ctx, exec_result)
    }
}

impl<ERROR, CTX> EEVM<ERROR, CTX>
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
{
    // TODO
    // transact_commit (needs DatabaseCommit requirement)

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
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.tx().common_fields().gas_limit() - initial_gas_spend;

        // apply EIP-7702 auth list.
        let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(ctx)? as i64;

        // start execution

        //let instructions = self.handler.take_instruction_table();
        let exec = self.handler.execution();

        // create first frame action
        let first_frame = exec.init_first_frame(ctx, gas_limit)?;
        let frame_result = match first_frame {
            FrameOrResultGen::Frame(frame) => exec.run(ctx, frame)?,
            FrameOrResultGen::Result(result) => result,
        };

        let mut exec_result = exec.last_frame_result(ctx, frame_result)?;

        //self.handler.set_instruction_table(instructions);

        let post_exec = self.handler.post_execution();
        // calculate final refund and add EIP-7702 refund to gas.
        post_exec.refund(ctx, &mut exec_result, eip7702_gas_refund);
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, &mut exec_result)?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, &mut exec_result)?;
        // Returns output of transaction.
        post_exec.output(ctx, exec_result)
    }
}

/*
TODO TESTS
#[cfg(test)]
mod test {
    use core::cell::RefCell;
    use database_interface::EmptyDB;
    use std::{rc::Rc, sync::Arc};
    use wiring::{result::EVMError, EthereumWiring, EvmWiring};

    use super::*;

    type TestEvmWiring = EthereumWiring<EmptyDB, ()>;

    #[test]
    fn test_handler_register_pop() {
        let register = |inner: &Rc<RefCell<i32>>| -> HandleRegisterBox<'_, TestEvmWiring> {
            let inner = inner.clone();
            Box::new(move |h| {
                *inner.borrow_mut() += 1;
                //h.post_execution.output = Arc::new(|_, _| Err(EVMError::Custom("test".into())))
            })
        };

        let mut handler = EvmHandler::<'_, TestEvmWiring>::mainnet_with_spec(
            <TestEvmWiring as EvmWiring>::Hardfork::default(),
        );
        let test = Rc::new(RefCell::new(0));

        handler.append_handler_register_box(register(&test));
        assert_eq!(*test.borrow(), 1);

        handler.append_handler_register_box(register(&test));
        assert_eq!(*test.borrow(), 2);

        assert!(handler.pop_handle_register().is_some());

        // first handler is reapplied
        assert_eq!(*test.borrow(), 3);
    }
}
 */
