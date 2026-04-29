use crate::{
    evm::FrameTr, item_or_result::FrameInitOrResult, precompile_provider::PrecompileProvider,
    CallFrame, CreateFrame, FrameData, FrameResult, ItemOrResult,
};
use context::result::FromStringError;
use context_interface::{
    context::{take_error, ContextError},
    journaled_state::{account::JournaledAccountTr, JournalCheckpoint, JournalTr},
    local::{FrameToken, OutFrame},
    Cfg, ContextTr, Database,
};
use core::cmp::min;
use derive_where::derive_where;
use interpreter::{
    interpreter::{EthInterpreter, ExtBytecode},
    interpreter_action::FrameInit,
    interpreter_types::ReturnData,
    CallInput, CallInputs, CallOutcome, CallValue, CreateInputs, CreateOutcome, CreateScheme,
    FrameInput, Gas, InputsImpl, InstructionResult, Interpreter, InterpreterAction,
    InterpreterResult, InterpreterTypes, SharedMemory,
};
use primitives::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, SPURIOUS_DRAGON},
    keccak256, Bytes, U256,
};
use state::Bytecode;
use std::{borrow::ToOwned, boxed::Box, vec::Vec};

/// Frame implementation for Ethereum.
#[derive_where(Clone, Debug; IW,
    <IW as InterpreterTypes>::Stack,
    <IW as InterpreterTypes>::Memory,
    <IW as InterpreterTypes>::Bytecode,
    <IW as InterpreterTypes>::ReturnData,
    <IW as InterpreterTypes>::Input,
    <IW as InterpreterTypes>::RuntimeFlag,
    <IW as InterpreterTypes>::Extend,
)]
pub struct EthFrame<IW: InterpreterTypes = EthInterpreter> {
    /// Frame-specific data (Call, Create, or EOFCreate).
    pub data: FrameData,
    /// Input data for the frame.
    pub input: FrameInput,
    /// Current call depth in the execution stack.
    pub depth: usize,
    /// Journal checkpoint for state reversion.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter instance for executing bytecode.
    pub interpreter: Interpreter<IW>,
    /// Whether the frame has been finished its execution.
    /// Frame is considered finished if it has been called and returned a result.
    pub is_finished: bool,
}

impl<IT: InterpreterTypes> FrameTr for EthFrame<IT> {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

impl Default for EthFrame<EthInterpreter> {
    fn default() -> Self {
        Self::do_default(Interpreter::default())
    }
}

impl EthFrame<EthInterpreter> {
    /// Creates an new invalid [`EthFrame`].
    pub fn invalid() -> Self {
        Self::do_default(Interpreter::invalid())
    }

    fn do_default(interpreter: Interpreter<EthInterpreter>) -> Self {
        Self {
            data: FrameData::Call(CallFrame {
                return_memory_range: 0..0,
            }),
            input: FrameInput::Empty,
            depth: 0,
            checkpoint: JournalCheckpoint::default(),
            interpreter,
            is_finished: false,
        }
    }

    /// Returns true if the frame has finished execution.
    pub const fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// Sets the finished state of the frame.
    pub const fn set_finished(&mut self, finished: bool) {
        self.is_finished = finished;
    }
}

/// Type alias for database errors from a context.
pub type ContextTrDbError<CTX> = <<CTX as ContextTr>::Db as Database>::Error;

impl EthFrame<EthInterpreter> {
    /// Clear and initialize a frame.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn clear(
        &mut self,
        data: FrameData,
        input: FrameInput,
        depth: usize,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        inputs: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
        reservoir_remaining_gas: u64,
        checkpoint: JournalCheckpoint,
    ) {
        let Self {
            data: data_ref,
            input: input_ref,
            depth: depth_ref,
            interpreter,
            checkpoint: checkpoint_ref,
            is_finished: is_finished_ref,
        } = self;
        *data_ref = data;
        *input_ref = input;
        *depth_ref = depth;
        *is_finished_ref = false;
        interpreter.clear(
            memory,
            bytecode,
            inputs,
            is_static,
            spec_id,
            gas_limit,
            reservoir_remaining_gas,
        );
        *checkpoint_ref = checkpoint;
    }

    /// Make call frame
    #[inline]
    pub fn make_call_frame<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        mut this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CallInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        let reservoir_remaining_gas = inputs.reservoir;
        let gas =
            Gas::new_with_regular_gas_and_reservoir(inputs.gas_limit, reservoir_remaining_gas);
        let return_result = |instruction_result: InstructionResult| {
            Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result: InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                memory_offset: inputs.return_memory_offset.clone(),
                was_precompile_called: false,
                precompile_call_logs: Vec::new(),
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_result(InstructionResult::CallTooDeep);
        }

        // Create subroutine checkpoint
        let checkpoint = ctx.journal_mut().checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if let CallValue::Transfer(value) = inputs.value {
            // Transfer value from caller to called account
            // Target will get touched even if balance transferred is zero.
            if let Some(i) =
                ctx.journal_mut()
                    .transfer_loaded(inputs.caller, inputs.target_address, value)
            {
                ctx.journal_mut().checkpoint_revert(checkpoint);
                return return_result(i.into());
            }
        }

        let interpreter_input = InputsImpl {
            target_address: inputs.target_address,
            caller_address: inputs.caller,
            bytecode_address: Some(inputs.bytecode_address),
            input: inputs.input.clone(),
            call_value: inputs.value.get(),
        };
        let is_static = inputs.is_static;
        let gas_limit = inputs.gas_limit;

        if let Some(result) = precompiles.run(ctx, &inputs).map_err(ERROR::from_string)? {
            let mut logs = Vec::new();
            if result.result.is_ok() {
                // Preserve the reservoir on the result gas so it can be reimbursed.
                // Precompiles don't use reservoir gas, but the first frame carries it.
                ctx.journal_mut().checkpoint_commit();
            } else {
                // clone logs that precompile created, only possible with custom precompiles.
                // checkpoint.log_i will be always correct.
                logs = ctx.journal_mut().logs()[checkpoint.log_i..].to_vec();
                ctx.journal_mut().checkpoint_revert(checkpoint);
            }
            return Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result,
                memory_offset: inputs.return_memory_offset.clone(),
                was_precompile_called: true,
                precompile_call_logs: logs,
            })));
        }

        // Get bytecode and hash - either from known_bytecode or load from account
        let (bytecode_hash, bytecode) = inputs.known_bytecode.clone();

        // Returns success if bytecode is empty.
        if bytecode.is_empty() {
            ctx.journal_mut().checkpoint_commit();
            return return_result(InstructionResult::Stop);
        }

        // Create interpreter and executes call and push new CallStackFrame.
        this.get(EthFrame::invalid).clear(
            FrameData::Call(CallFrame {
                return_memory_range: inputs.return_memory_offset.clone(),
            }),
            FrameInput::Call(inputs),
            depth,
            memory,
            ExtBytecode::new_with_hash(bytecode, bytecode_hash),
            interpreter_input,
            is_static,
            ctx.cfg().spec().into(),
            gas_limit,
            reservoir_remaining_gas,
            checkpoint,
        );
        Ok(ItemOrResult::Item(this.consume()))
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        mut this: OutFrame<'_, Self>,
        context: &mut CTX,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CreateInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        let reservoir_remaining_gas = inputs.reservoir();
        let spec = context.cfg().spec().into();
        // EIP-8037 refund for the CREATE opcode's upfront `create_state_gas` is
        // applied uniformly in `return_result` when the create fails (revert,
        // halt, or early-fail with `address == None`), so early-fail results
        // only carry the reservoir they inherited from the parent.
        let return_error = |e| {
            Ok(ItemOrResult::Result(FrameResult::Create(CreateOutcome {
                result: InterpreterResult {
                    result: e,
                    gas: Gas::new_with_regular_gas_and_reservoir(
                        inputs.gas_limit(),
                        reservoir_remaining_gas,
                    ),
                    output: Bytes::new(),
                },
                address: None,
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let journal = context.journal_mut();
        let mut caller_info = journal.load_account_mut(inputs.caller())?;

        // Check if caller has enough balance to send to the created contract.
        // decrement of balance is done in the create_account_checkpoint.
        if *caller_info.balance() < inputs.value() {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce = caller_info.nonce();
        if !caller_info.bump_nonce() {
            return return_error(InstructionResult::Return);
        };

        // Create address
        let mut init_code_hash = None;
        let created_address = match inputs.scheme() {
            CreateScheme::Create => inputs.caller().create(old_nonce),
            CreateScheme::Create2 { salt } => {
                let init_code_hash = *init_code_hash.insert(keccak256(inputs.init_code()));
                inputs.caller().create2(salt.to_be_bytes(), init_code_hash)
            }
            CreateScheme::Custom { address } => address,
        };

        drop(caller_info); // Drop caller info to avoid borrow checker issues.

        // warm load account.
        journal.load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal_mut().create_account_checkpoint(
            inputs.caller(),
            created_address,
            inputs.value(),
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let bytecode = ExtBytecode::new_with_optional_hash(
            Bytecode::new_legacy(inputs.init_code().clone()),
            init_code_hash,
        );

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller(),
            bytecode_address: None,
            input: CallInput::Bytes(Bytes::new()),
            call_value: inputs.value(),
        };
        let gas_limit = inputs.gas_limit();

        this.get(EthFrame::invalid).clear(
            FrameData::Create(CreateFrame { created_address }),
            FrameInput::Create(inputs),
            depth,
            memory,
            bytecode,
            interpreter_input,
            false,
            spec,
            gas_limit,
            reservoir_remaining_gas,
            checkpoint,
        );

        Ok(ItemOrResult::Item(this.consume()))
    }

    /// Initializes a frame with the given context and precompiles.
    pub fn init_with_context<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
    >(
        this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        frame_init: FrameInit,
    ) -> Result<
        ItemOrResult<FrameToken, FrameResult>,
        ContextError<<<CTX as ContextTr>::Db as Database>::Error>,
    > {
        // TODO cleanup inner make functions
        let FrameInit {
            depth,
            memory,
            frame_input,
        } = frame_init;

        match frame_input {
            FrameInput::Call(inputs) => {
                Self::make_call_frame(this, ctx, precompiles, depth, memory, inputs)
            }
            FrameInput::Create(inputs) => Self::make_create_frame(this, ctx, depth, memory, inputs),
            FrameInput::Empty => unreachable!(),
        }
    }
}

impl EthFrame<EthInterpreter> {
    /// Processes the next interpreter action, either creating a new frame or returning a result.
    pub fn process_next_action<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        &mut self,
        context: &mut CTX,
        next_action: InterpreterAction,
    ) -> Result<FrameInitOrResult<Self>, ERROR> {
        // Run interpreter

        let mut interpreter_result = match next_action {
            InterpreterAction::NewFrame(frame_input) => {
                let depth = self.depth + 1;
                return Ok(ItemOrResult::Item(FrameInit {
                    frame_input,
                    depth,
                    memory: self.interpreter.memory.new_child_context(),
                }));
            }
            InterpreterAction::Return(result) => result,
        };

        // For Create frames, run validation + bump new_state for code deposit.
        // This may set interpreter_result.result to a failure variant.
        let create_address = match &self.data {
            FrameData::Create(frame) => Some(frame.created_address),
            FrameData::Call(_) => None,
        };
        if create_address.is_some() {
            return_create(context, &mut self.interpreter, &mut interpreter_result);
        }

        // Apply per-frame state-gas reconciliation (EIP-8037, deferred model).
        // - on ok: refund first (grows reservoir), then charge (may OOG)
        // - on revert/halt: drop the counters (state work undone, no charge)
        let new_state = self.interpreter.new_state;
        self.interpreter.new_state.clear();
        if interpreter_result.result.is_ok() {
            interpreter_result
                .gas
                .refill_reservoir(new_state.state_gas_refunded);
            if !interpreter_result.gas.record_state_cost(new_state.state_gas) {
                interpreter_result.result = InstructionResult::OutOfGas;
            }
        }

        // Commit or revert the journal based on the final result; for a
        // successful CREATE, also persist the deployed bytecode.
        if interpreter_result.result.is_ok() {
            context.journal_mut().checkpoint_commit();
            if let Some(address) = create_address {
                let bytecode = Bytecode::new_legacy(interpreter_result.output.clone());
                context.journal_mut().set_code(address, bytecode);
                interpreter_result.result = InstructionResult::Return;
            }
        } else {
            context.journal_mut().checkpoint_revert(self.checkpoint);
        }

        let result = match &self.data {
            FrameData::Call(frame) => ItemOrResult::Result(FrameResult::Call(CallOutcome::new(
                interpreter_result,
                frame.return_memory_range.clone(),
            ))),
            FrameData::Create(frame) => ItemOrResult::Result(FrameResult::Create(
                CreateOutcome::new(interpreter_result, Some(frame.created_address)),
            )),
        };

        Ok(result)
    }

    /*

    10:5 regular:reservoir

    1:6 sstore 1 regular 6 state.

    call1 10:5 state_created:0 state_refunded:0
    add 9:5 state_created:0 state_refunded:0
    call2 9:5 state_created:0 state_refunded:0
    sstore 8:5 state_created:6 state_refunded:0
    call2 return ok 7:0 state_created:6 state_refunded:0
    call1 7:0 state_created:6 state_refunded:0
    call3 7:0 state_created:0 state_refunded:0
    sstore 6:0 state_created:6 state_refunded:0

        call3 ok 1:0 state_created:6 state_refunded:0
        call1 1:0 state_created:12 state_refunded:0

        call3 halts: 6:0 state_created:0 state_refunded:0
        call1 6:0 state_created:6 state_refunded:0

        call3 reverts: 6:0 state_created:6 state_refunded:0
        call1 6:0 state_created:6 state_refunded:0

    call1 1:0 state_created:12 state_refunded:0

        case call1 return ok: Expectation two sstores were created 12 gas should be spent.
        call1 1:0

        case call1 return halt: Expectation two sstores were created 12 gas should be returned.
        call1 0:5 (use previous parent gas value)

        case call1 return revert: Expectation two sstores were created 12 gas should be returned.
        call1 1:12 reservoir gets increased by 12.



    Rules:
    * Inside one frame

    */

    /// Processes a frame result and updates the interpreter state accordingly.
    pub fn return_result<CTX: ContextTr, ERROR: From<ContextTrDbError<CTX>> + FromStringError>(
        &mut self,
        ctx: &mut CTX,
        result: FrameResult,
    ) -> Result<(), ERROR> {
        self.interpreter.memory.free_child_context();
        take_error::<ERROR, _>(ctx.error())?;

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                let out_gas = outcome.gas();
                let ins_result = *outcome.instruction_result();
                let returned_len = outcome.result.output.len();

                let interpreter = &mut self.interpreter;
                let mem_length = outcome.memory_length();
                let mem_start = outcome.memory_start();
                interpreter.return_data.set_buffer(outcome.result.output);

                let target_len = min(mem_length, returned_len);

                if ins_result == InstructionResult::FatalExternalError {
                    panic!("Fatal external error in insert_call_outcome");
                }

                let item = if ins_result.is_ok() {
                    U256::from(1)
                } else {
                    U256::ZERO
                };
                // Safe to push without stack limit check
                let _ = interpreter.stack.push(item);

                // Return unspend gas.
                if ins_result.is_ok_or_revert() {
                    interpreter.gas.erase_cost(out_gas.remaining());
                    interpreter
                        .memory
                        .set(mem_start, &interpreter.return_data.buffer()[..target_len]);
                }

                // handle reservoir remaining gas
                handle_reservoir_remaining_gas(ins_result.is_ok(), &mut interpreter.gas, &out_gas);

                if ins_result.is_ok() {
                    interpreter.gas.record_refund(out_gas.refunded());
                }
            }
            FrameResult::Create(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;

                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    interpreter
                        .return_data
                        .set_buffer(outcome.output().to_owned());
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                {
                    let this_gas = &mut interpreter.gas;
                    // Refund unused gas for success and revert cases.
                    if instruction_result.is_ok_or_revert() {
                        this_gas.erase_cost(outcome.gas().remaining());
                    }

                    // handle reservoir remaining gas
                    handle_reservoir_remaining_gas(
                        instruction_result.is_ok(),
                        this_gas,
                        outcome.gas(),
                    );

                    if instruction_result.is_ok() {
                        this_gas.record_refund(outcome.gas().refunded());
                    }
                }

                // EIP-8037 (deferred model): the CREATE opcode bumped this
                // frame's `new_state.add_create_account()` for the upfront
                // create_state_gas. On child failure (revert/halt/early-fail
                // paths that return `address == None` — nonce overflow, depth,
                // OutOfFunds), undo the bump so the parent doesn't end up
                // charged for a CREATE that didn't deploy.
                let create_failed = outcome.address.is_none() || !instruction_result.is_ok();
                if create_failed && ctx.cfg().is_amsterdam_eip8037_enabled() {
                    interpreter.new_state.remove_create_account();
                }

                let stack_item = if instruction_result.is_ok() {
                    outcome.address.unwrap_or_default().into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
        }

        Ok(())
    }
}

/// Handles the remaining gas of the parent frame.
///
/// Under the deferred state-gas model (EIP-8037), the child frame applies its
/// own state-gas to its gas tracker at frame return (`process_next_action`):
/// on success the charge is committed to `reservoir` / `remaining`, on
/// revert/halt the per-frame counters are dropped. As a result the child's
/// reservoir is always "the parent's pre-call value plus the net effect of
/// successful sub-work" — there's no spill-on-revert distortion to undo here.
#[inline]
pub fn handle_reservoir_remaining_gas(is_success: bool, parent_gas: &mut Gas, child_gas: &Gas) {
    parent_gas.set_reservoir(child_gas.reservoir());

    if is_success {
        // Audit totals: accumulate child's contribution into the parent's.
        // (state_gas_spent / refill_amount on the gas tracker are populated
        // by the child's process_next_action when state-gas was applied.)
        parent_gas.set_state_gas_spent(
            parent_gas
                .state_gas_spent()
                .saturating_add(child_gas.state_gas_spent()),
        );
        parent_gas.set_refill_amount(
            parent_gas
                .refill_amount()
                .saturating_add(child_gas.refill_amount()),
        );
    }
}

/// Validates the deployed bytecode of a successful CREATE/CREATE2 and bumps
/// the per-frame state-gas counter. Charging state gas and committing the
/// journal are handled by [`EthFrame::process_next_action`] after this call.
///
/// Sets `interpreter_result.result` to a failure variant (e.g.
/// `CreateContractSizeLimit`, `OutOfGas`) when validation or gas charges fail;
/// the caller is responsible for journal revert in that case.
pub fn return_create<CTX: ContextTr>(
    context: &mut CTX,
    interpreter: &mut Interpreter<EthInterpreter>,
    interpreter_result: &mut InterpreterResult,
) {
    let (_, _, cfg, _, _, _) = context.all_mut();

    let max_code_size = cfg.max_code_size();
    let is_eip3541_disabled = cfg.is_eip3541_disabled();
    let spec_id = cfg.spec().into();
    let is_amsterdam_eip8037 = cfg.is_amsterdam_eip8037_enabled();
    let gas_params = cfg.gas_params();

    // If return is not ok, nothing to validate or charge.
    if !interpreter_result.result.is_ok() {
        return;
    }

    // EIP-170 / EIP-7954: code size limit. Checked BEFORE state gas so
    // oversized code does not incur storage gas costs.
    if spec_id.is_enabled_in(SPURIOUS_DRAGON) && interpreter_result.output.len() > max_code_size {
        interpreter_result.result = InstructionResult::CreateContractSizeLimit;
        return;
    }

    // EIP-3541: reject new contract code starting with the 0xEF byte.
    if !is_eip3541_disabled
        && spec_id.is_enabled_in(LONDON)
        && interpreter_result.output.first() == Some(&0xEF)
    {
        interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
        return;
    }

    // Regular gas for code deposit. Zero under EIP-8037.
    let gas_for_code = gas_params.code_deposit_cost(interpreter_result.output.len());
    if !interpreter_result.gas.record_regular_cost(gas_for_code) {
        // EIP-2 point 3: If contract creation does not have enough gas to pay
        // for the final gas fee for adding the contract code to the state,
        // the contract creation fails (i.e. goes out-of-gas) rather than
        // leaving an empty contract.
        if spec_id.is_enabled_in(HOMESTEAD) {
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        } else {
            interpreter_result.output = Bytes::new();
        }
    }

    // EIP-8037: hash cost for deployed bytecode (keccak256).
    // HASH_COST(L) = 6 × ceil(L / 32). Both CREATE and CREATE2 pay this — it
    // covers hashing the deployed code to compute the code_hash stored on the
    // account (distinct from CREATE2's init-code hash).
    if is_amsterdam_eip8037 {
        let hash_cost = gas_params.keccak256_cost(interpreter_result.output.len());
        if !interpreter_result.gas.record_regular_cost(hash_cost) {
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        }
        // State gas for code deposit accumulates on the frame's NewStateTracker;
        // the actual reservoir charge happens in process_next_action.
        interpreter
            .new_state
            .add_code_deposit_bytes(interpreter_result.output.len() as u64);
    }
}
