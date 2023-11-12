use crate::{
    db::Database,
    handler::Handler,
    inspector_instruction,
    interpreter::{
        gas::initial_tx_gas,
        opcode::{make_boxed_instruction_table, make_instruction_table, InstructionTables},
        CallContext, CallInputs, CallScheme, CreateInputs, Host, Interpreter, InterpreterAction,
        InterpreterResult, SelfDestructResult, SharedMemory, Transfer,
    },
    journaled_state::JournaledState,
    precompile::Precompiles,
    primitives::{
        specification, Address, Bytecode, Bytes, EVMError, EVMResult, Env, InvalidTransaction, Log,
        Output, Spec, SpecId::*, TransactTo, B256, U256,
    },
    CallStackFrame, EvmContext, Inspector,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use auto_impl::auto_impl;
use core::{fmt, marker::PhantomData, ops::Range};

#[cfg(feature = "optimism")]
use crate::optimism;

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMImpl<'a, SPEC: Spec, DB: Database> {
    pub context: EvmContext<'a, DB>,
    pub inspector: Option<&'a mut dyn Inspector<DB>>,
    pub instruction_table: InstructionTables<'a, Self>,
    pub handler: Handler<DB>,
    _phantomdata: PhantomData<SPEC>,
}

impl<SPEC, DB> fmt::Debug for EVMImpl<'_, SPEC, DB>
where
    SPEC: Spec,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EVMImpl")
            .field("data", &self.context)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "optimism")]
impl<'a, SPEC: Spec, DB: Database> EVMImpl<'a, SPEC, DB> {
    /// If the transaction is not a deposit transaction, subtract the L1 data fee from the
    /// caller's balance directly after minting the requested amount of ETH.
    fn remove_l1_cost(
        is_deposit: bool,
        tx_caller: Address,
        l1_cost: U256,
        db: &mut DB,
        journal: &mut JournaledState,
    ) -> Result<(), EVMError<DB::Error>> {
        if is_deposit {
            return Ok(());
        }
        let acc = journal
            .load_account(tx_caller, db)
            .map_err(EVMError::Database)?
            .0;
        if l1_cost.gt(&acc.info.balance) {
            return Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: Box::new(l1_cost),
                    balance: Box::new(acc.info.balance),
                },
            ));
        }
        acc.info.balance = acc.info.balance.saturating_sub(l1_cost);
        Ok(())
    }

    /// If the transaction is a deposit with a `mint` value, add the mint value
    /// in wei to the caller's balance. This should be persisted to the database
    /// prior to the rest of execution.
    fn commit_mint_value(
        tx_caller: Address,
        tx_mint: Option<u128>,
        db: &mut DB,
        journal: &mut JournaledState,
    ) -> Result<(), EVMError<DB::Error>> {
        if let Some(mint) = tx_mint {
            journal
                .load_account(tx_caller, db)
                .map_err(EVMError::Database)?
                .0
                .info
                .balance += U256::from(mint);
            journal.checkpoint();
        }
        Ok(())
    }
}

impl<'a, SPEC: Spec + 'static, DB: Database> EVMImpl<'a, SPEC, DB> {
    pub fn new_with_spec(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: Option<&'a mut dyn Inspector<DB>>,
        precompiles: Precompiles,
    ) -> Self {
        let journaled_state =
            JournaledState::new(SPEC::SPEC_ID, precompiles.addresses().copied().collect());
        // If T is present it should be a generic T that modifies handler.
        let instruction_table = if inspector.is_some() {
            let instruction_table = make_boxed_instruction_table::<Self, SPEC, _>(
                make_instruction_table::<Self, SPEC>(),
                inspector_instruction,
            );
            InstructionTables::Boxed(Arc::new(instruction_table))
        } else {
            InstructionTables::Plain(Arc::new(make_instruction_table::<Self, SPEC>()))
        };
        #[cfg(feature = "optimism")]
        let mut handler = if env.cfg.optimism {
            Handler::optimism::<SPEC>()
        } else {
            Handler::mainnet::<SPEC>()
        };
        #[cfg(not(feature = "optimism"))]
        let mut handler = Handler::mainnet::<SPEC>();

        if env.cfg.is_beneficiary_reward_disabled() {
            // do nothing
            handler.reward_beneficiary = |_, _| Ok(());
        }

        Self {
            context: EvmContext {
                env,
                journaled_state,
                db,
                error: None,
                precompiles,
                #[cfg(feature = "optimism")]
                l1_block_info: None,
            },
            inspector,
            instruction_table,
            handler,
            _phantomdata: PhantomData {},
        }
    }

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
            SharedMemory::new_with_memory_limit(self.context.env.cfg.memory_limit);
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
                } => self.handle_sub_call(
                    inputs,
                    stack_frame,
                    return_memory_offset,
                    &mut shared_memory,
                ),
                InterpreterAction::Create { inputs } => self.handle_sub_create(inputs, stack_frame),
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    let child = call_stack.pop().unwrap();
                    let parent = call_stack.last_mut();

                    if let Some(result) =
                        self.handle_frame_return(child, parent, &mut shared_memory, result)
                    {
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

    fn handle_frame_return(
        &mut self,
        mut child_stack_frame: Box<CallStackFrame>,
        parent_stack_frame: Option<&mut Box<CallStackFrame>>,
        shared_memory: &mut SharedMemory,
        mut result: InterpreterResult,
    ) -> Option<InterpreterResult> {
        if let Some(inspector) = self.inspector.as_mut() {
            result = if child_stack_frame.is_create {
                let (result, address) = inspector.create_end(
                    &mut self.context,
                    result,
                    child_stack_frame.created_address,
                );
                child_stack_frame.created_address = address;
                result
            } else {
                inspector.call_end(&mut self.context, result)
            };
        }

        // break from loop if this is last CallStackFrame.
        let Some(parent_stack_frame) = parent_stack_frame else {
            let result = if child_stack_frame.is_create {
                self.context
                    .create_return::<SPEC>(result, child_stack_frame)
                    .0
            } else {
                self.context.call_return(result, child_stack_frame)
            };

            return Some(result);
        };

        if child_stack_frame.is_create {
            let (result, address) = self
                .context
                .create_return::<SPEC>(result, child_stack_frame);
            parent_stack_frame
                .interpreter
                .insert_create_output(result, Some(address))
        } else {
            let subcall_memory_return_offset =
                child_stack_frame.subcall_return_memory_range.clone();
            let result = self.context.call_return(result, child_stack_frame);

            parent_stack_frame.interpreter.insert_call_output(
                shared_memory,
                result,
                subcall_memory_return_offset,
            )
        }
        None
    }

    /// Handle Action for new sub create call, return None if there is no need
    /// to add new stack frame.
    #[inline]
    fn handle_sub_create(
        &mut self,
        mut inputs: Box<CreateInputs>,
        curent_stack_frame: &mut CallStackFrame,
    ) -> Option<Box<CallStackFrame>> {
        // Call inspector if it is some.
        if let Some(inspector) = self.inspector.as_mut() {
            if let Some((result, address)) = inspector.create(&mut self.context, &mut inputs) {
                curent_stack_frame
                    .interpreter
                    .insert_create_output(result, address);
                return None;
            }
        }

        match self.context.make_create_frame::<SPEC>(&inputs) {
            Ok(new_frame) => Some(new_frame),
            Err(mut result) => {
                let mut address = None;
                if let Some(inspector) = self.inspector.as_mut() {
                    let ret = inspector.create_end(
                        &mut self.context,
                        result,
                        curent_stack_frame.created_address,
                    );
                    result = ret.0;
                    address = ret.1;
                }
                // insert result of the failed creation of create CallStackFrame.
                curent_stack_frame
                    .interpreter
                    .insert_create_output(result, address);
                None
            }
        }
    }

    /// Handles action for new sub call, return None if there is no need to add
    /// new stack frame.
    #[inline]
    fn handle_sub_call(
        &mut self,
        mut inputs: Box<CallInputs>,
        curent_stake_frame: &mut CallStackFrame,
        return_memory_offset: Range<usize>,
        shared_memory: &mut SharedMemory,
    ) -> Option<Box<CallStackFrame>> {
        // Call inspector if it is some.
        if let Some(inspector) = self.inspector.as_mut() {
            if let Some((result, range)) = inspector.call(&mut self.context, &mut inputs) {
                curent_stake_frame
                    .interpreter
                    .insert_call_output(shared_memory, result, range);
                return None;
            }
        }
        match self
            .context
            .make_call_frame(&inputs, return_memory_offset.clone())
        {
            Ok(new_frame) => Some(new_frame),
            Err(mut result) => {
                //println!("Result returned right away: {:#?}", result);
                if let Some(inspector) = self.inspector.as_mut() {
                    result = inspector.call_end(&mut self.context, result);
                }
                curent_stake_frame.interpreter.insert_call_output(
                    shared_memory,
                    result,
                    return_memory_offset,
                );
                None
            }
        }
    }

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<DB::Error>> {
        let env = self.env();

        // Important: validate block before tx.
        env.validate_block_env::<SPEC>()?;
        env.validate_tx::<SPEC>()?;

        let initial_gas_spend = initial_tx_gas::<SPEC>(
            &env.tx.data,
            env.tx.transact_to.is_create(),
            &env.tx.access_list,
        );

        // Additional check to see if limit is big enough to cover initial gas.
        if initial_gas_spend > env.tx.gas_limit {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
        }

        // load acc
        let tx_caller = env.tx.caller;
        let (caller_account, _) = self
            .context
            .journaled_state
            .load_account(tx_caller, self.context.db)
            .map_err(EVMError::Database)?;

        self.context
            .env
            .validate_tx_against_state(caller_account)
            .map_err(Into::into)
    }

    /// Transact preverified transaction.
    pub fn transact_preverified_inner(&mut self) -> EVMResult<DB::Error> {
        let env = &self.context.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;

        // the L1-cost fee is only computed for Optimism non-deposit transactions.
        #[cfg(feature = "optimism")]
        let tx_l1_cost = if env.cfg.optimism && env.tx.optimism.source_hash.is_none() {
            let l1_block_info =
                optimism::L1BlockInfo::try_fetch(self.context.db).map_err(EVMError::Database)?;

            let Some(enveloped_tx) = &env.tx.optimism.enveloped_tx else {
                panic!("[OPTIMISM] Failed to load enveloped transaction.");
            };
            let tx_l1_cost = l1_block_info.calculate_tx_l1_cost::<SPEC>(enveloped_tx);

            // storage l1 block info for later use.
            self.context.l1_block_info = Some(l1_block_info);

            tx_l1_cost
        } else {
            U256::ZERO
        };

        let initial_gas_spend = initial_tx_gas::<SPEC>(
            &tx_data,
            env.tx.transact_to.is_create(),
            &env.tx.access_list,
        );

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if SPEC::enabled(SHANGHAI) {
            self.context
                .journaled_state
                .initial_account_load(self.context.env.block.coinbase, &[], self.context.db)
                .map_err(EVMError::Database)?;
        }

        self.context.load_access_list()?;

        // load acc
        let journal = &mut self.context.journaled_state;

        #[cfg(feature = "optimism")]
        if self.context.env.cfg.optimism {
            EVMImpl::<SPEC, DB>::commit_mint_value(
                tx_caller,
                self.context.env.tx.optimism.mint,
                self.context.db,
                journal,
            )?;

            let is_deposit = self.context.env.tx.optimism.source_hash.is_some();
            EVMImpl::<SPEC, DB>::remove_l1_cost(
                is_deposit,
                tx_caller,
                tx_l1_cost,
                self.context.db,
                journal,
            )?;
        }

        let (caller_account, _) = journal
            .load_account(tx_caller, self.context.db)
            .map_err(EVMError::Database)?;

        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            U256::from(tx_gas_limit).saturating_mul(self.context.env.effective_gas_price());

        // EIP-4844
        if SPEC::enabled(CANCUN) {
            let data_fee = self.context.env.calc_data_fee().expect("already checked");
            gas_cost = gas_cost.saturating_add(data_fee);
        }

        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

        // touch account so we know it is changed.
        caller_account.mark_touch();

        let transact_gas_limit = tx_gas_limit - initial_gas_spend;

        // call inner handling of call/create
        let first_stack_frame = match self.context.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                self.context.make_call_frame(
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
            TransactTo::Create(scheme) => self.context.make_create_frame::<SPEC>(&CreateInputs {
                caller: tx_caller,
                scheme,
                value: tx_value,
                init_code: tx_data,
                gas_limit: transact_gas_limit,
            }),
        };
        // Some only if it is create.
        let mut created_address = None;

        // start main loop if CallStackFrame is created correctly
        let interpreter_result = match first_stack_frame {
            Ok(first_stack_frame) => {
                created_address = first_stack_frame.created_address;
                let table = self.instruction_table.clone();
                match table {
                    InstructionTables::Plain(table) => self.run(&table, first_stack_frame),
                    InstructionTables::Boxed(table) => self.run(&table, first_stack_frame),
                }
            }
            Err(interpreter_result) => interpreter_result,
        };

        let handler = &self.handler;
        let data = &mut self.context;

        // handle output of call/create calls.
        let mut gas =
            handler.call_return(data.env, interpreter_result.result, interpreter_result.gas);

        // set refund. Refund amount depends on hardfork.
        gas.set_refund(handler.calculate_gas_refund(data.env, &gas) as i64);

        // Reimburse the caller
        handler.reimburse_caller(data, &gas)?;

        // Reward beneficiary
        handler.reward_beneficiary(data, &gas)?;

        // output of execution
        let output = match data.env.tx.transact_to {
            TransactTo::Call(_) => Output::Call(interpreter_result.output),
            TransactTo::Create(_) => Output::Create(interpreter_result.output, created_address),
        };

        // main return
        handler.main_return(data, interpreter_result.result, output, &gas)
    }
}

/// EVM transaction interface.
#[auto_impl(&mut, Box)]
pub trait Transact<DBError> {
    /// Run checks that could make transaction fail before call/create.
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DBError>>;

    /// Skip pre-verification steps and execute the transaction.
    fn transact_preverified(&mut self) -> EVMResult<DBError>;

    /// Execute transaction by running pre-verification steps and then transaction itself.
    fn transact(&mut self) -> EVMResult<DBError>;
}

impl<'a, SPEC: Spec + 'static, DB: Database> Transact<DB::Error> for EVMImpl<'a, SPEC, DB> {
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

impl<'a, SPEC: Spec + 'static, DB: Database> Host for EVMImpl<'a, SPEC, DB> {
    fn env(&mut self) -> &mut Env {
        self.context.env()
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.context.block_hash(number)
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.context.load_account(address)
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.context.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        self.context.code(address)
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.context.code_hash(address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.context.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.context.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.context.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.context.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.log(&mut self.context, &address, &topics, &data);
        }
        let log = Log {
            address,
            topics,
            data,
        };
        self.context.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        if let Some(inspector) = self.inspector.as_mut() {
            let acc = self.context.journaled_state.state.get(&address).unwrap();
            inspector.selfdestruct(address, target, acc.info.balance);
        }
        self.context
            .journaled_state
            .selfdestruct(address, target, self.context.db)
            .map_err(|e| self.context.error = Some(e))
            .ok()
    }
}

/// Creates new EVM instance with erased types.
pub fn new_evm<'a, DB: Database>(
    env: &'a mut Env,
    db: &'a mut DB,
    insp: Option<&'a mut dyn Inspector<DB>>,
) -> Box<dyn Transact<DB::Error> + 'a> {
    macro_rules! create_evm {
        ($spec:ident) => {
            Box::new(EVMImpl::<'a, $spec, DB>::new_with_spec(
                db,
                env,
                insp,
                Precompiles::new(revm_precompile::SpecId::from_spec_id($spec::SPEC_ID)).clone(),
            ))
        };
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
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::commit_mint_value(
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
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::commit_mint_value(
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
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
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
        assert!(EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
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
            EVMImpl::<BedrockSpec, InMemoryDB>::remove_l1_cost(
                false,
                caller,
                U256::from(101),
                &mut db,
                &mut journal
            ),
            Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: Box::new(U256::from(101)),
                    balance: Box::new(U256::from(100)),
                },
            ))
        );
    }
}
