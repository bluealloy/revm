use crate::{
    handler::{
        EthPrecompileProvider, ExecutionWire, Frame, FrameOrResultGen, Handler, PostExecutionWire, PreExecutionWire, ValidationWire
    },
    mainnet::{EthExecution, EthFrame, EthHandler, EthPreExecution, EthValidation, FrameResult},
};
use context::{
    default::{block::BlockEnv, tx::TxEnv},
    BlockGetter, CfgGetter, Context, DatabaseGetter, ErrorGetter, JournalStateGetter,
    JournalStateGetterDBError, TransactionGetter,
};
use database_interface::Database;
use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter},
    Host,
};
use precompile::PrecompileErrors;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
use std::vec::Vec;
use context_interface::{
    journaled_state::JournaledState,
    result::{EVMError, HaltReason, InvalidHeader, InvalidTransaction, ResultAndState},
    Transaction,
};

/// Main EVM structure
pub struct Evm<ERROR, CTX = Context, HAND = EthHandler<CTX, ERROR>> {
    pub context: CTX,
    pub handler: HAND,
    pub _error: std::marker::PhantomData<fn() -> ERROR>,
}

pub type Error<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

pub type EthContext<DB> = Context<BlockEnv, TxEnv, SpecId, DB, ()>;

pub type MainEvm<DB> = Evm<
    Error<DB>,
    EthContext<DB>,
    EthHandler<
        EthContext<DB>,
        Error<DB>,
        EthValidation<EthContext<DB>, Error<DB>>,
        EthPreExecution<EthContext<DB>, Error<DB>>,
        EthExecution<
            EthContext<DB>,
            Error<DB>,
            EthFrame<
                EthContext<DB>,
                Error<DB>,
                EthInterpreter<()>,
                EthPrecompileProvider<EthContext<DB>, Error<DB>>,
                EthInstructionProvider<EthInterpreter<()>, EthContext<DB>>,
            >,
        >,
    >,
>;

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
