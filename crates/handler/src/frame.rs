use super::frame_data::*;
use bytecode::{Eof, EOF_MAGIC_BYTES};
use context_interface::{
    journaled_state::{JournalCheckpoint, JournaledState},
    BlockGetter, Cfg, CfgGetter, ErrorGetter, JournalStateGetter, JournalStateGetterDBError,
    Transaction, TransactionGetter,
};
use core::{cell::RefCell, cmp::min};
use handler_interface::{Frame, FrameOrResultGen, PrecompileProvider};
use interpreter::{
    gas,
    interpreter::{EthInterpreter, InstructionProvider},
    interpreter_wiring::{LoopControl, ReturnData, RuntimeFlag},
    return_ok, return_revert, CallInputs, CallOutcome, CallValue, CreateInputs, CreateOutcome,
    CreateScheme, EOFCreateInputs, EOFCreateKind, FrameInput, Gas, Host, InputsImpl,
    InstructionResult, InterpreterAction, InterpreterResult, InterpreterWire, NewInterpreter,
    SharedMemory,
};
use precompile::PrecompileErrors;
use primitives::{keccak256, Address, Bytes, B256, U256};
use specification::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, PRAGUE_EOF, SPURIOUS_DRAGON},
};
use state::Bytecode;
use std::borrow::ToOwned;
use std::{rc::Rc, sync::Arc};

/*
EVM
    context:
        TX
        BLOCK
        JOURNAL
        CFG
    handler:
        VAL
        PRE
        POST
        EXEC:
            FRAME
                Data: CALL/CREATE/EOFCREATE
                INTERPRETER
                INSTRUCTION
                PRECOMPILE


EthFrame<CTX,IW,PR<CTX>,IP<IW,CTX> {

}
*/

/*

EXEC -> FRAME makes sense

FRAME needs to have
NewInterpreter<WIRE>
InnerContext<PRECOMPILES,INSTRUCTIONS, MEMORY>


*/

pub struct EthFrame<CTX, ERROR, IW: InterpreterWire, PRECOMPILE, INSTRUCTIONS> {
    _phantom: core::marker::PhantomData<fn() -> (CTX, ERROR)>,
    data: FrameData,
    // TODO include this
    depth: usize,
    /// Journal checkpoint.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter.
    pub interpreter: NewInterpreter<IW>,
    /// Precompiles provider.
    pub precompiles: PRECOMPILE,
    /// Instruction provider.
    pub instructions: INSTRUCTIONS,
    // This is worth making as a generic type FrameSharedContext.
    pub memory: Rc<RefCell<SharedMemory>>,
}

impl<CTX, IW, ERROR, PRECOMP, INST> EthFrame<CTX, ERROR, IW, PRECOMP, INST>
where
    CTX: JournalStateGetter,
    IW: InterpreterWire,
{
    pub fn new(
        data: FrameData,
        interpreter: NewInterpreter<IW>,
        checkpoint: JournalCheckpoint,
        precompiles: PRECOMP,
        instructions: INST,
        memory: Rc<RefCell<SharedMemory>>,
    ) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            data,
            depth: 0,
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
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter
        + CfgGetter,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
    ERROR: From<JournalStateGetterDBError<CTX>> + From<PrecompileErrors>,
{
    /// Make call frame
    #[inline]
    pub fn make_call_frame(
        ctx: &mut CTX,
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
        let _ = ctx
            .journal()
            .load_account_delegated(inputs.bytecode_address)?;

        // Create subroutine checkpoint
        let checkpoint = ctx.journal().checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        match inputs.value {
            // if transfer value is zero, load account and force the touch.
            CallValue::Transfer(value) if value.is_zero() => {
                ctx.journal().load_account(inputs.target_address)?;
                ctx.journal().touch_account(inputs.target_address);
            }
            CallValue::Transfer(value) => {
                // Transfer value from caller to called account. As value get transferred
                // target gets touched.
                if let Some(_) =
                    ctx.journal()
                        .transfer(&inputs.caller, &inputs.target_address, value)?
                {
                    ctx.journal().checkpoint_revert(checkpoint);
                    // TODO this is hardcoded value, we need to resolve conflict in Journal trait.
                    return return_result(InstructionResult::CreateCollision);
                }
            }
            _ => {}
        };
        // TODO
        if let Some(result) = precompile.run(
            ctx,
            &inputs.bytecode_address,
            &inputs.input,
            inputs.gas_limit,
        )? {
            if result.result.is_ok() {
                ctx.journal().checkpoint_commit();
            } else {
                ctx.journal().checkpoint_revert(checkpoint);
            }
            Ok(FrameOrResultGen::Result(FrameResult::Call(CallOutcome {
                result,
                memory_offset: inputs.return_memory_offset.clone(),
            })))
        } else {
            let account = ctx.journal().load_account_code(inputs.bytecode_address)?;

            // TODO Request from foundry to get bytecode hash.
            let _code_hash = account.info.code_hash();
            let mut bytecode = account.info.code.clone().unwrap_or_default();

            // ExtDelegateCall is not allowed to call non-EOF contracts.
            if inputs.scheme.is_ext_delegate_call()
                && !bytecode.bytes_slice().starts_with(&EOF_MAGIC_BYTES)
            {
                return return_result(InstructionResult::InvalidExtDelegateCallTarget);
            }

            if bytecode.is_empty() {
                ctx.journal().checkpoint_commit();
                return return_result(InstructionResult::Stop);
            }

            if let Bytecode::Eip7702(eip7702_bytecode) = bytecode {
                bytecode = ctx
                    .journal()
                    .load_account_code(eip7702_bytecode.delegated_address)?
                    .info
                    .code
                    .clone()
                    .unwrap_or_default();
            }

            //let contract =
            //    Contract::new_with_context(inputs.input.clone(), bytecode, Some(code_hash), inputs);
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
                NewInterpreter::new(
                    memory.clone(),
                    bytecode,
                    interpreter_input,
                    inputs.is_static,
                    false,
                    ctx.cfg().spec().into(),
                    inputs.gas_limit,
                ),
                checkpoint,
                precompile,
                instructions,
                memory,
            )))
        }
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame(
        ctx: &mut CTX,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: &CreateInputs,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        let spec = ctx.cfg().spec().into();
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
        if spec.is_enabled_in(PRAGUE_EOF) && inputs.init_code.starts_with(&EOF_MAGIC_BYTES) {
            return return_error(InstructionResult::CreateInitCodeStartingEF00);
        }

        // Fetch balance of caller.
        let caller_balance = ctx
            .journal()
            .load_account(inputs.caller)?
            .map(|a| a.info.balance);

        // Check if caller has enough balance to send to the created contract.
        if caller_balance.data < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = ctx.journal().inc_account_nonce(inputs.caller)? {
            old_nonce = nonce - 1;
        } else {
            return return_error(InstructionResult::Return);
        }

        // Create address
        // TODO incorporating code hash inside interpreter. It was a request by foundry.
        let mut _init_code_hash = B256::ZERO;
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => {
                _init_code_hash = keccak256(&inputs.init_code);
                inputs.caller.create2(salt.to_be_bytes(), _init_code_hash)
            }
        };

        // created address is not allowed to be a precompile.
        // TODO add precompile check
        // if self.precompiles.contains(&created_address) {
        //     return return_error(InstructionResult::CreateCollision);
        // }

        // warm load account.
        ctx.journal().load_account(created_address)?;

        // create account, transfer funds and make the journal checkpoint.
        let Some(checkpoint) = ctx.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) else {
            return return_error(InstructionResult::CreateCollision);
        };

        let bytecode = Bytecode::new_legacy(inputs.init_code.clone()).into_analyzed();

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input: Bytes::new(),
            call_value: inputs.value,
        };

        Ok(FrameOrResultGen::Frame(Self::new(
            FrameData::Create(CreateFrame { created_address }),
            NewInterpreter::new(
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
        ctx: &mut CTX,
        depth: usize,
        memory: Rc<RefCell<SharedMemory>>,
        inputs: &EOFCreateInputs,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        let spec = ctx.cfg().spec().into();
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
                // decode eof and init code.
                // TODO handle inc_nonce handling more gracefully.
                let Ok((eof, input)) = Eof::decode_dangling(initdata.clone()) else {
                    ctx.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                };

                if eof.validate().is_err() {
                    // TODO (EOF) new error type.
                    ctx.journal().inc_account_nonce(inputs.caller)?;
                    return return_error(InstructionResult::InvalidEOFInitCode);
                }

                // Use nonce from tx to calculate address.
                let tx = ctx.tx().common_fields();
                let create_address = tx.caller().create(tx.nonce());

                (input, eof, Some(create_address))
            }
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let caller_balance = ctx
            .journal()
            .load_account(inputs.caller)?
            .map(|a| a.info.balance);

        // Check if caller has enough balance to send to the created contract.
        if caller_balance.data < inputs.value {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let Some(nonce) = ctx.journal().inc_account_nonce(inputs.caller)? else {
            // can't happen on mainnet.
            return return_error(InstructionResult::Return);
        };
        let old_nonce = nonce - 1;

        let created_address = created_address.unwrap_or_else(|| inputs.caller.create(old_nonce));

        // created address is not allowed to be a precompile.
        // TODO check precompile
        // if self.precompiles.contains(&created_address) {
        //     return return_error(InstructionResult::CreateCollision);
        // }

        // Load account so it needs to be marked as warm for access list.
        ctx.journal().load_account(created_address)?;

        // create account, transfer funds and make the journal checkpoint.
        let Some(checkpoint) = ctx.journal().create_account_checkpoint(
            inputs.caller,
            created_address,
            inputs.value,
            spec,
        ) else {
            return return_error(InstructionResult::CreateCollision);
        };

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller,
            input,
            call_value: inputs.value,
        };

        Ok(FrameOrResultGen::Frame(Self::new(
            FrameData::Create(CreateFrame { created_address }),
            NewInterpreter::new(
                memory.clone(),
                Bytecode::Eof(Arc::new(initcode)),
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

    pub fn init_with_context(
        depth: usize,
        frame_init: FrameInput,
        memory: Rc<RefCell<SharedMemory>>,
        precompile: PRECOMPILE,
        instructions: INSTRUCTION,
        ctx: &mut CTX,
    ) -> Result<FrameOrResultGen<Self, FrameResult>, ERROR> {
        match frame_init {
            FrameInput::Call(inputs) => {
                Self::make_call_frame(ctx, depth, memory, &inputs, precompile, instructions)
            }
            FrameInput::Create(inputs) => {
                Self::make_create_frame(ctx, depth, memory, &inputs, precompile, instructions)
            }
            FrameInput::EOFCreate(inputs) => {
                Self::make_eofcreate_frame(ctx, depth, memory, &inputs, precompile, instructions)
            }
        }
    }
}

impl<CTX, ERROR, PRECOMPILE, INSTRUCTION> Frame
    for EthFrame<CTX, ERROR, EthInterpreter<()>, PRECOMPILE, INSTRUCTION>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + Host,
    ERROR: From<JournalStateGetterDBError<CTX>> + From<PrecompileErrors>,
    PRECOMPILE: PrecompileProvider<Context = CTX, Error = ERROR>,
    INSTRUCTION: InstructionProvider<WIRE = EthInterpreter<()>, Host = CTX>,
{
    type Context = CTX;
    type Error = ERROR;
    type FrameInit = FrameInput;
    type FrameResult = FrameResult;

    fn init_first(
        ctx: &mut Self::Context,
        frame_input: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        let memory = Rc::new(RefCell::new(SharedMemory::new()));
        let precompiles = PRECOMPILE::new(ctx);
        let instructions = INSTRUCTION::new(ctx);

        // load precompiles addresses as warm.
        for address in precompiles.warm_addresses() {
            ctx.journal().warm_account(address);
        }

        memory.borrow_mut().new_context();
        Self::init_with_context(0, frame_input, memory, precompiles, instructions, ctx)
    }

    fn init(
        &self,
        ctx: &mut CTX,
        frame_init: Self::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.memory.borrow_mut().new_context();
        Self::init_with_context(
            self.depth + 1,
            frame_init,
            self.memory.clone(),
            self.precompiles.clone(),
            self.instructions.clone(),
            ctx,
        )
    }

    fn run(
        &mut self,
        //instructions: &InstructionTables<'_, Context<EvmWiringT>>,
        ctx: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        let spec = ctx.cfg().spec().into();

        // run interpreter
        let next_action = self.interpreter.run(self.instructions.table(), ctx);

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
                // revert changes or not.
                if interpreter_result.result.is_ok() {
                    ctx.journal().checkpoint_commit();
                } else {
                    ctx.journal().checkpoint_revert(self.checkpoint);
                }
                FrameOrResultGen::Result(FrameResult::Call(CallOutcome::new(
                    interpreter_result,
                    frame.return_memory_range.clone(),
                )))
            }
            FrameData::Create(frame) => {
                let max_code_size = ctx.cfg().max_code_size();
                return_create(
                    ctx.journal(),
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
                let max_code_size = ctx.cfg().max_code_size();
                return_eofcreate(
                    ctx.journal(),
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
        ctx: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error> {
        self.memory.borrow_mut().free_context();
        ctx.take_error()?;

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
                    } else {
                        if ins_result.is_ok() {
                            U256::from(1)
                        } else {
                            U256::ZERO
                        }
                    }
                };
                interpreter.stack.push(item);

                // return unspend gas.
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

                let item = if instruction_result == InstructionResult::ReturnContract {
                    outcome.address.expect("EOF Address").into_word().into()
                } else {
                    U256::ZERO
                };
                interpreter.stack.push(item);

                assert_eq!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let gas = interpreter.control.gas();
                if instruction_result.is_ok_or_revert() {
                    gas.erase_cost(outcome.gas().remaining());
                }

                if instruction_result.is_ok() {
                    gas.record_refund(outcome.gas().refunded());
                }
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

                assert_eq!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                if instruction_result.is_ok_or_revert() {
                    interpreter
                        .control
                        .gas()
                        .erase_cost(outcome.gas().remaining());
                }

                if instruction_result.is_ok() {
                    interpreter
                        .control
                        .gas()
                        .record_refund(outcome.gas().refunded());
                    interpreter
                        .stack
                        .push(outcome.address.expect("EOF Address").into_word().into());
                } else {
                    interpreter.stack.push(U256::ZERO);
                }
            }
        }

        Ok(())
    }
}

pub fn return_create<Journal: JournaledState>(
    journal: &mut Journal,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    address: Address,
    max_code_size: usize,
    spec_id: SpecId,
) {
    // if return is not ok revert and return.
    if !interpreter_result.result.is_ok() {
        journal.checkpoint_revert(checkpoint);
        return;
    }
    // Host error if present on execution
    // if ok, check contract creation limit and calculate gas deduction on output len.
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
        // record code deposit gas cost and check if we are out of gas.
        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
        // final gas fee for adding the contract code to the state, the contract
        //  creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
        if spec_id.is_enabled_in(HOMESTEAD) {
            journal.checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        } else {
            interpreter_result.output = Bytes::new();
        }
    }
    // if we have enough gas we can commit changes.
    journal.checkpoint_commit();

    // Do analysis of bytecode straight away.
    let bytecode = Bytecode::new_legacy(interpreter_result.output.clone()).into_analyzed();

    // set code
    journal.set_code(address, bytecode);

    interpreter_result.result = InstructionResult::Return;
}

pub fn return_eofcreate<Journal: JournaledState>(
    journal: &mut Journal,
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

    // deduct gas for code deployment.
    let gas_for_code = interpreter_result.output.len() as u64 * gas::CODEDEPOSIT;
    if !interpreter_result.gas.record_cost(gas_for_code) {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::OutOfGas;
        return;
    }

    // commit changes reduces depth by -1.
    journal.checkpoint_commit();

    // decode bytecode has a performance hit, but it has reasonable restrains.
    let bytecode = Eof::decode(interpreter_result.output.clone()).expect("Eof is already verified");

    // eof bytecode is going to be hashed.
    journal.set_code(address, Bytecode::Eof(Arc::new(bytecode)));
}
