use crate::handler::{wires::Frame, FrameOrResultGen};
use bytecode::{Eof, EOF_MAGIC_BYTES};
use context::{
    BlockGetter, CfgGetter, Context, ErrorGetter, Frame as FrameData, FrameOrResult, FrameResult,
    JournalCheckpoint, JournalStateGetter, JournalStateGetterDBError, TransactionGetter,
};
use core::{cell::RefCell, mem, ops::DerefMut};
use interpreter::{
    gas, table::InstructionTables, CallInputs, CallOutcome, CallValue, Contract, CreateInputs,
    CreateOutcome, CreateScheme, EOFCreateInputs, EOFCreateKind, Gas, InstructionResult,
    Interpreter, InterpreterAction, InterpreterResult, NewFrameAction, SharedMemory,
    EMPTY_SHARED_MEMORY,
};
use primitives::{keccak256, Address, Bytes, B256};
use specification::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, PRAGUE_EOF, SPURIOUS_DRAGON},
};
use state::Bytecode;
use std::{rc::Rc, sync::Arc};
use wiring::{
    journaled_state::JournaledState,
    result::{EVMError, EVMErrorWiring, InvalidTransaction},
    Cfg, EvmWiring, Transaction,
};

pub struct EthFrame<CTX, ERROR> {
    _phantom: std::marker::PhantomData<(CTX, ERROR)>,
    data: FrameData,
    // TODO include this
    depth: usize,
    spec_id: SpecId,
    // This is worth making as a generic type FrameSharedContext.
    shared_memory: Rc<RefCell<SharedMemory>>,
}

impl<CTX, ERROR> EthFrame<CTX, ERROR>
where
    CTX: JournalStateGetter,
{
    pub fn new(data: FrameData, shared_memory: Rc<RefCell<SharedMemory>>, spec_id: SpecId) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            data,
            depth: 0,
            spec_id,
            shared_memory,
        }
    }
}

impl<CTX, ERROR> EthFrame<CTX, ERROR>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter<Journal: JournaledState<Checkpoint = JournalCheckpoint>>
        + CfgGetter,
    ERROR: From<JournalStateGetterDBError<CTX>>,
{
    /// Make call frame
    #[inline]
    pub fn make_call_frame(ctx: &mut CTX, inputs: &CallInputs) -> Result<FrameOrResult, ERROR> {
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
        // TODO
        // if self.journal().depth() > CALL_STACK_LIMIT {
        //     return return_result(InstructionResult::CallTooDeep);
        // }

        // Make account warm and loaded
        // TODO
        // let _ = ctx
        //     .journal()
        //     .load_account_delegated(inputs.bytecode_address)
        //     .map_err(EVMError::Database)?;

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
                if let Some(result) =
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
        return return_result(InstructionResult::CreateCollision);
        /*
        if let Some(result) = self.call_precompile(&inputs.bytecode_address, &inputs.input, gas)? {
            if matches!(result.result, return_ok!()) {
                self.journaled_state.checkpoint_commit();
            } else {
                self.journaled_state.checkpoint_revert(checkpoint);
            }
            Ok(FrameOrResult::new_call_result(
                result,
                inputs.return_memory_offset.clone(),
            ))
        } else {
            let account = self
                .inner
                .journaled_state
                .load_code(inputs.bytecode_address)
                .map_err(EVMError::Database)?;

            let code_hash = account.info.code_hash();
            let mut bytecode = account.info.code.clone().unwrap_or_default();

            // ExtDelegateCall is not allowed to call non-EOF contracts.
            if inputs.scheme.is_ext_delegate_call()
                && !bytecode.bytes_slice().starts_with(&EOF_MAGIC_BYTES)
            {
                return return_result(InstructionResult::InvalidExtDelegateCallTarget);
            }

            if bytecode.is_empty() {
                self.journaled_state.checkpoint_commit();
                return return_result(InstructionResult::Stop);
            }

            if let Bytecode::Eip7702(eip7702_bytecode) = bytecode {
                bytecode = self
                    .inner
                    .journaled_state
                    .load_code(eip7702_bytecode.delegated_address)
                    .map_err(EVMError::Database)?
                    .info
                    .code
                    .clone()
                    .unwrap_or_default();
            }

            let contract =
                Contract::new_with_context(inputs.input.clone(), bytecode, Some(code_hash), inputs);
            // Create interpreter and executes call and push new CallStackFrame.
            Ok(FrameOrResult::new_call_frame(
                inputs.return_memory_offset.clone(),
                checkpoint,
                Interpreter::new(contract, gas.limit(), inputs.is_static),
            ))

        }
        */
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame(
        ctx: &mut CTX,
        spec_id: SpecId,
        inputs: &CreateInputs,
    ) -> Result<FrameOrResult, ERROR> {
        let return_error = |e| {
            Ok(FrameOrResult::new_create_result(
                InterpreterResult {
                    result: e,
                    gas: Gas::new(inputs.gas_limit),
                    output: Bytes::new(),
                },
                None,
            ))
        };

        // Check depth
        // TODO add depth check
        // if ctx.journal().depth() > CALL_STACK_LIMIT {
        //     return return_error(InstructionResult::CallTooDeep);
        // }

        // Prague EOF
        if spec_id.is_enabled_in(PRAGUE_EOF) && inputs.init_code.starts_with(&EOF_MAGIC_BYTES) {
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
        let mut init_code_hash = B256::ZERO;
        let created_address = match inputs.scheme {
            CreateScheme::Create => inputs.caller.create(old_nonce),
            CreateScheme::Create2 { salt } => {
                init_code_hash = keccak256(&inputs.init_code);
                inputs.caller.create2(salt.to_be_bytes(), init_code_hash)
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
        // TODO add create account checkpoint
        // let checkpoint = match ctx.journal().create_account_checkpoint(
        //     inputs.caller,
        //     created_address,
        //     inputs.value,
        //     spec_id,
        // ) {
        //     Ok(checkpoint) => checkpoint,
        //     Err(e) => {
        //         return return_error(e);
        //     }
        // };
        //let checkpoint = JournalCheckpoint

        let bytecode = Bytecode::new_legacy(inputs.init_code.clone());

        let contract = Contract::new(
            Bytes::new(),
            bytecode,
            Some(init_code_hash),
            created_address,
            None,
            inputs.caller,
            inputs.value,
        );

        // Ok(FrameOrResult::new_create_frame(
        //     created_address,
        //     checkpoint,
        //     Interpreter::new(contract, inputs.gas_limit, false),
        // ))
        todo!()
    }

    /// Make create frame.
    #[inline]
    pub fn make_eofcreate_frame(
        ctx: &mut CTX,
        spec_id: SpecId,
        inputs: &EOFCreateInputs,
    ) -> Result<FrameOrResult, ERROR> {
        let return_error = |e| {
            Ok(FrameOrResult::new_eofcreate_result(
                InterpreterResult {
                    result: e,
                    gas: Gas::new(inputs.gas_limit),
                    output: Bytes::new(),
                },
                None,
            ))
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
        // TODO check depth
        // if self.journaled_state.depth() > CALL_STACK_LIMIT {
        //     return return_error(InstructionResult::CallTooDeep);
        // }

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
        todo!("create account checkpoint");
        // let checkpoint = match self.journal().create_account_checkpoint(
        //     inputs.caller,
        //     created_address,
        //     inputs.value,
        //     spec_id,
        // ) {
        //     Ok(checkpoint) => checkpoint,
        //     Err(e) => {
        //         return return_error(e);
        //     }
        // };

        // let contract = Contract::new(
        //     input.clone(),
        //     // fine to clone as it is Bytes.
        //     Bytecode::Eof(Arc::new(initcode.clone())),
        //     None,
        //     created_address,
        //     None,
        //     inputs.caller,
        //     inputs.value,
        // );

        // let mut interpreter = Interpreter::new(contract, inputs.gas_limit, false);
        // // EOF init will enable RETURNCONTRACT opcode.
        // interpreter.set_is_eof_init();

        // Ok(FrameOrResult::new_eofcreate_frame(
        //     created_address,
        //     checkpoint,
        //     interpreter,
        // ))
    }

    pub fn init_with_context(
        frame_init: NewFrameAction,
        spec_id: SpecId,
        shared_memory: Rc<RefCell<SharedMemory>>,
        ctx: &mut CTX,
    ) -> Result<FrameOrResultGen<Self, <Self as Frame>::FrameResult>, ERROR> {
        // let frame_or_result = match frame_init {
        //     NewFrameAction::Call(inputs) => Self::make_call_frame(ctx, &inputs)?,
        //     NewFrameAction::Create(inputs) => Self::make_create_frame(ctx, spec_id, &inputs)?,
        //     NewFrameAction::EOFCreate(inputs) => Self::make_eofcreate_frame(ctx, spec_id, &inputs)?,
        // };
        // let ret = match frame_or_result {
        //     FrameOrResult::Frame(frame) => {
        //         FrameOrResultGen::Frame(EthFrame::new(frame, shared_memory, spec_id))
        //     }
        //     FrameOrResult::Result(result) => FrameOrResultGen::Result(result),
        // };
        // Ok(ret)
        todo!()
    }
}

//spub trait HostTemp: TransactionGetter + BlockGetter + JournalStateGetter {}

impl<CTX, ERROR> Frame for EthFrame<CTX, ERROR>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter<Journal: JournaledState<Checkpoint = JournalCheckpoint>>
        + CfgGetter,
    ERROR: From<JournalStateGetterDBError<CTX>>,
{
    type Context = CTX;

    type Error = ERROR;

    type FrameInit = NewFrameAction;

    type FrameResult = FrameResult;

    fn init(
        &self,
        frame_init: Self::FrameInit,
        ctx: &mut CTX,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.shared_memory.borrow_mut().new_context();
        let spec_id = self.spec_id;
        Self::init_with_context(frame_init, spec_id, self.shared_memory.clone(), ctx)
    }

    fn run(
        &mut self,
        //instructions: &InstructionTables<'_, Context<EvmWiringT>>,
        ctx: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        let interpreter = self.data.interpreter_mut();

        let memory = mem::replace(
            self.shared_memory.borrow_mut().deref_mut(),
            EMPTY_SHARED_MEMORY,
        );
        // TODO fix after instructions are implemented.
        // let next_action = match instructions {
        //     InstructionTables::Plain(table) => interpreter.run(memory, table, ctx),
        //     InstructionTables::Boxed(table) => interpreter.run(memory, table, ctx),
        // };
        let next_action = Default::default();
        // Take the shared memory back.
        *self.shared_memory.borrow_mut() = interpreter.take_memory();

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
                    ctx.journal().checkpoint_revert(frame.frame_data.checkpoint);
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
                    frame.frame_data.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                    max_code_size,
                    self.spec_id,
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
                    frame.frame_data.checkpoint,
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
        self.shared_memory.borrow_mut().free_context();
        ctx.take_error()?;

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                // return_call
                let mut shared_memory = self.shared_memory.borrow_mut();
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_call_outcome(&mut shared_memory, outcome);
            }
            FrameResult::Create(outcome) => {
                // return_create
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_create_outcome(outcome);
            }
            FrameResult::EOFCreate(outcome) => {
                self.data
                    .frame_data_mut()
                    .interpreter
                    .insert_eofcreate_outcome(outcome);
            }
        }

        Ok(())
    }
}

pub fn return_create<Journal: JournaledState>(
    journal: &mut Journal,
    checkpoint: Journal::Checkpoint,
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
    checkpoint: Journal::Checkpoint,
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
