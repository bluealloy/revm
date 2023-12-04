use crate::{
    db::Database,
    interpreter::{
        analysis::to_analysed, gas, return_ok, CallInputs, Contract, CreateInputs, Gas,
        InstructionResult, Interpreter, InterpreterResult, MAX_CODE_SIZE,
    },
    journaled_state::JournaledState,
    precompile::{Precompile, Precompiles},
    primitives::{
        keccak256, Address, AnalysisKind, Bytecode, Bytes, EVMError, Env, Spec, SpecId::*, B256,
        U256,
    },
    CallStackFrame, CALL_STACK_LIMIT,
};
use alloc::boxed::Box;
use core::ops::Range;

/// EVM Data contains all the data that EVM needs to execute.
#[derive(Debug)]
pub struct EvmContext<'a, DB: Database> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: &'a mut Env,
    /// EVM State with journaling support.
    pub journaled_state: JournaledState,
    /// Database to load data from.
    pub db: &'a mut DB,
    /// Error that happened during execution.
    pub error: Option<DB::Error>,
    /// Precompiles that are available for evm.
    pub precompiles: Precompiles,
    /// Used as temporary value holder to store L1 block info.
    #[cfg(feature = "optimism")]
    pub l1_block_info: Option<crate::optimism::L1BlockInfo>,
}

impl<'a, DB: Database> EvmContext<'a, DB> {
    /// Load access list for berlin hard fork.
    ///
    /// Loading of accounts/storages is needed to make them warm.
    #[inline]
    pub fn load_access_list(&mut self) -> Result<(), EVMError<DB::Error>> {
        for (address, slots) in self.env.tx.access_list.iter() {
            self.journaled_state
                .initial_account_load(*address, slots, self.db)
                .map_err(EVMError::Database)?;
        }
        Ok(())
    }

    /// Return environment.
    pub fn env(&mut self) -> &mut Env {
        self.env
    }

    /// Fetch block hash from database.
    pub fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.db
            .block_hash(number)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Load account and return flags (is_cold, exists)
    pub fn load_account(&mut self, address: Address) -> Option<(bool, bool)> {
        self.journaled_state
            .load_account_exist(address, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Return account balance and is_cold flag.
    pub fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.journaled_state
            .load_account(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    /// Return account code and if address is cold loaded.
    pub fn code(&mut self, address: Address) -> Option<(Bytecode, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        Some((acc.info.code.clone().unwrap(), is_cold))
    }

    /// Get code hash of address.
    pub fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        let (acc, is_cold) = self
            .journaled_state
            .load_code(address, &mut self.db)
            .map_err(|e| self.error = Some(e))
            .ok()?;
        if acc.is_empty() {
            return Some((B256::ZERO, is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    /// Load storage slot, if storage is not present inside the account then it will be loaded from database.
    pub fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        // account is always warm. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.journaled_state
            .sload(address, index, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Storage change of storage slot, before storing `sload` will be called for that slot.
    pub fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.journaled_state
            .sstore(address, index, value, self.db)
            .map_err(|e| self.error = Some(e))
            .ok()
    }

    /// Returns transient storage value.
    pub fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journaled_state.tload(address, index)
    }

    /// Stores transient storage value.
    pub fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journaled_state.tstore(address, index, value)
    }

    /// Make create frame.
    pub fn make_create_frame<SPEC: Spec>(
        &mut self,
        inputs: &CreateInputs,
    ) -> Result<Box<CallStackFrame>, InterpreterResult> {
        // Prepare crate.
        let gas = Gas::new(inputs.gas_limit);

        let return_error = |e| {
            Err(InterpreterResult {
                result: e,
                gas,
                output: Bytes::new(),
            })
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let Some((caller_balance, _)) = self.balance(inputs.caller) else {
            return return_error(InstructionResult::FatalExternalError);
        };

        // Check if caller has enough balance to send to the created contract.
        if caller_balance < inputs.value {
            return return_error(InstructionResult::OutOfFund);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        let code_hash = keccak256(&inputs.init_code);
        let created_address = inputs.created_address_with_hash(old_nonce, &code_hash);

        // Load account so it needs to be marked as warm for access list.
        if self
            .journaled_state
            .load_account(created_address, self.db)
            .map_err(|e| self.error = Some(e))
            .is_err()
        {
            return return_error(InstructionResult::FatalExternalError);
        }

        // create account, transfer funds and make the journal checkpoint.
        let checkpoint = match self.journaled_state.create_account_checkpoint::<SPEC>(
            inputs.caller,
            created_address,
            inputs.value,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => {
                return return_error(e);
            }
        };

        let bytecode = Bytecode::new_raw(inputs.init_code.clone());

        let contract = Box::new(Contract::new(
            Bytes::new(),
            bytecode,
            code_hash,
            created_address,
            inputs.caller,
            inputs.value,
        ));

        Ok(Box::new(CallStackFrame {
            is_create: true,
            checkpoint,
            created_address: Some(created_address),
            subcall_return_memory_range: 0..0,
            interpreter: Interpreter::new(contract, gas.limit(), false),
        }))
    }

    /// Make call frame
    pub fn make_call_frame(
        &mut self,
        inputs: &CallInputs,
        return_memory_offset: Range<usize>,
    ) -> Result<Box<CallStackFrame>, InterpreterResult> {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            Err(InterpreterResult {
                result: instruction_result,
                gas,
                output: Bytes::new(),
            })
        };

        // Check depth
        if self.journaled_state.depth() > CALL_STACK_LIMIT {
            return return_result(InstructionResult::CallTooDeep);
        }

        let account = match self.journaled_state.load_code(inputs.contract, self.db) {
            Ok((account, _)) => account,
            Err(e) => {
                self.error = Some(e);
                return return_result(InstructionResult::FatalExternalError);
            }
        };
        let code_hash = account.info.code_hash();
        let bytecode = account.info.code.clone().unwrap_or_default();

        // Create subroutine checkpoint
        let checkpoint = self.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.load_account(inputs.context.address);
            self.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Err(e) = self.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            self.db,
        ) {
            //println!("transfer error");
            self.journaled_state.checkpoint_revert(checkpoint);
            return return_result(e);
        }

        if let Some(precompile) = self.precompiles.get(&inputs.contract) {
            //println!("Call precompile");
            let result = self.call_precompile(precompile, inputs, gas);
            if matches!(result.result, return_ok!()) {
                self.journaled_state.checkpoint_commit();
            } else {
                self.journaled_state.checkpoint_revert(checkpoint);
            }
            Err(result)
        } else if !bytecode.is_empty() {
            let contract = Box::new(Contract::new_with_context(
                inputs.input.clone(),
                bytecode,
                code_hash,
                &inputs.context,
            ));
            // Create interpreter and execute subcall and push new CallStackFrame.
            Ok(Box::new(CallStackFrame {
                is_create: false,
                checkpoint,
                created_address: None,
                subcall_return_memory_range: return_memory_offset,
                interpreter: Interpreter::new(contract, gas.limit(), inputs.is_static),
            }))
        } else {
            self.journaled_state.checkpoint_commit();
            return_result(InstructionResult::Stop)
        }
    }

    /// Call precompile contract
    fn call_precompile(
        &mut self,
        precompile: Precompile,
        inputs: &CallInputs,
        gas: Gas,
    ) -> InterpreterResult {
        let input_data = &inputs.input;

        let out = match precompile {
            Precompile::Standard(fun) => fun(input_data, gas.limit()),
            Precompile::Env(fun) => fun(input_data, gas.limit(), self.env()),
        };

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas,
            output: Bytes::new(),
        };

        match out {
            Ok((gas_used, data)) => {
                if result.gas.record_cost(gas_used) {
                    result.result = InstructionResult::Return;
                    result.output = Bytes::from(data);
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Err(e) => {
                result.result = if crate::precompile::Error::OutOfGas == e {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
        }
        result
    }

    /// Handles call return.
    #[inline]
    pub fn call_return(
        &mut self,
        interpreter_result: InterpreterResult,
        frame: Box<CallStackFrame>,
    ) -> InterpreterResult {
        // revert changes or not.
        if matches!(interpreter_result.result, return_ok!()) {
            self.journaled_state.checkpoint_commit();
        } else {
            self.journaled_state.checkpoint_revert(frame.checkpoint);
        }
        interpreter_result
    }

    /// Handles create return.
    #[inline]
    pub fn create_return<SPEC: Spec>(
        &mut self,
        mut interpreter_result: InterpreterResult,
        frame: Box<CallStackFrame>,
    ) -> (InterpreterResult, Address) {
        let address = frame.created_address.unwrap();
        // if return is not ok revert and return.
        if !matches!(interpreter_result.result, return_ok!()) {
            self.journaled_state.checkpoint_revert(frame.checkpoint);
            return (interpreter_result, address);
        }
        // Host error if present on execution
        // if ok, check contract creation limit and calculate gas deduction on output len.
        //
        // EIP-3541: Reject new contract code starting with the 0xEF byte
        if SPEC::enabled(LONDON)
            && !interpreter_result.output.is_empty()
            && interpreter_result.output.first() == Some(&0xEF)
        {
            self.journaled_state.checkpoint_revert(frame.checkpoint);
            interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
            return (interpreter_result, address);
        }

        // EIP-170: Contract code size limit
        // By default limit is 0x6000 (~25kb)
        if SPEC::enabled(SPURIOUS_DRAGON)
            && interpreter_result.output.len()
                > self
                    .env
                    .cfg
                    .limit_contract_code_size
                    .unwrap_or(MAX_CODE_SIZE)
        {
            self.journaled_state.checkpoint_revert(frame.checkpoint);
            interpreter_result.result = InstructionResult::CreateContractSizeLimit;
            return (interpreter_result, frame.created_address.unwrap());
        }
        let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
        if !interpreter_result.gas.record_cost(gas_for_code) {
            // record code deposit gas cost and check if we are out of gas.
            // EIP-2 point 3: If contract creation does not have enough gas to pay for the
            // final gas fee for adding the contract code to the state, the contract
            //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
            if SPEC::enabled(HOMESTEAD) {
                self.journaled_state.checkpoint_revert(frame.checkpoint);
                interpreter_result.result = InstructionResult::OutOfGas;
                return (interpreter_result, address);
            } else {
                interpreter_result.output = Bytes::new();
            }
        }
        // if we have enough gas we can commit changes.
        self.journaled_state.checkpoint_commit();

        // Do analysis of bytecode straight away.
        let bytecode = match self.env.cfg.perf_analyse_created_bytecodes {
            AnalysisKind::Raw => Bytecode::new_raw(interpreter_result.output.clone()),
            AnalysisKind::Check => {
                Bytecode::new_raw(interpreter_result.output.clone()).to_checked()
            }
            AnalysisKind::Analyse => {
                to_analysed(Bytecode::new_raw(interpreter_result.output.clone()))
            }
        };

        // set code
        self.journaled_state.set_code(address, bytecode);

        interpreter_result.result = InstructionResult::Return;
        (interpreter_result, address)
    }
}
