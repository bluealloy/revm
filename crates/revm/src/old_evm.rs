use crate::{
    builder::{EvmBuilder, SetGenericStage},
    handler::{EthHandler, FrameOrResultGen, Hand, Handler},
    Context, ContextWithEvmWiring, EvmContext, EvmWiring, InnerEvmContext,
};
use context::JournaledState;
use core::fmt::{self, Debug};
use database_interface::{Database, DatabaseCommit};
use interpreter::Host;
use std::boxed::Box;
use context_interface::{
    default::{CfgEnv, EnvWiring},
    result::{EVMError, EVMResult, EVMResultGeneric, ExecutionResult, ResultAndState},
    Transaction,
};

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Evm<'a, EvmWiringT: EvmWiring> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EvmWiringT>,
    /// Handler is a component of the of EVM that contains all the logic. Handler contains specification id
    /// and it different depending on the specified fork.
    pub handler: Handler<'a, EvmWiringT, Context<EvmWiringT>>,
    //pub handler: EthHandler<,
}

impl<EvmWiringT> Debug for Evm<'_, EvmWiringT>
where
    EvmWiringT:
        EvmWiring<Block: Debug, Transaction: Debug, Database: Debug, ExternalContext: Debug>,
    <EvmWiringT::Database as Database>::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EvmWiringT: EvmWiring<Database: DatabaseCommit>> Evm<'_, EvmWiringT> {
    /// Commit the changes to the database.
    pub fn transact_commit(
        &mut self,
    ) -> EVMResultGeneric<ExecutionResult<EvmWiringT::HaltReason>, EvmWiringT> {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.journaled_state.database.commit(state);
        Ok(result)
    }
}

impl<'a, EvmWiringT: EvmWiring> Evm<'a, EvmWiringT>
where
    EvmWiringT::Transaction: Default,
    EvmWiringT::Block: Default,
{
    /// Returns evm builder with the mainnet chain spec, empty database, and empty external context.
    pub fn builder() -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        EvmBuilder::new()
    }
}

impl<'a, EvmWiringT: EvmWiring> Evm<'a, EvmWiringT> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<EvmWiringT>,
        handler: Handler<'a, EvmWiringT, Context<EvmWiringT>>,
    ) -> Evm<'a, EvmWiringT> {
        context
            .evm
            .journaled_state
            .set_spec_id(handler.spec_id.into());
        Evm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        let Evm {
            context:
                Context {
                    evm:
                        EvmContext {
                            inner:
                                InnerEvmContext {
                                    journaled_state: JournaledState { database, .. },
                                    env,
                                    ..
                                },
                            ..
                        },
                    external,
                },
            handler,
        } = self;
        EvmBuilder::<'a>::new_with(database, external, env, handler)
    }

    // /// Runs main call loop.
    // #[inline]
    // pub fn run_the_loop(
    //     &mut self,
    //     first_frame: Frame,
    // ) -> EVMResultGeneric<FrameResult, EvmWiringT> {
    //     let mut call_stack: Vec<Frame> = Vec::with_capacity(1025);
    //     call_stack.push(first_frame);

    //     #[cfg(feature = "memory_limit")]
    //     let mut shared_memory =
    //         SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
    //     #[cfg(not(feature = "memory_limit"))]
    //     let mut shared_memory = SharedMemory::new();

    //     shared_memory.new_context();

    //     // Peek the last stack frame.
    //     let mut stack_frame = call_stack.last_mut().unwrap();

    //     loop {
    //         // Execute the frame.
    //         let next_action =
    //             self.handler
    //                 .execute_frame(stack_frame, &mut shared_memory, &mut self.context)?;

    //         // Take error and break the loop, if any.
    //         // This error can be set in the Interpreter when it interacts with the context.
    //         self.context.evm.take_error().map_err(EVMError::Database)?;

    //         let exec = &mut self.handler.execution;
    //         let frame_or_result = match next_action {
    //             InterpreterAction::NewFrame(NewFrameAction::Call(inputs)) => {
    //                 exec.call(&mut self.context, inputs)?
    //             }
    //             InterpreterAction::NewFrame(NewFrameAction::Create(inputs)) => {
    //                 exec.create(&mut self.context, inputs)?
    //             }
    //             InterpreterAction::NewFrame(NewFrameAction::EOFCreate(inputs)) => {
    //                 exec.eofcreate(&mut self.context, inputs)?
    //             }
    //             InterpreterAction::Return { result } => {
    //                 // free memory context.
    //                 shared_memory.free_context();

    //                 // pop last frame from the stack and consume it to create FrameResult.
    //                 let returned_frame = call_stack
    //                     .pop()
    //                     .expect("We just returned from Interpreter frame");

    //                 let ctx = &mut self.context;
    //                 FrameOrResult::Result(match returned_frame {
    //                     Frame::Call(frame) => {
    //                         // return_call
    //                         FrameResult::Call(exec.call_return(ctx, frame, result)?)
    //                     }
    //                     Frame::Create(frame) => {
    //                         // return_create
    //                         FrameResult::Create(exec.create_return(ctx, frame, result)?)
    //                     }
    //                     Frame::EOFCreate(frame) => {
    //                         // return_eofcreate
    //                         FrameResult::EOFCreate(exec.eofcreate_return(ctx, frame, result)?)
    //                     }
    //                 })
    //             }
    //             InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
    //         };
    //         // handle result
    //         match frame_or_result {
    //             FrameOrResult::Frame(frame) => {
    //                 shared_memory.new_context();
    //                 call_stack.push(frame);
    //                 stack_frame = call_stack.last_mut().unwrap();
    //             }
    //             FrameOrResult::Result(result) => {
    //                 let Some(top_frame) = call_stack.last_mut() else {
    //                     // Break the loop if there are no more frames.
    //                     return Ok(result);
    //                 };
    //                 stack_frame = top_frame;
    //                 let ctx = &mut self.context;
    //                 // Insert result to the top frame.
    //                 match result {
    //                     FrameResult::Call(outcome) => {
    //                         // return_call
    //                         exec.insert_call_outcome(ctx, stack_frame, &mut shared_memory, outcome)?
    //                     }
    //                     FrameResult::Create(outcome) => {
    //                         // return_create
    //                         exec.insert_create_outcome(ctx, stack_frame, outcome)?
    //                     }
    //                     FrameResult::EOFCreate(outcome) => {
    //                         // return_eofcreate
    //                         exec.insert_eofcreate_outcome(ctx, stack_frame, outcome)?
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}

impl<EvmWiringT: EvmWiring> Evm<'_, EvmWiringT> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> EvmWiringT::Hardfork {
        self.handler.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balance to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> EVMResultGeneric<(), EvmWiringT> {
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
    pub fn transact_preverified(&mut self) -> EVMResult<EvmWiringT> {
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
    fn preverify_transaction_inner(&mut self) -> EVMResultGeneric<u64, EvmWiringT> {
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
    pub fn transact(&mut self) -> EVMResult<EvmWiringT> {
        let initial_gas_spend = self.preverify_transaction_inner().inspect_err(|_| {
            self.clear();
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of Env configuration
    #[inline]
    pub fn cfg(&self) -> &CfgEnv {
        &self.context.env().cfg
    }

    /// Returns the mutable reference of Env configuration
    #[inline]
    pub fn cfg_mut(&mut self) -> &mut CfgEnv {
        &mut self.context.evm.env.cfg
    }

    /// Returns the reference of transaction
    #[inline]
    pub fn tx(&self) -> &EvmWiringT::Transaction {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut EvmWiringT::Transaction {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &EvmWiringT::Database {
        &self.context.evm.journaled_state.database
    }

    /// Returns the mutable reference of database
    #[inline]
    pub fn db_mut(&mut self) -> &mut EvmWiringT::Database {
        &mut self.context.evm.journaled_state.database
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &EvmWiringT::Block {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut EvmWiringT::Block {
        &mut self.context.evm.env.block
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(&mut self, spec_id: EvmWiringT::Hardfork) {
        self.context.evm.journaled_state.set_spec_id(spec_id.into());
        self.handler.modify_spec_id(spec_id);
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EvmWiringT> {
        self.context
    }

    /// Returns database, [`EnvWiring`] and Hardfork.
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(
        self,
    ) -> (
        EvmWiringT::Database,
        Box<EnvWiring<EvmWiringT>>,
        EvmWiringT::Hardfork,
    ) {
        (
            self.context.evm.inner.journaled_state.database,
            self.context.evm.inner.env,
            self.handler.spec_id,
        )
    }

    /// Returns [Context] and hardfork.
    #[inline]
    pub fn into_context_with_spec_id(self) -> ContextWithEvmWiring<EvmWiringT> {
        ContextWithEvmcontext_interface::new(self.context, self.handler.spec_id)
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<EvmWiringT> {
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // load precompiles
        let precompiles = pre_exec.load_precompiles();
        // TODO SET PRECOMPILE
        //ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.common_fields().gas_limit() - initial_gas_spend;

        // apply EIP-7702 auth list.
        let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(ctx)? as i64;

        // start execution

        let instructions = self.handler.take_instruction_table();
        let exec = self.handler.execution();

        // create first frame action
        let first_frame = exec.init_first_frame(ctx, gas_limit)?;
        let frame_result = match first_frame {
            FrameOrResultGen::Frame(frame) => exec.run(ctx, frame)?,
            FrameOrResultGen::Result(result) => result,
        };

        let mut exec_result = exec.last_frame_result(ctx, frame_result)?;

        self.handler.set_instruction_table(instructions);

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
            .with_default_ext_ctx()
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
