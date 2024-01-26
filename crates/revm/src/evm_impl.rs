use crate::{
    db::Database,
    handler::Handler,
    inspector_instruction,
    interpreter::{
        gas,
        gas::initial_tx_gas,
        opcode::{make_boxed_instruction_table, make_instruction_table, InstructionTables},
        return_ok, CallContext, CallInputs, CallScheme, Contract, CreateInputs, Gas, Host,
        InstructionResult, Interpreter, SelfDestructResult, SharedMemory, Transfer, MAX_CODE_SIZE,
    },
    journaled_state::{JournalCheckpoint, JournaledState},
    precompile::{self, Precompile, Precompiles},
    primitives::{
        keccak256, Address, Bytecode, Bytes, EVMError, EVMResult, Env, InvalidTransaction, Log,
        Output, Spec, SpecId::*, TransactTo, B256, U256,
    },
    EVMData, Inspector,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use auto_impl::auto_impl;
use core::{fmt, marker::PhantomData};
use fluentbase_rwasm::rwasm::{Compiler, CompilerConfig, CompilerError, FuncOrExport};
use fluentbase_types::{Account, AccountDb, STATE_DEPLOY, STATE_MAIN};

#[cfg(feature = "optimism")]
use crate::optimism;

use fluentbase_sdk::evm::{ContractInput, ContractOutput};

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database> {
    pub data: EVMData<'a, DB>,
    pub inspector: Option<&'a mut dyn Inspector<DB>>,
    pub instruction_table: InstructionTables<'a, Self>,
    pub handler: Handler<DB>,
    _phantomdata: PhantomData<GSPEC>,
}

impl<GSPEC, DB> fmt::Debug for EVMImpl<'_, GSPEC, DB>
where
    GSPEC: Spec,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EVMImpl")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

struct PreparedCreate {
    gas: Gas,
    created_address: Address,
    checkpoint: JournalCheckpoint,
    contract: Box<Contract>,
}

struct CreateResult {
    result: InstructionResult,
    created_address: Option<Address>,
    gas: Gas,
    return_value: Bytes,
}

struct PreparedCall {
    gas: Gas,
    checkpoint: JournalCheckpoint,
    contract: Box<Contract>,
}

struct CallResult {
    result: InstructionResult,
    gas: Gas,
    return_value: Bytes,
}

enum BytecodeType {
    Rwasm,
    Evm,
    Wasm,
}

impl BytecodeType {
    pub(crate) fn from_slice(input: &[u8]) -> Self {
        if input.len() >= 4 && input[0..4] == [0x00, 0x61, 0x73, 0x6d] {
            Self::Wasm
        } else if input.len() >= 1 && input[0] == 0xef {
            Self::Rwasm
        } else {
            Self::Evm
        }
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

#[cfg(feature = "optimism")]
impl<'a, GSPEC: Spec, DB: Database> EVMImpl<'a, GSPEC, DB> {
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
            let u64_cost = if U256::from(u64::MAX).lt(&l1_cost) {
                u64::MAX
            } else {
                l1_cost.as_limbs()[0]
            };
            return Err(EVMError::Transaction(
                InvalidTransaction::LackOfFundForMaxFee {
                    fee: u64_cost,
                    balance: acc.info.balance,
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

impl<'a, GSPEC: Spec + 'static, DB: Database> Transact<DB::Error> for EVMImpl<'a, GSPEC, DB> {
    #[inline]
    fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        self.preverify_transaction_inner()
    }

    #[inline]
    fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
        let output = self.transact_preverified_inner();
        self.handler.end(&mut self.data, output)
    }

    #[inline]
    fn transact(&mut self) -> EVMResult<DB::Error> {
        let output = self
            .preverify_transaction_inner()
            .and_then(|()| self.transact_preverified_inner());
        self.handler.end(&mut self.data, output)
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> EVMImpl<'a, GSPEC, DB> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: Option<&'a mut dyn Inspector<DB>>,
        precompiles: Precompiles,
    ) -> Self {
        let journaled_state = JournaledState::new(
            GSPEC::SPEC_ID,
            precompiles
                .addresses()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
        );
        let instruction_table = if inspector.is_some() {
            let instruction_table = make_boxed_instruction_table::<Self, GSPEC, _>(
                make_instruction_table::<Self, GSPEC>(),
                inspector_instruction,
            );
            InstructionTables::Boxed(Arc::new(instruction_table))
        } else {
            InstructionTables::Plain(Arc::new(make_instruction_table::<Self, GSPEC>()))
        };
        #[cfg(feature = "optimism")]
        let handler = if env.cfg.optimism {
            Handler::optimism::<GSPEC>()
        } else {
            Handler::mainnet::<GSPEC>()
        };
        #[cfg(not(feature = "optimism"))]
        let handler = Handler::mainnet::<GSPEC>();

        Self {
            data: EVMData {
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

    /// Pre verify transaction.
    pub fn preverify_transaction_inner(&mut self) -> Result<(), EVMError<DB::Error>> {
        let env = self.env();

        // Important: validate block before tx.
        env.validate_block_env::<GSPEC>()?;
        env.validate_tx::<GSPEC>()?;

        let initial_gas_spend = initial_tx_gas::<GSPEC>(
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
            .data
            .journaled_state
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        self.data
            .env
            .validate_tx_against_state(caller_account)
            .map_err(Into::into)
    }

    /// Transact preverified transaction.
    pub fn transact_preverified_inner(&mut self) -> EVMResult<DB::Error> {
        let env = &self.data.env;
        let tx_caller = env.tx.caller;
        let tx_value = env.tx.value;
        let tx_data = env.tx.data.clone();
        let tx_gas_limit = env.tx.gas_limit;

        // the L1-cost fee is only computed for Optimism non-deposit transactions.
        #[cfg(feature = "optimism")]
        let tx_l1_cost = if env.cfg.optimism && env.tx.optimism.source_hash.is_none() {
            let l1_block_info =
                optimism::L1BlockInfo::try_fetch(self.data.db).map_err(EVMError::Database)?;

            let Some(enveloped_tx) = &env.tx.optimism.enveloped_tx else {
                panic!("[OPTIMISM] Failed to load enveloped transaction.");
            };
            let tx_l1_cost = l1_block_info.calculate_tx_l1_cost::<GSPEC>(enveloped_tx);

            // storage l1 block info for later use.
            self.data.l1_block_info = Some(l1_block_info);

            tx_l1_cost
        } else {
            U256::ZERO
        };

        let initial_gas_spend = initial_tx_gas::<GSPEC>(
            &tx_data,
            env.tx.transact_to.is_create(),
            &env.tx.access_list,
        );

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if GSPEC::enabled(SHANGHAI) {
            self.data
                .journaled_state
                .initial_account_load(self.data.env.block.coinbase, &[], self.data.db)
                .map_err(EVMError::Database)?;
        }

        self.data.load_access_list()?;

        // load acc
        let journal = &mut self.data.journaled_state;

        #[cfg(feature = "optimism")]
        if self.data.env.cfg.optimism {
            EVMImpl::<GSPEC, DB>::commit_mint_value(
                tx_caller,
                self.data.env.tx.optimism.mint,
                self.data.db,
                journal,
            )?;

            let is_deposit = self.data.env.tx.optimism.source_hash.is_some();
            EVMImpl::<GSPEC, DB>::remove_l1_cost(
                is_deposit,
                tx_caller,
                tx_l1_cost,
                self.data.db,
                journal,
            )?;
        }

        let (caller_account, _) = journal
            .load_account(tx_caller, self.data.db)
            .map_err(EVMError::Database)?;

        // Subtract gas costs from the caller's account.
        // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
        let mut gas_cost =
            U256::from(tx_gas_limit).saturating_mul(self.data.env.effective_gas_price());

        // EIP-4844
        if GSPEC::enabled(CANCUN) {
            let data_fee = self.data.env.calc_data_fee().expect("already checked");
            gas_cost = gas_cost.saturating_add(data_fee);
        }

        caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

        // touch account so we know it is changed.
        caller_account.mark_touch();

        let transact_gas_limit = tx_gas_limit - initial_gas_spend;

        #[cfg(feature = "memory_limit")]
        let mut shared_memory = SharedMemory::new_with_memory_limit(self.data.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        // call inner handling of call/create
        let (call_result, ret_gas, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                // Nonce is already checked
                caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);

                let (exit, gas, bytes) = self.call(
                    &mut CallInputs {
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
                    &mut shared_memory,
                );
                (exit, gas, Output::Call(bytes))
            }
            TransactTo::Create(scheme) => {
                let (exit, address, ret_gas, bytes) = self.create(
                    &mut CreateInputs {
                        caller: tx_caller,
                        scheme,
                        value: tx_value,
                        init_code: tx_data,
                        gas_limit: transact_gas_limit,
                    },
                    &mut shared_memory,
                );
                (exit, ret_gas, Output::Create(bytes, address))
            }
        };

        let handler = &self.handler;
        let data = &mut self.data;

        // handle output of call/create calls.
        let mut gas = handler.call_return(data.env, call_result, ret_gas);

        // set refund. Refund amount depends on hardfork.
        gas.set_refund(handler.calculate_gas_refund(data.env, &gas) as i64);

        // Reimburse the caller
        handler.reimburse_caller(data, &gas)?;

        // Reward beneficiary
        if !data.env.cfg.is_beneficiary_reward_disabled() {
            handler.reward_beneficiary(data, &gas)?;
        }

        // main return
        handler.main_return(data, call_result, output, &gas)
    }

    #[inline(never)]
    fn prepare_create(&mut self, inputs: &CreateInputs) -> Result<PreparedCreate, CreateResult> {
        let gas = Gas::new(inputs.gas_limit);

        // Check depth of calls
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CreateResult {
                result: InstructionResult::CallTooDeep,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.balance(inputs.caller) else {
            return Err(CreateResult {
                result: InstructionResult::FatalExternalError,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        };

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return Err(CreateResult {
                result: InstructionResult::OutOfFund,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.data.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return Err(CreateResult {
                result: InstructionResult::Return,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Create address
        let code_hash = keccak256(&inputs.init_code);
        let created_address = inputs.created_address_with_hash(old_nonce, &code_hash);

        // Load account so it needs to be marked as warm for access list.
        if self
            .data
            .journaled_state
            .load_account(created_address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .is_err()
        {
            return Err(CreateResult {
                result: InstructionResult::FatalExternalError,
                created_address: None,
                gas,
                return_value: Bytes::new(),
            });
        }

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self
            .data
            .journaled_state
            .create_account_checkpoint::<GSPEC>(inputs.caller, created_address, inputs.value)
        {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return Err(CreateResult {
                    result: e,
                    created_address: None,
                    gas,
                    return_value: Bytes::new(),
                });
            }
        };

        let bytecode = match BytecodeType::from_slice(inputs.init_code.as_ref()) {
            BytecodeType::Wasm => Self::translate_wasm_to_rwasm(&inputs.init_code, "deploy")
                .map_err(|_| {
                    return CreateResult {
                        result: InstructionResult::Revert,
                        created_address: None,
                        gas,
                        return_value: Bytes::new(),
                    };
                })?,
            _ => inputs.init_code.clone(),
        };

        let contract = Box::new(Contract::new(
            Bytes::new(),
            Bytecode::new_raw(bytecode),
            code_hash,
            created_address,
            inputs.caller,
            inputs.value,
        ));

        Ok(PreparedCreate {
            gas,
            created_address,
            checkpoint,
            contract,
        })
    }

    fn translate_wasm_to_rwasm(
        input: &Bytes,
        func_name: &'static str,
    ) -> Result<Bytes, CompilerError> {
        use fluentbase_runtime::Runtime;
        let import_linker = Runtime::<()>::new_linker();
        let mut compiler = Compiler::new_with_linker(
            input.as_ref(),
            CompilerConfig::default(),
            Some(&import_linker),
        )?;
        compiler.translate(FuncOrExport::Export(func_name))?;
        let output = compiler.finalize()?;
        Ok(Bytes::from(output))
    }

    /// EVM create opcode for both initial crate and CREATE and CREATE2 opcodes.
    fn create_inner(
        &mut self,
        inputs: &CreateInputs,
        shared_memory: &mut SharedMemory,
    ) -> CreateResult {
        // Prepare crate.
        let prepared_create = match self.prepare_create(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        // Create new interpreter and execute init code
        let (exit_reason, bytes, mut gas) = self.run_interpreter(
            prepared_create.contract,
            prepared_create.gas.limit(),
            false,
            STATE_DEPLOY,
            shared_memory,
        );

        // Host error if present on execution
        match exit_reason {
            return_ok!() => {
                let mut bytes = match BytecodeType::from_slice(inputs.init_code.as_ref()) {
                    BytecodeType::Wasm => match Self::translate_wasm_to_rwasm(&bytes, "main") {
                        Err(_) => {
                            self.data
                                .journaled_state
                                .checkpoint_revert(prepared_create.checkpoint);
                            return CreateResult {
                                result: InstructionResult::CreateContractStartingWithEF,
                                created_address: Some(prepared_create.created_address),
                                gas,
                                return_value: bytes,
                            };
                        }
                        Ok(result) => result,
                    },
                    _ => bytes,
                };

                // if ok, check contract creation limit and calculate gas deduction on output len.
                //
                // EIP-3541: Reject new contract code starting with the 0xEF byte
                // if GSPEC::enabled(LONDON) && !bytes.is_empty() && bytes.first() == Some(&0xEF) {
                //     self.data
                //         .journaled_state
                //         .checkpoint_revert(prepared_create.checkpoint);
                //     return CreateResult {
                //         result: InstructionResult::CreateContractStartingWithEF,
                //         created_address: Some(prepared_create.created_address),
                //         gas,
                //         return_value: bytes,
                //     };
                // }

                // EIP-170: Contract code size limit
                // By default limit is 0x6000 (~25kb)
                if GSPEC::enabled(SPURIOUS_DRAGON)
                    && bytes.len()
                        > self
                            .data
                            .env
                            .cfg
                            .limit_contract_code_size
                            .unwrap_or(MAX_CODE_SIZE)
                {
                    self.data
                        .journaled_state
                        .checkpoint_revert(prepared_create.checkpoint);
                    return CreateResult {
                        result: InstructionResult::CreateContractSizeLimit,
                        created_address: Some(prepared_create.created_address),
                        gas,
                        return_value: bytes,
                    };
                }
                if crate::USE_GAS {
                    let gas_for_code = bytes.len() as u64 * gas::CODEDEPOSIT;
                    if !gas.record_cost(gas_for_code) {
                        // record code deposit gas cost and check if we are out of gas.
                        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
                        // final gas fee for adding the contract code to the state, the contract
                        //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
                        if GSPEC::enabled(HOMESTEAD) {
                            self.data
                                .journaled_state
                                .checkpoint_revert(prepared_create.checkpoint);
                            return CreateResult {
                                result: InstructionResult::OutOfGas,
                                created_address: Some(prepared_create.created_address),
                                gas,
                                return_value: bytes,
                            };
                        } else {
                            bytes = Bytes::new();
                        }
                    }
                }
                // if we have enough gas
                self.data.journaled_state.checkpoint_commit();
                self.data.journaled_state.set_code(
                    prepared_create.created_address,
                    Bytecode::new_raw(bytes.clone()),
                );
                CreateResult {
                    result: InstructionResult::Return,
                    created_address: Some(prepared_create.created_address),
                    gas,
                    return_value: bytes,
                }
            }
            _ => {
                self.data
                    .journaled_state
                    .checkpoint_revert(prepared_create.checkpoint);
                CreateResult {
                    result: exit_reason,
                    created_address: Some(prepared_create.created_address),
                    gas,
                    return_value: bytes,
                }
            }
        }
    }

    pub fn run_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
        state: u32,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Bytes, Gas) {
        match BytecodeType::from_slice(contract.bytecode.bytecode()) {
            BytecodeType::Rwasm => {
                self.run_rwasm_interpreter(contract, gas_limit, is_static, state, shared_memory)
            }
            BytecodeType::Evm => {
                self.run_evm_interpreter(contract, gas_limit, is_static, shared_memory)
            }
            BytecodeType::Wasm => {
                panic!("not supported wasm runtime")
            }
        }
    }

    #[cfg(feature = "runtime")]
    pub fn run_rwasm_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        _is_static: bool,
        state: u32,
        _shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Bytes, Gas) {
        use fluentbase_codec::{BufferDecoder, Encoder};
        use fluentbase_runtime::{Runtime, RuntimeContext};
        let contract_address = contract.address.clone();
        let bytecode = Bytes::copy_from_slice(contract.bytecode.original_bytecode_slice());
        let hash_keccak256 = contract.bytecode.hash_slow();
        let execution_result = {
            let import_linker = Runtime::<'_, EVMData<'a, DB>>::new_linker();
            let contract_input = ContractInput {
                env_chain_id: self.data.env.cfg.chain_id,
                contract_address: contract.address,
                contract_caller: contract.caller,
                contract_bytecode: bytecode.clone(),
                contract_code_size: bytecode.len() as u32,
                contract_code_hash: hash_keccak256,
                contract_input_size: contract.input.len() as u32,
                contract_input: contract.input,
                contract_value: contract.value,
                block_hash: Default::default(),
                block_coinbase: self.data.env.block.coinbase,
                block_timestamp: self.data.env.block.timestamp.as_limbs()[0],
                block_number: self.data.env.block.number.as_limbs()[0],
                block_difficulty: self.data.env.block.difficulty.as_limbs()[0],
                block_gas_limit: self.data.env.block.gas_limit.as_limbs()[0],
                block_base_fee: self.data.env.block.basefee,
                tx_gas_price: self.data.env.tx.gas_price,
                tx_gas_priority_fee: self.data.env.tx.gas_priority_fee,
                tx_caller: self.data.env.tx.caller,
                // tx_blob_hashes: self.data.env.tx.blob_hashes.clone(),
                // tx_blob_gas_price: 0,
            };
            let raw_input = contract_input.encode_to_vec(0);
            let ctx = RuntimeContext::<'_, EVMData<'a, DB>>::new(bytecode.as_ref())
                .with_context(&mut self.data)
                .with_input(raw_input)
                .with_state(state)
                .with_fuel_limit(gas_limit as u32);
            let runtime = Runtime::<'_, EVMData<'a, DB>>::new_uninit(ctx, &import_linker);
            if runtime.is_err() {
                return (InstructionResult::Revert, Bytes::new(), Gas::new(gas_limit));
            }
            let mut runtime = runtime.unwrap();
            runtime.register_bindings();
            if let Err(_) = runtime.instantiate() {
                return (InstructionResult::Revert, Bytes::new(), Gas::new(gas_limit));
            }
            let result = runtime.call();
            if result.is_err() {
                return (InstructionResult::Revert, Bytes::new(), Gas::new(gas_limit));
            }
            result.unwrap()
        };
        let contract_output = {
            let mut buffer_decoder =
                BufferDecoder::new(execution_result.data().output().as_slice());
            let mut contract_output = ContractOutput::default();
            ContractOutput::decode_body(&mut buffer_decoder, 0, &mut contract_output);
            contract_output
        };
        let exit_code = execution_result.data().exit_code();
        if exit_code != 0 {
            return (InstructionResult::Revert, Bytes::new(), Gas::new(gas_limit));
        }
        let return_data = contract_output.return_data;
        for log in contract_output.logs.iter() {
            let topics = if let Some(topics) = log.topic0 {
                topics.to_vec()
            } else if let Some(topics) = log.topic1 {
                topics.to_vec()
            } else if let Some(topics) = log.topic2 {
                topics.to_vec()
            } else if let Some(topics) = log.topic3 {
                topics.to_vec()
            } else if let Some(topics) = log.topic4 {
                topics.to_vec()
            } else {
                Default::default()
            };
            self.log(contract_address.clone(), topics, log.data.clone())
        }
        (InstructionResult::Stop, return_data, Gas::new(gas_limit))
    }

    /// Create a Interpreter and run it.
    /// Returns the exit reason, return value and gas from interpreter
    #[cfg(not(feature = "runtime"))]
    pub fn run_rwasm_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
        state: u32,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Bytes, Gas) {
        let bytecode = contract.bytecode.original_bytecode_slice();
        let mut output = vec![0u8; 1024];
        let input = &contract.input;
        let err = SDK::rwasm_transact(
            bytecode,
            input.as_ref(),
            output.as_mut_slice(),
            state,
            gas_limit as u32,
        );
        if err < 0 {
            return (InstructionResult::Revert, Bytes::new(), Gas::new(gas_limit));
        }
        (
            InstructionResult::Stop,
            Bytes::copy_from_slice(&output),
            Gas::new(gas_limit),
        )
    }

    /// Create a Interpreter and run it.
    /// Returns the exit reason, return value and gas from interpreter
    pub fn run_evm_interpreter(
        &mut self,
        contract: Box<Contract>,
        gas_limit: u64,
        is_static: bool,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Bytes, Gas) {
        let mut interpreter = Box::new(Interpreter::new(
            contract,
            gas_limit,
            is_static,
            shared_memory,
        ));

        interpreter.shared_memory.new_context();

        if let Some(inspector) = self.inspector.as_mut() {
            inspector.initialize_interp(&mut interpreter, &mut self.data);
        }

        let exit_reason = match &mut self.instruction_table {
            InstructionTables::Plain(table) => interpreter.run::<_, Self>(&table.clone(), self),
            InstructionTables::Boxed(table) => interpreter.run::<_, Self>(&table.clone(), self),
        };

        let (return_value, gas) = (interpreter.return_value(), *interpreter.gas());

        interpreter.shared_memory.free_context();

        (exit_reason, return_value, gas)
    }

    /// Call precompile contract
    fn call_precompile(
        &mut self,
        precompile: Precompile,
        inputs: &CallInputs,
        mut gas: Gas,
    ) -> CallResult {
        let input_data = &inputs.input;

        let out = match precompile {
            Precompile::Standard(fun) => fun(input_data, gas.limit()),
            Precompile::Env(fun) => fun(input_data, gas.limit(), self.env()),
        };

        match out {
            Ok((gas_used, data)) => {
                if !crate::USE_GAS || gas.record_cost(gas_used) {
                    CallResult {
                        result: InstructionResult::Return,
                        gas,
                        return_value: Bytes::from(data),
                    }
                } else {
                    CallResult {
                        result: InstructionResult::PrecompileOOG,
                        gas,
                        return_value: Bytes::new(),
                    }
                }
            }
            Err(e) => {
                let result = if precompile::Error::OutOfGas == e {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
                CallResult {
                    result,
                    gas,
                    return_value: Bytes::new(),
                }
            }
        }
    }

    #[inline(never)]
    fn prepare_call(&mut self, inputs: &CallInputs) -> Result<PreparedCall, CallResult> {
        let gas = Gas::new(inputs.gas_limit);
        let account = match self
            .data
            .journaled_state
            .load_code(inputs.contract, self.data.db)
        {
            Ok((account, _)) => account,
            Err(e) => {
                self.data.error = Some(e);
                return Err(CallResult {
                    result: InstructionResult::FatalExternalError,
                    gas,
                    return_value: Bytes::new(),
                });
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return Err(CallResult {
                result: InstructionResult::CallTooDeep,
                gas,
                return_value: Bytes::new(),
            });
        }

        // Create subroutine checkpoint
        let checkpoint = self.data.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.load_account(inputs.context.address);
            self.data.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Err(e) = self.data.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            self.data.db,
        ) {
            self.data.journaled_state.checkpoint_revert(checkpoint);
            return Err(CallResult {
                result: e,
                gas,
                return_value: Bytes::new(),
            });
        }

        let contract = Box::new(Contract::new_with_context(
            inputs.input.clone(),
            bytecode,
            code_hash,
            &inputs.context,
        ));

        Ok(PreparedCall {
            gas,
            checkpoint,
            contract,
        })
    }

    /// Main contract call of the EVM.
    fn call_inner(&mut self, inputs: &CallInputs, shared_memory: &mut SharedMemory) -> CallResult {
        // Prepare call
        let prepared_call = match self.prepare_call(inputs) {
            Ok(o) => o,
            Err(e) => return e,
        };

        let ret = if let Some(precompile) = self.data.precompiles.get(&inputs.contract) {
            self.call_precompile(precompile, inputs, prepared_call.gas)
        } else if !prepared_call.contract.bytecode.is_empty() {
            // Create interpreter and execute subcall
            let (exit_reason, bytes, gas) = self.run_interpreter(
                prepared_call.contract,
                prepared_call.gas.limit(),
                inputs.is_static,
                STATE_MAIN,
                shared_memory,
            );
            CallResult {
                result: exit_reason,
                gas,
                return_value: bytes,
            }
        } else {
            CallResult {
                result: InstructionResult::Stop,
                gas: prepared_call.gas,
                return_value: Bytes::new(),
            }
        };

        // revert changes or not.
        if matches!(ret.result, return_ok!()) {
            self.data.journaled_state.checkpoint_commit();
        } else {
            self.data
                .journaled_state
                .checkpoint_revert(prepared_call.checkpoint);
        }

        ret
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> AccountDb for EVMImpl<'a, GSPEC, DB> {
    fn get_account(&mut self, address: &Address) -> Option<Account> {
        self.data.account(*address).map(|acc| Account {
            balance: acc.balance,
            nonce: acc.nonce,
            code_hash: acc.code_hash,
            code: acc.code.map(|code| code.bytecode),
        })
    }

    fn update_account(&mut self, _address: &Address, _account: &Account) {
        todo!()
    }

    fn get_storage(&mut self, address: &Address, index: &U256) -> Option<U256> {
        self.data.sload(*address, *index).map(|val| val.0)
    }

    fn update_storage(&mut self, address: &Address, index: &U256, value: &U256) {
        self.data.sstore(*address, *index, *value);
    }
}

impl<'a, GSPEC: Spec + 'static, DB: Database> Host for EVMImpl<'a, GSPEC, DB> {
    fn env(&mut self) -> &mut Env {
        self.data.env()
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.data.block_hash(number)
    }

    fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.data.load_account(address)
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.data.balance(address)
    }

    fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        self.data.code(address)
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.data.code_hash(address)
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.data.sload(address, index)
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.data.sstore(address, index, value)
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.data.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.data.tstore(address, index, value)
    }

    fn log(&mut self, address: Address, topics: Vec<B256>, data: Bytes) {
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.log(&mut self.data, &address, &topics, &data);
        }
        let log = Log {
            address,
            topics,
            data,
        };
        self.data.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        if let Some(inspector) = self.inspector.as_mut() {
            let acc = self.data.journaled_state.state.get(&address).unwrap();
            inspector.selfdestruct(address, target, acc.info.balance);
        }
        self.data
            .journaled_state
            .selfdestruct(address, target, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn create(
        &mut self,
        inputs: &mut CreateInputs,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // Call inspector
        if let Some(inspector) = self.inspector.as_mut() {
            let (ret, address, gas, out) = inspector.create(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return inspector.create_end(&mut self.data, inputs, ret, address, gas, out);
            }
        }
        let ret = self.create_inner(inputs, shared_memory);
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.create_end(
                &mut self.data,
                inputs,
                ret.result,
                ret.created_address,
                ret.gas,
                ret.return_value,
            )
        } else {
            (ret.result, ret.created_address, ret.gas, ret.return_value)
        }
    }

    fn call(
        &mut self,
        inputs: &mut CallInputs,
        shared_memory: &mut SharedMemory,
    ) -> (InstructionResult, Gas, Bytes) {
        if let Some(inspector) = self.inspector.as_mut() {
            let (ret, gas, out) = inspector.call(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return inspector.call_end(&mut self.data, inputs, gas, ret, out);
            }
        }
        let ret = self.call_inner(inputs, shared_memory);
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.call_end(
                &mut self.data,
                inputs,
                ret.gas,
                ret.result,
                ret.return_value,
            )
        } else {
            (ret.result, ret.gas, ret.return_value)
        }
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
                    fee: 101u64,
                    balance: U256::from(100),
                },
            ))
        );
    }
}
