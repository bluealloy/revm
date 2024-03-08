mod context_precompiles;
mod evm_context;

pub use context_precompiles::{
    ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile, ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
};
pub use evm_context::EvmContext;

use crate::{
    db::{Database, EmptyDB},
    interpreter::{
        return_ok, CallInputs, Contract, Gas, InstructionResult, Interpreter, InterpreterResult,
    },
    primitives::{Address, Bytes, EVMError, HandlerCfg, HashSet, U256},
    FrameOrResult, CALL_STACK_LIMIT,
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<EXT, DB: Database> {
    /// Evm Context.
    pub evm: EvmContext<DB>,
    /// External contexts.
    pub external: EXT,
    /// Precompiles that are available for evm.
    pub precompiles: ContextPrecompiles<DB>,
}

impl<EXT: Clone, DB: Database + Clone> Clone for Context<EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            evm: self.evm.clone(),
            external: self.external.clone(),
            precompiles: self.precompiles.clone(),
        }
    }
}

impl Default for Context<(), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl Context<(), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<(), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
            precompiles: ContextPrecompiles::default(),
        }
    }
}

impl<DB: Database> Context<(), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<(), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
            precompiles: ContextPrecompiles::default(),
        }
    }
}

impl<EXT, DB: Database> Context<EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<DB>, external: EXT) -> Context<EXT, DB> {
        Context {
            evm,
            external,
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Sets precompiles
    #[inline]
    pub fn set_precompiles(&mut self, precompiles: ContextPrecompiles<DB>) {
        // set warm loaded addresses.
        self.evm.journaled_state.warm_preloaded_addresses =
            precompiles.addresses().copied().collect::<HashSet<_>>();
        self.precompiles = precompiles;
    }

    /// Call precompile contract
    #[inline]
    fn call_precompile(
        &mut self,
        address: Address,
        input_data: &Bytes,
        gas: Gas,
    ) -> Option<InterpreterResult> {
        let out = self
            .precompiles
            .call(address, input_data, gas.limit(), &mut self.evm)?;

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas,
            output: Bytes::new(),
        };

        match out {
            Ok((gas_used, data)) => {
                if result.gas.record_cost(gas_used) {
                    result.result = InstructionResult::Return;
                    result.output = data;
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Err(e) => {
                result.result = if e == crate::precompile::Error::OutOfGas {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
        }
        Some(result)
    }

    /// Make call frame
    #[inline]
    pub fn make_call_frame(
        &mut self,
        inputs: &CallInputs,
    ) -> Result<FrameOrResult, EVMError<DB::Error>> {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            Ok(FrameOrResult::new_call_result(
                InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                inputs.return_memory_offset.clone(),
            ))
        };

        // Check depth
        if self.evm.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_result(InstructionResult::CallTooDeep);
        }

        let (account, _) = self
            .evm
            .inner
            .journaled_state
            .load_code(inputs.contract, &mut self.evm.inner.db)?;
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Create subroutine checkpoint
        let checkpoint = self.evm.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.evm.load_account(inputs.context.address)?;
            self.evm.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Some(result) = self.evm.inner.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            &mut self.evm.inner.db,
        )? {
            self.evm.journaled_state.checkpoint_revert(checkpoint);
            return return_result(result);
        }

        if let Some(result) = self.call_precompile(inputs.contract, &inputs.input, gas) {
            if matches!(result.result, return_ok!()) {
                self.evm.journaled_state.checkpoint_commit();
            } else {
                self.evm.journaled_state.checkpoint_revert(checkpoint);
            }
            Ok(FrameOrResult::new_call_result(
                result,
                inputs.return_memory_offset.clone(),
            ))
        } else if !bytecode.is_empty() {
            let contract = Box::new(Contract::new_with_context(
                inputs.input.clone(),
                bytecode,
                code_hash,
                &inputs.context,
            ));
            // Create interpreter and executes call and push new CallStackFrame.
            Ok(FrameOrResult::new_call_frame(
                inputs.return_memory_offset.clone(),
                checkpoint,
                Interpreter::new(contract, gas.limit(), inputs.is_static),
            ))
        } else {
            self.evm.journaled_state.checkpoint_commit();
            return_result(InstructionResult::Stop)
        }
    }
}

/// Context with handler configuration.
pub struct ContextWithHandlerCfg<EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<EXT, DB>,
    /// Handler configuration.
    pub cfg: HandlerCfg,
}

impl<EXT, DB: Database> ContextWithHandlerCfg<EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<EXT, DB>, cfg: HandlerCfg) -> Self {
        Self { cfg, context }
    }
}

impl<EXT: Clone, DB: Database + Clone> Clone for ContextWithHandlerCfg<EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            cfg: self.cfg,
        }
    }
}

/// Test utilities for the [`EvmContext`].
#[cfg(any(test, feature = "test-utils"))]
pub(crate) mod test_utils {
    use self::evm_context::InnerEvmContext;

    use super::*;
    use crate::{
        db::{CacheDB, EmptyDB},
        journaled_state::JournaledState,
        primitives::{address, Address, Bytes, Env, HashSet, SpecId, B256, U256},
    };
    use std::boxed::Box;

    /// Mock caller address.
    pub const MOCK_CALLER: Address = address!("0000000000000000000000000000000000000000");

    /// Creates `CallInputs` that calls a provided contract address from the mock caller.
    pub fn create_mock_call_inputs(to: Address) -> CallInputs {
        CallInputs {
            contract: to,
            transfer: revm_interpreter::Transfer {
                source: MOCK_CALLER,
                target: to,
                value: U256::ZERO,
            },
            input: Bytes::new(),
            gas_limit: 0,
            context: revm_interpreter::CallContext {
                address: MOCK_CALLER,
                caller: MOCK_CALLER,
                code_address: MOCK_CALLER,
                apparent_value: U256::ZERO,
                scheme: revm_interpreter::CallScheme::Call,
            },
            is_static: false,
            return_memory_offset: 0..0,
        }
    }

    /// Creates an evm context with a cache db backend.
    /// Additionally loads the mock caller account into the db,
    /// and sets the balance to the provided U256 value.
    pub fn create_cache_db_evm_context_with_balance(
        env: Box<Env>,
        mut db: CacheDB<EmptyDB>,
        balance: U256,
    ) -> Context<(), CacheDB<EmptyDB>> {
        db.insert_account_info(
            test_utils::MOCK_CALLER,
            crate::primitives::AccountInfo {
                nonce: 0,
                balance,
                code_hash: B256::default(),
                code: None,
            },
        );
        create_cache_db_evm_context(env, db)
    }

    /// Creates a cached db evm context.
    pub fn create_cache_db_evm_context(
        env: Box<Env>,
        db: CacheDB<EmptyDB>,
    ) -> Context<(), CacheDB<EmptyDB>> {
        Context {
            evm: EvmContext {
                inner: InnerEvmContext {
                    env,
                    journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
                    db,
                    error: Ok(()),
                    #[cfg(feature = "optimism")]
                    l1_block_info: None,
                },
                precompiles: (),
            },
            external: (),
            precompiles: ContextPrecompiles::default(),
        }
    }

    /// Returns a new `EvmContext` with an empty journaled state.
    pub fn create_empty_evm_context(env: Box<Env>, db: EmptyDB) -> Context<(), EmptyDB> {
        Context::new(
            EvmContext {
                inner: InnerEvmContext {
                    env,
                    journaled_state: JournaledState::new(SpecId::CANCUN, HashSet::new()),
                    db,
                    error: Ok(()),
                    #[cfg(feature = "optimism")]
                    l1_block_info: None,
                },
                precompiles: (),
            },
            (),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    use crate::{
        db::{CacheDB, EmptyDB},
        interpreter::InstructionResult,
        primitives::{address, Bytecode, Bytes, Env, U256},
        Frame, FrameOrResult, JournalEntry,
    };
    use std::boxed::Box;

    // Tests that the `EVMContext::make_call_frame` function returns an error if the
    // call stack is too deep.
    #[test]
    fn test_make_call_frame_stack_too_deep() {
        let env = Env::default();
        let db = EmptyDB::default();
        let mut context = test_utils::create_empty_evm_context(Box::new(env), db);
        context.evm.journaled_state.depth = CALL_STACK_LIMIT as usize + 1;
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(err)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(
            err.interpreter_result().result,
            InstructionResult::CallTooDeep
        );
    }

    // Tests that the `EVMContext::make_call_frame` function returns an error if the
    // transfer fails on the journaled state. It also verifies that the revert was
    // checkpointed on the journaled state correctly.
    #[test]
    fn test_make_call_frame_transfer_revert() {
        let env = Env::default();
        let db = EmptyDB::default();
        let mut evm_context = test_utils::create_empty_evm_context(Box::new(env), db);
        let contract = address!("dead10000000000000000000000000000001dead");
        let mut call_inputs = test_utils::create_mock_call_inputs(contract);
        call_inputs.transfer.value = U256::from(1);
        let res = evm_context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(result)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(
            result.interpreter_result().result,
            InstructionResult::OutOfFunds
        );
        let checkpointed = vec![vec![JournalEntry::AccountLoaded { address: contract }]];
        assert_eq!(evm_context.evm.journaled_state.journal, checkpointed);
        assert_eq!(evm_context.evm.journaled_state.depth, 0);
    }

    #[test]
    fn test_make_call_frame_missing_code_context() {
        let env = Env::default();
        let cdb = CacheDB::new(EmptyDB::default());
        let bal = U256::from(3_000_000_000_u128);
        let mut context = create_cache_db_evm_context_with_balance(Box::new(env), cdb, bal);
        let contract = address!("dead10000000000000000000000000000001dead");
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Result(result)) = res else {
            panic!("Expected FrameOrResult::Result");
        };
        assert_eq!(result.interpreter_result().result, InstructionResult::Stop);
    }

    #[test]
    fn test_make_call_frame_succeeds() {
        let env = Env::default();
        let mut cdb = CacheDB::new(EmptyDB::default());
        let bal = U256::from(3_000_000_000_u128);
        let by = Bytecode::new_raw(Bytes::from(vec![0x60, 0x00, 0x60, 0x00]));
        let contract = address!("dead10000000000000000000000000000001dead");
        cdb.insert_account_info(
            contract,
            crate::primitives::AccountInfo {
                nonce: 0,
                balance: bal,
                code_hash: by.clone().hash_slow(),
                code: Some(by),
            },
        );
        let mut evm_context = create_cache_db_evm_context_with_balance(Box::new(env), cdb, bal);
        let call_inputs = test_utils::create_mock_call_inputs(contract);
        let res = evm_context.make_call_frame(&call_inputs);
        let Ok(FrameOrResult::Frame(Frame::Call(call_frame))) = res else {
            panic!("Expected FrameOrResult::Frame(Frame::Call(..))");
        };
        assert_eq!(call_frame.return_memory_range, 0..0,);
    }
}
