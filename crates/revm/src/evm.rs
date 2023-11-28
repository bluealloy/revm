use crate::{
    db::{Database, EmptyDB},
    evm_builder::EvmBuilder,
    handler::Handler,
    handler::{MainnetHandle, RegisterHandler},
    interpreter::{
        opcode::InstructionTables, CallContext, CallInputs, CallScheme, CreateInputs, Host,
        Interpreter, InterpreterAction, InterpreterResult, SelfDestructResult, SharedMemory,
        Transfer,
    },
    journaled_state::JournaledState,
    precompile::Precompiles,
    primitives::{
        specification::{self, SpecId},
        Address, Bytecode, Bytes, EVMError, EVMResult, Env, HashSet, InvalidTransaction, Output,
        SpecId::*,
        TransactTo, B256, U256,
    },
    CallStackFrame, Context, EvmContext, FrameOrResult,
};
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

#[cfg(feature = "optimism")]
use crate::optimism;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct Evm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler of EVM that contains all the logic.
    pub handler: Handler<'a, Self, EXT, DB>,
    /// Specification id.
    pub spec_id: SpecId,
}

impl<EXT, DB> fmt::Debug for Evm<'_, EXT, DB>
where
    EXT: fmt::Debug,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("data", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<'a> Evm<'a, MainnetHandle, EmptyDB> {
    /// Returns evm builder.
    pub fn builder() -> EvmBuilder<'a, MainnetHandle, EmptyDB> {
        EvmBuilder::default()
    }
}

impl<'a, EXT, DB: Database> Evm<'a, EXT, DB>
where
    EXT: RegisterHandler<'a, DB, EXT>,
{
    pub fn new(context: Context<EXT, DB>, handler: Handler<'a, Self, EXT, DB>) -> Evm<'a, EXT, DB> {
        let spec_id = handler.spec_id;
        Evm {
            context,
            handler,
            spec_id,
        }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// to the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, EXT, DB> {
        EvmBuilder::new(self)
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(self, spec_id: SpecId) -> Self {
        if self.spec_id == spec_id {
            return self;
        }
        self.modify().with_spec_id(spec_id).build()
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// TODO add spec_id as variable.
    pub fn new_with_spec(
        db: DB,
        env: Box<Env>,
        external: EXT,
        handler: Handler<'a, Self, EXT, DB>,
        precompiles: Precompiles,
    ) -> Self {
        let spec_id = handler.spec_id;
        let journaled_state = JournaledState::new(
            spec_id,
            precompiles
                .addresses()
                .into_iter()
                .cloned()
                .collect::<HashSet<_>>(),
        );

        Self {
            context: Context {
                evm: EvmContext {
                    env,
                    journaled_state,
                    db,
                    error: None,
                    precompiles,
                    #[cfg(feature = "optimism")]
                    l1_block_info: None,
                },
                external,
            },
            spec_id,
            handler,
        }
    }

    /// Runs main call loop.
    #[inline]
    pub fn run<FN>(
        &mut self,
        instruction_table: &[FN; 256],
        first_frame: Box<CallStackFrame>,
    ) -> InterpreterResult
    where
        FN: Fn(&mut Interpreter, &mut Self),
    {
        let mut call_stack: Vec<Box<CallStackFrame>> = Vec::with_capacity(1025);
        call_stack.push(first_frame);

        #[cfg(feature = "memory_limit")]
        let mut shared_memory =
            SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();

        let mut stack_frame = call_stack.first_mut().unwrap();

        loop {
            // run interpreter
            let action = stack_frame
                .interpreter
                .run(shared_memory, instruction_table, self);
            // take shared memory back.
            shared_memory = stack_frame.interpreter.take_memory();

            let new_frame = match action {
                InterpreterAction::SubCall {
                    inputs,
                    return_memory_offset,
                } => self.handler.frame_sub_call(
                    &mut self.context,
                    inputs,
                    stack_frame,
                    &mut shared_memory,
                    return_memory_offset,
                ),
                InterpreterAction::Create { inputs } => {
                    self.handler
                        .frame_sub_create(&mut self.context, stack_frame, inputs)
                }
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    let child = call_stack.pop().unwrap();
                    let parent = call_stack.last_mut();

                    if let Some(result) = self.handler.frame_return(
                        &mut self.context,
                        child,
                        parent,
                        &mut shared_memory,
                        result,
                    ) {
                        return result;
                    }
                    stack_frame = call_stack.last_mut().unwrap();
                    continue;
                }
            };
            if let Some(new_frame) = new_frame {
                shared_memory.new_context();
                call_stack.push(new_frame);
            }
            stack_frame = call_stack.last_mut().unwrap();
        }
    }

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.handler.validate_env(&self.context.evm.env)?;
        let initial_gas_spend = self.handler.initial_tx_gas(&self.context.evm.env);

        let env = self.env();

        // Additional check to see if limit is big enough to cover initial gas.
        if initial_gas_spend > env.tx.gas_limit {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
        }

        // load acc
        let tx_caller = env.tx.caller;
        let (caller_account, _) = self
            .context
            .evm
            .journaled_state
            .load_account(tx_caller, &mut self.context.evm.db)
            .map_err(EVMError::Database)?;

        self.context
            .evm
            .env
            .validate_tx_against_state(caller_account)
            .map_err(Into::into)
    }

    /// Transact preverified transaction.
    pub fn transact_preverified_inner(&mut self) -> EVMResult<DB::Error> {
        let env = &self.context.evm.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;

        // the L1-cost fee is only computed for Optimism non-deposit transactions.
        #[cfg(feature = "optimism")]
        let tx_l1_cost = if env.cfg.optimism && env.tx.optimism.source_hash.is_none() {
            let l1_block_info = optimism::L1BlockInfo::try_fetch(self.context.evm.db)
                .map_err(EVMError::Database)?;

            let Some(enveloped_tx) = &env.tx.optimism.enveloped_tx else {
                panic!("[OPTIMISM] Failed to load enveloped transaction.");
            };
            // TODO specs
            let tx_l1_cost = l1_block_info.calculate_tx_l1_cost(enveloped_tx, self.spec_id);

            // storage l1 block info for later use.
            self.context.evm.l1_block_info = Some(l1_block_info);

            tx_l1_cost
        } else {
            U256::ZERO
        };

        let initial_gas_spend = self.handler.initial_tx_gas(&self.context.evm.env);

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if self.spec_id.is_enabled_in(SHANGHAI) {
            self.context
                .evm
                .journaled_state
                .initial_account_load(
                    self.context.evm.env.block.coinbase,
                    &[],
                    &mut self.context.evm.db,
                )
                .map_err(EVMError::Database)?;
        }

        self.context.evm.load_access_list()?;

        // load acc
        let journal = &mut self.context.evm.journaled_state;

        #[cfg(feature = "optimism")]
        if self.context.evm.env.cfg.optimism {
            crate::optimism::commit_mint_value(
                tx_caller,
                self.context.evm.env.tx.optimism.mint,
                self.context.evm.db,
                journal,
            )?;

            let is_deposit = self.context.evm.env.tx.optimism.source_hash.is_some();
            crate::optimism::remove_l1_cost(
                is_deposit,
                tx_caller,
                tx_l1_cost,
                self.context.evm.db,
                journal,
            )?;
        }

        let (caller_account, _) = journal
            .load_account(tx_caller, &mut self.context.evm.db)
            .map_err(EVMError::Database)?;

        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            U256::from(tx_gas_limit).saturating_mul(self.context.evm.env.effective_gas_price());

        // EIP-4844
        if self.spec_id.is_enabled_in(CANCUN) {
            let data_fee = self
                .context
                .evm
                .env
                .calc_data_fee()
                .expect("already checked");
            gas_cost = gas_cost.saturating_add(data_fee);
        }

        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

        // touch account so we know it is changed.
        caller_account.mark_touch();

        let transact_gas_limit = tx_gas_limit - initial_gas_spend;

        // call inner handling of call/create
        let first_stack_frame = match self.context.evm.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                self.context.evm.make_call_frame(
                    &CallInputs {
                        contract: address,
                        transfer: Transfer {
                            source: tx_caller,
                            target: address,
                            value: tx_value,
                        },
                        input: tx_data,
                        gas_limit: transact_gas_limit,
                        context: CallContext {
                            caller: tx_caller,
                            address,
                            code_address: address,
                            apparent_value: tx_value,
                            scheme: CallScheme::Call,
                        },
                        is_static: false,
                    },
                    0..0,
                )
            }
            TransactTo::Create(scheme) => self.context.evm.make_create_frame(
                self.spec_id,
                &CreateInputs {
                    caller: tx_caller,
                    scheme,
                    value: tx_value,
                    init_code: tx_data,
                    gas_limit: transact_gas_limit,
                },
            ),
        };
        // Some only if it is create.
        let mut created_address = None;

        // start main loop if CallStackFrame is created correctly
        let interpreter_result = match first_stack_frame {
            FrameOrResult::Frame(first_stack_frame) => {
                created_address = first_stack_frame.created_address;
                let table = self.handler.instruction_table.clone();
                match table {
                    InstructionTables::Plain(table) => self.run(&table, first_stack_frame),
                    InstructionTables::Boxed(table) => self.run(&table, first_stack_frame),
                }
            }
            FrameOrResult::Result(interpreter_result) => interpreter_result,
        };

        let handler = &self.handler;
        let context = &mut self.context;

        // handle output of call/create calls.
        let mut gas = handler.call_return(
            &mut context.evm.env,
            interpreter_result.result,
            interpreter_result.gas,
        );

        // set refund. Refund amount depends on hardfork.
        gas.set_refund(handler.calculate_gas_refund(&mut context.evm.env, &gas) as i64);

        // Reimburse the caller
        handler.reimburse_caller(context, &gas)?;

        // Reward beneficiary
        handler.reward_beneficiary(context, &gas)?;

        // output of execution
        let output = match context.evm.env.tx.transact_to {
            TransactTo::Call(_) => Output::Call(interpreter_result.output),
            TransactTo::Create(_) => Output::Create(interpreter_result.output, created_address),
        };

        // main return
        handler.main_return(context, interpreter_result.result, output, &gas)
    }
}

/// EVM transaction interface.
pub trait Transact<DB: Database, EXT> {
    /// Run checks that could make transaction fail before call/create.
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>>;

    /// Skip pre-verification steps and execute the transaction.
    fn transact_preverified(&mut self) -> EVMResult<DB::Error>;

    /// Execute transaction by running pre-verification steps and then transaction itself.
    fn transact(&mut self) -> EVMResult<DB::Error>;
}

impl<'a, EXT: RegisterHandler<'a, DB, EXT>, DB: Database> Transact<DB, EXT> for Evm<'a, EXT, DB> {
    #[inline]
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.preverify_transaction_inner()
    }

    #[inline]
    fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let output = self.transact_preverified_inner();
        self.handler.end(&mut self.context, output)
    }

    #[inline]
    fn transact(&mut self) -> EVMResult<DB::Error> {
        let output = self
            .preverify_transaction_inner()
            .and_then(|()| self.transact_preverified_inner());
        self.handler.end(&mut self.context, output)
    }
}

impl<'a, EXT, DB: Database> Host for Evm<'a, EXT, DB> {
    fn env(&mut self) -> &mut Env {
        self.context.evm.env()
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.context.evm.block_hash(number)
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.context.evm.load_account(address)
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.context.evm.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        self.context.evm.code(address)
    }

    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.context.evm.code_hash(address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.context.evm.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.context.evm.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.context.evm.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.context.evm.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        self.handler
            .host_log(&mut self.context, address, topics, data);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        self.handler
            .host_selfdestruct(&mut self.context, address, target)
    }
}

/// Creates new EVM instance with erased types.
pub fn new_evm<'a, EXT: RegisterHandler<'a, DB, EXT> + 'a, DB: Database + 'a>(
    env: Box<Env>,
    db: DB,
    external: EXT,
) -> Evm<'a, EXT, DB> {
    macro_rules! create_evm {
        ($spec:ident) => {{
            let handler = external.register_handler::<$spec>(Handler::mainnet::<$spec>());
            Evm::new_with_spec(
                db,
                env,
                external,
                handler,
                Precompiles::new(revm_precompile::SpecId::from_spec_id($spec::SPEC_ID)).clone(),
            )
        }};
    }

    use specification::*;
    match env.cfg.spec_id {
        SpecId::FRONTIER | SpecId::FRONTIER_THAWING => create_evm!(FrontierSpec),
        SpecId::HOMESTEAD | SpecId::DAO_FORK => create_evm!(HomesteadSpec),
        SpecId::TANGERINE => create_evm!(TangerineSpec),
        SpecId::SPURIOUS_DRAGON => create_evm!(SpuriousDragonSpec),
        SpecId::BYZANTIUM => create_evm!(ByzantiumSpec),
        SpecId::PETERSBURG | SpecId::CONSTANTINOPLE => create_evm!(PetersburgSpec),
        SpecId::ISTANBUL | SpecId::MUIR_GLACIER => create_evm!(IstanbulSpec),
        SpecId::BERLIN => create_evm!(BerlinSpec),
        SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => {
            create_evm!(LondonSpec)
        }
        SpecId::MERGE => create_evm!(MergeSpec),
        SpecId::SHANGHAI => create_evm!(ShanghaiSpec),
        SpecId::CANCUN => create_evm!(CancunSpec),
        SpecId::LATEST => create_evm!(LatestSpec),
        #[cfg(feature = "optimism")]
        SpecId::BEDROCK => create_evm!(BedrockSpec),
        #[cfg(feature = "optimism")]
        SpecId::REGOLITH => create_evm!(RegolithSpec),
    }
}

#[cfg(feature = "optimism")]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::db::InMemoryDB;
    use crate::primitives::{specification::BedrockSpec, state::AccountInfo, SpecId};

    #[test]
    fn test_commit_mint_value() {
        let caller = Address::ZERO;
        let mint_value = Some(1u128);
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(Evm::<BedrockSpec, InMemoryDB>::commit_mint_value(
            caller,
            mint_value,
            &mut db,
            &mut journal
        )
        .is_ok(),);

        // Check the account balance is updated.
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));

        // No mint value should be a no-op.
        assert!(Evm::<BedrockSpec, InMemoryDB>::commit_mint_value(
            caller,
            None,
            &mut db,
            &mut journal
        )
        .is_ok(),);
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(101));
    }

    #[test]
    fn test_remove_l1_cost_non_deposit() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        let slots = &[U256::from(100)];
        journal
            .initial_account_load(caller, slots, &mut db)
            .unwrap();
        assert!(Evm::<BedrockSpec, InMemoryDB>::remove_l1_cost(
            true,
            caller,
            U256::ZERO,
            &mut db,
            &mut journal
        )
        .is_ok(),);
    }

    #[test]
    fn test_remove_l1_cost() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert!(Evm::<BedrockSpec, InMemoryDB>::remove_l1_cost(
            false,
            caller,
            U256::from(1),
            &mut db,
            &mut journal
        )
        .is_ok(),);

        // Check the account balance is updated.
        let (account, _) = journal.load_account(caller, &mut db).unwrap();
        assert_eq!(account.info.balance, U256::from(99));
    }

    #[test]
    fn test_remove_l1_cost_lack_of_funds() {
        let caller = Address::ZERO;
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                nonce: 0,
                balance: U256::from(100),
                code_hash: B256::ZERO,
                code: None,
            },
        );
        let mut journal = JournaledState::new(SpecId::BERLIN, vec![]);
        journal
            .initial_account_load(caller, &[U256::from(100)], &mut db)
            .unwrap();
        assert_eq!(
            Evm::<BedrockSpec, InMemoryDB>::remove_l1_cost(
                false,
                caller,
                U256::from(101),
                &mut db,
                &mut journal
            ),
            Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: 101u64,
                    balance: U256::from(100),
                },
            ))
        );
    }
}
