use super::frame_data::*;
use bytecode::{Eof, EOF_MAGIC_BYTES};
use context_interface::{
    journaled_state::{Journal, JournalCheckpoint},
    BlockGetter, Cfg, CfgGetter, ErrorGetter, JournalDBError, JournalGetter, Transaction,
    TransactionGetter,
};
use core::{cell::RefCell, cmp::min};
use handler_interface::{Frame, FrameOrResultGen, PrecompileProvider};
use interpreter::{
    gas,
    interpreter::{EthInterpreter, ExtBytecode, InstructionProvider},
    interpreter_types::{LoopControl, ReturnData, RuntimeFlag},
    return_ok, return_revert, CallInputs, CallOutcome, CallValue, CreateInputs, CreateOutcome,
    CreateScheme, EOFCreateInputs, EOFCreateKind, FrameInput, Gas, Host, InputsImpl,
    InstructionResult, Interpreter, InterpreterAction, InterpreterResult, InterpreterTypes,
    SharedMemory,
};
use precompile::PrecompileErrors;
use primitives::{keccak256, Address, Bytes, B256, U256};
use specification::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, OSAKA, SPURIOUS_DRAGON},
};
use state::Bytecode;
use std::borrow::ToOwned;
use std::{rc::Rc, sync::Arc};

pub struct EthFrame<CTX, ERROR, IW: InterpreterTypes, PRECOMPILE, INSTRUCTIONS> {
    _phantom: core::marker::PhantomData<fn() -> (CTX, ERROR)>,
    data: FrameData,
    // TODO : Include this
    depth: usize,
    /// Journal checkpoint.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter.
    pub interpreter: Interpreter<IW>,
    /// Precompiles provider.
    pub precompiles: PRECOMPILE,
    /// Instruction provider.
    pub instructions: INSTRUCTIONS,
    // This is worth making as a generic type FrameSharedContext.
    pub memory: Rc<RefCell<SharedMemory>>,
}

impl<CTX, IW, ERROR, PRECOMP, INST> EthFrame<CTX, ERROR, IW, PRECOMP, INST>
where
    CTX: JournalGetter,
    IW: InterpreterTypes,
{
    pub fn new(
        data: FrameData,
        depth: usize,
        interpreter: Interpreter<IW>,
        checkpoint: JournalCheckpoint,
        precompiles: PRECOMP,
        instructions: INST,
        memory: Rc<RefCell<SharedMemory>>,
    ) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            data,
            depth,
            interpreter,
            checkpoint,
            precompiles,
            instructions,
            memory,
        }
    }
}

impl<CTX, ERROR, PRECOMPILE, INSTRUCTION>
    EthFrame<CTX, ERROR, EthInterpreter<()>, PRECOMPILE, INSTRUCTION>
where
    CTX: EthFrameContext,
    ERROR: EthFrameError<CTX>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR, Output = InterpreterResult>,
{
    /// Make call frame
    #[inline]
    pub fn make_call_frame(
        context: &mut CTX,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: &CallInputs,
        mut precompile: PRECOMPILE,
        instructions: INSTRUCTION,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        let gas = Gas::new(inputs.gas_limit);

        let return_result = |instruction_result: InstructionResult| {
            Ok(FrameOrResultGen::Result(FrameResult::Call(CallOutcome {
                result: InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                memory_offset: inputs.return_memory_offset.clone(),
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_result(InstructionResult::CallTooDeep);
        }

        // Make account warm and loaded
        let _ = context
            .journal()
            .load_account(inputs.bytecode_address)?;

        // Create subroutine checkpoint
        let checkpoint = context.journal().checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if let CallValue::Transfer(value) = inputs.value {
            // Transfer value from caller to called account
            // Target will get touched even if balance transferred is zero.
            if let Some(i) =
                context
                    .journal()
                    .transfer(&inputs.caller, &inputs.target_address, value)?
            {
                context.journal().checkpoint_revert(checkpoint);
                return return_result(i.into());
            }
        }
        let is_ext_delegate_call = inputs.scheme.is_ext_delegate_call();
        if !is_ext_delegate_call {
            if let Some(result) = precompile.run(
                context,
                &inputs.bytecode_address,
                &inputs.input,
                inputs.gas_limit,
            )? {
                if result.result.is_ok() {
                    context.journal().checkpoint_commit();
                } else {
                    context.journal().checkpoint_revert(checkpoint);
                }
                return Ok(FrameOrResultGen::Result(FrameResult::Call(CallOutcome {
                    result,
                    memory_offset: inputs.return_memory_offset.clone(),
                })));
            }
        }

        let account = context
            .journal()
            .load_account_code(inputs.bytecode_address)?;

        let mut code_hash = account.info.code_hash();
        let mut bytecode = account.info.code.clone().unwrap_or_default();

        // ExtDelegateCall is not allowed to call non-EOF contracts.
        if is_ext_delegate_call && !bytecode.bytes_slice().starts_with(&EOF_MAGIC_BYTES) {
            return return_result(InstructionResult::InvalidExtDelegateCallTarget);
        }

        if bytecode.is_empty() {
            context.journal().checkpoint_commit();
            return return_result(InstructionResult::Stop);
        }

        if let Bytecode::Eip7702(eip7702_bytecode) = bytecode {
            let account = &context
                .journal()
                .load_account_code(eip7702_bytecode.delegated_address)?
                .info;
            bytecode = account.code.clone().unwrap_or_default();
            code_hash = account.code_hash();
        }

        // Create interpreter and executes call and push new CallStackFrame.
        let interpreter_input = InputsImpl {
            target_address: inputs.target_address,
            caller_address: inputs.caller,
            input: inputs.input.clone(),
            call_value: inputs.value.get(),
        };

        Ok(FrameOrResultGen::Frame(Self::new(
            FrameData::Call(CallFrame {
                return_memory_range: inputs.return_memory_offset.clone(),
            }),
            depth,
            Interpreter::new(
                memory.clone(),
                ExtBytecode::new_with_hash(bytecode, code_hash),
                interpreter_input,
                inputs.is_static,
                false,
                context.cfg().spec().into(),
                inputs.gas_limit,
            ),
            checkpoint,
            precompile,
            instructions,
            memory,
        )))
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame(
        context: &mut CTX,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: &CreateInputs,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        let spec = context.cfg().spec().into();
        let return_error = |e| {
            Ok(FrameOrResultGen::Result(FrameResult::Create(
                CreateOutcome {
                    result: InterpreterResult {
                        result: e,
                        gas: Gas::new(inputs.gas_limit),
                        output: Bytes::new(),
                    },
                    address: None,
                },
            )))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Prague EOF
        if spec.is_enabled_in(OSAKA) && inputs.init_code.starts_with(&EOF_MAGIC_BYTES) {
            return return_error(InstructionResult::CreateInitCodeStartingEF00);
        }

        // Fetch balance of caller.
        let caller_balance = context
            .journal()
            .load_account(inputs.caller)?
            .map(|a| a.info.balance);

        // Check if caller has enough balance to send to the created contract.
        if caller_balance.data < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = context.journal().inc_account_nonce(inputs.caller)? {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        // TODO : Incorporating code hash inside interpreter. It was a request by foundry.
        let mut _init_code_hash = B256::ZERO;
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => {
                _init_code_hash = keccak256(&inputs.init_code);
                inputs.caller.create2(salt.to_be_bytes(), _init_code_hash)
            }
        };

        // warm load account.
        context.journal().load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let bytecode = ExtBytecode::new(Bytecode::new_legacy(inputs.init_code.clone()));

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input: Bytes::new(),
            call_value: inputs.value,
        };

        Ok(FrameOrResultGen::Frame(Self::new(
            FrameData::Create(CreateFrame { created_address }),
            depth,
            Interpreter::new(
                memory.clone(),
                bytecode,
                interpreter_input,
                false,
                false,
                spec,
                inputs.gas_limit,
            ),
            checkpoint,
            precompile,
            instructions,
            memory,
        )))
    }

    /// Make create frame.
    #[inline]
    pub fn make_eofcreate_frame(
        context: &mut CTX,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: &EOFCreateInputs,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        let spec = context.cfg().spec().into();
        let return_error = |e| {
            Ok(FrameOrResultGen::Result(FrameResult::EOFCreate(
                CreateOutcome {
                    result: InterpreterResult {
                        result: e,
                        gas: Gas::new(inputs.gas_limit),
                        output: Bytes::new(),
                    },
                    address: None,
                },
            )))
        };

        let (input, initcode, created_address) = match &inputs.kind {
            EOFCreateKind::Opcode {
                initcode,
                input,
                created_address,
            } => (input.clone(), initcode.clone(), Some(*created_address)),
            EOFCreateKind::Tx { initdata } => {
                // Decode eof and init code.
                // TODO : Handle inc_nonce handling more gracefully.
                let Ok((eof, input)) = Eof::decode_dangling(initdata.clone()) else {
                    context.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                };

                if eof.validate().is_err() {
                    // TODO : (EOF) New error type.
                    context.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                }

                // Use nonce from tx to calculate address.
                let tx = context.tx();
                let create_address = tx.caller().create(tx.nonce());

                (input, eof, Some(create_address))
            }
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let caller_balance = context
            .journal()
            .load_account(inputs.caller)?
            .map(|a| a.info.balance);

        // Check if caller has enough balance to send to the created contract.
        if caller_balance.data < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let Some(nonce) = context.journal().inc_account_nonce(inputs.caller)? else {
            // Can't happen on mainnet.
            return return_error(InstructionResult::Return);
        };
        let old_nonce = nonce - 1;

        let created_address = created_address.unwrap_or_else(|| inputs.caller.create(old_nonce));

        // Load account so it needs to be marked as warm for access list.
        context.journal().load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input,
            call_value: inputs.value,
        };

        Ok(FrameOrResultGen::Frame(Self::new(
            FrameData::EOFCreate(EOFCreateFrame { created_address }),
            depth,
            Interpreter::new(
                memory.clone(),
                ExtBytecode::new(Bytecode::Eof(Arc::new(initcode))),
                interpreter_input,
                false,
                true,
                spec,
                inputs.gas_limit,
            ),
            checkpoint,
            precompile,
            instructions,
            memory,
        )))
    }

    pub fn init_with_context(
        depth: usize,
        frame_init: FrameInput,
        memory: Rc<RefCell<SharedMemory>>,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
        context: &mut CTX,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        match frame_init {
            FrameInput::Call(inputs) => {
                Self::make_call_frame(context, depth, memory, &inputs, precompile, instructions)
            }
            FrameInput::Create(inputs) => {
                Self::make_create_frame(context, depth, memory, &inputs, precompile, instructions)
            }
            FrameInput::EOFCreate(inputs) => Self::make_eofcreate_frame(
                context,
                depth,
                memory,
                &inputs,
                precompile,
                instructions,
            ),
        }
    }
}

impl<CTX, ERROR, PRECOMPILE, INSTRUCTION> Frame
    for EthFrame<CTX, ERROR, EthInterpreter<()>, PRECOMPILE, INSTRUCTION>
where
    CTX: EthFrameContext,
    ERROR: EthFrameError<CTX>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR, Output = InterpreterResult>,
    INSTRUCTION: InstructionProvider<WIRE = EthInterpreter<()>, Host = CTX>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;

    fn init_first(
        context: &mut Self::Context,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        let memory = Rc::new(RefCell::new(SharedMemory::new()));
        let precompiles = PRECOMPILE::new(context);
        let instructions = INSTRUCTION::new(context);

        // Load precompiles addresses as warm.
        for address in precompiles.warm_addresses() {
            context.journal().warm_account(address);
        }

        memory.borrow_mut().new_context();
        Self::init_with_context(0, frame_input, memory, precompiles, instructions, context)
    }

    fn final_return(
        _context: &mut Self::Context,
        _result: &mut Self::FrameResult,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn init(
        &self,
        context: &mut CTX,
        frame_init: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.memory.borrow_mut().new_context();
        Self::init_with_context(
            self.depth + 1,
            frame_init,
            self.memory.clone(),
            self.precompiles.clone(),
            self.instructions.clone(),
            context,
        )
    }

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        let spec = context.cfg().spec().into();

        // Run interpreter
        let next_action = self.interpreter.run(self.instructions.table(), context);

        let mut interpreter_result = match next_action {
            InterpreterAction::NewFrame(new_frame) => {
                return Ok(FrameOrResultGen::Frame(new_frame))
            }
            InterpreterAction::Return { result } => result,
            InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
        };

        // Handle return from frame
        let result = match &self.data {
            FrameData::Call(frame) => {
                // return_call
                // Revert changes or not.
                if interpreter_result.result.is_ok() {
                    context.journal().checkpoint_commit();
                } else {
                    context.journal().checkpoint_revert(self.checkpoint);
                }
                FrameOrResultGen::Result(FrameResult::Call(CallOutcome::new(
                    interpreter_result,
                    frame.return_memory_range.clone(),
                )))
            }
            FrameData::Create(frame) => {
                let max_code_size = context.cfg().max_code_size();
                return_create(
                    context.journal(),
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                    max_code_size,
                    spec,
                );

                FrameOrResultGen::Result(FrameResult::Create(CreateOutcome::new(
                    interpreter_result,
                    Some(frame.created_address),
                )))
            }
            FrameData::EOFCreate(frame) => {
                let max_code_size = context.cfg().max_code_size();
                return_eofcreate(
                    context.journal(),
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                    max_code_size,
                );

                FrameOrResultGen::Result(FrameResult::EOFCreate(CreateOutcome::new(
                    interpreter_result,
                    Some(frame.created_address),
                )))
            }
        };

        Ok(result)
    }

    fn return_result(
        &mut self,
        context: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        self.memory.borrow_mut().free_context();
        context.take_error()?;

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                let out_gas = outcome.gas();
                let ins_result = *outcome.instruction_result();
                let returned_len = outcome.result.output.len();

                let interpreter = &mut self.interpreter;
                let mem_length = outcome.memory_length();
                let mem_start = outcome.memory_start();
                *interpreter.return_data.buffer_mut() = outcome.result.output;

                let target_len = min(mem_length, returned_len);

                if ins_result == InstructionResult::FatalExternalError {
                    panic!("Fatal external error in insert_call_outcome");
                }

                let item = {
                    if interpreter.runtime_flag.is_eof() {
                        match ins_result {
                            return_ok!() => U256::ZERO,
                            return_revert!() => U256::from(1),
                            _ => U256::from(2),
                        }
                    } else if ins_result.is_ok() {
                        U256::from(1)
                    } else {
                        U256::ZERO
                    }
                };
                // Safe to push without stack limit check
                let _ = interpreter.stack.push(item);

                // Return unspend gas.
                if ins_result.is_ok_or_revert() {
                    interpreter.control.gas().erase_cost(out_gas.remaining());
                    self.memory
                        .borrow_mut()
                        .set(mem_start, &interpreter.return_data.buffer()[..target_len]);
                }

                if ins_result.is_ok() {
                    interpreter.control.gas().record_refund(out_gas.refunded());
                }
            }
            FrameResult::Create(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;

                let buffer = interpreter.return_data.buffer_mut();
                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    *buffer = outcome.output().to_owned()
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    buffer.clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let this_gas = interpreter.control.gas();
                if instruction_result.is_ok_or_revert() {
                    this_gas.erase_cost(outcome.gas().remaining());
                }

                let stack_item = if instruction_result.is_ok() {
                    this_gas.record_refund(outcome.gas().refunded());
                    outcome.address.unwrap_or_default().into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
            FrameResult::EOFCreate(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;
                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    *interpreter.return_data.buffer_mut() = outcome.output().to_owned()
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.buffer_mut().clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let this_gas = interpreter.control.gas();
                if instruction_result.is_ok_or_revert() {
                    this_gas.erase_cost(outcome.gas().remaining());
                }

                let stack_item = if instruction_result.is_ok() {
                    this_gas.record_refund(outcome.gas().refunded());
                    outcome.address.expect("EOF Address").into_word().into()
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

pub fn return_create<JOURNAL: Journal>(
    journal: &mut JOURNAL,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    address: Address,
    max_code_size: usize,
    spec_id: SpecId,
) {
    // If return is not ok revert and return.
    if !interpreter_result.result.is_ok() {
        journal.checkpoint_revert(checkpoint);
        return;
    }
    // Host error if present on execution
    // If ok, check contract creation limit and calculate gas deduction on output len.
    //
    // EIP-3541: Reject new contract code starting with the 0xEF byte
    if spec_id.is_enabled_in(LONDON) && interpreter_result.output.first() == Some(&0xEF) {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
        return;
    }

    // EIP-170: Contract code size limit
    // By default limit is 0x6000 (~25kb)
    if spec_id.is_enabled_in(SPURIOUS_DRAGON) && interpreter_result.output.len() > max_code_size {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractSizeLimit;
        return;
    }
    let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
    if !interpreter_result.gas.record_cost(gas_for_code) {
        // Record code deposit gas cost and check if we are out of gas.
        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
        // final gas fee for adding the contract code to the state, the contract
        // creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
        if spec_id.is_enabled_in(HOMESTEAD) {
            journal.checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        } else {
            interpreter_result.output = Bytes::new();
        }
    }
    // If we have enough gas we can commit changes.
    journal.checkpoint_commit();

    // Do analysis of bytecode straight away.
    let bytecode = Bytecode::new_legacy(interpreter_result.output.clone());

    // Set code
    journal.set_code(address, bytecode);

    interpreter_result.result = InstructionResult::Return;
}

pub fn return_eofcreate<JOURNAL: Journal>(
    journal: &mut JOURNAL,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    address: Address,
    max_code_size: usize,
) {
    // Note we still execute RETURN opcode and return the bytes.
    // In EOF those opcodes should abort execution.
    //
    // In RETURN gas is still protecting us from ddos and in oog,
    // behaviour will be same as if it failed on return.
    //
    // Bytes of RETURN will drained in `insert_eofcreate_outcome`.
    if interpreter_result.result != InstructionResult::ReturnContract {
        journal.checkpoint_revert(checkpoint);
        return;
    }

    if interpreter_result.output.len() > max_code_size {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractSizeLimit;
        return;
    }

    // Deduct gas for code deployment.
    let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
    if !interpreter_result.gas.record_cost(gas_for_code) {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::OutOfGas;
        return;
    }

    journal.checkpoint_commit();

    // Decode bytecode has a performance hit, but it has reasonable restrains.
    let bytecode = Eof::decode(interpreter_result.output.clone()).expect("Eof is already verified");

    // Eof bytecode is going to be hashed.
    journal.set_code(address, Bytecode::Eof(Arc::new(bytecode)));
}

pub trait EthFrameContext:
    TransactionGetter
    + Host
    + ErrorGetter<Error = JournalDBError<Self>>
    + BlockGetter
    + JournalGetter
    + CfgGetter
{
}

impl<
        CTX: TransactionGetter
            + ErrorGetter<Error = JournalDBError<CTX>>
            + BlockGetter
            + JournalGetter
            + CfgGetter
            + Host,
    > EthFrameContext for CTX
{
}

pub trait EthFrameError<CTX: JournalGetter>:
    From<JournalDBError<CTX>> + From<PrecompileErrors>
{
}

impl<CTX: JournalGetter, T: From<JournalDBError<CTX>> + From<PrecompileErrors>> EthFrameError<CTX>
    for T
{
}
