use crate::handler::{wires::Frame, FrameOrResultGen};
use bytecode::Eof;
use context::{
    CfgGetter, Context, Frame as FrameData, FrameOrResult, FrameResult, JournalStateGetter,
};
use core::{cell::RefCell, mem, ops::DerefMut};
use interpreter::{
    gas, table::InstructionTables, CallOutcome, CreateOutcome, InstructionResult,
    InterpreterAction, InterpreterResult, NewFrameAction, SharedMemory, EMPTY_SHARED_MEMORY,
};
use primitives::{Address, Bytes};
use specification::hardfork::SpecId::{self, HOMESTEAD, LONDON, SPURIOUS_DRAGON};
use state::Bytecode;
use std::{rc::Rc, sync::Arc};
use wiring::{
    journaled_state::JournaledState,
    result::{EVMError, EVMErrorWiring},
    Cfg, EvmWiring,
};

pub struct EthFrame<EvmWiring, CTX> {
    _phantom: std::marker::PhantomData<(CTX, EvmWiring)>,
    data: FrameData,
    spec_id: SpecId,
    // This is worth making as a generic type FrameSharedContext.
    shared_memory: Rc<RefCell<SharedMemory>>,
}

impl<EvmWiringT: EvmWiring, CTX> EthFrame<EvmWiringT, CTX> {
    pub fn new(data: FrameData, shared_memory: Rc<RefCell<SharedMemory>>, spec_id: SpecId) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            data,
            spec_id,
            shared_memory,
        }
    }

    pub fn init_with_context(
        frame_init: NewFrameAction,
        spec_id: SpecId,
        shared_memory: Rc<RefCell<SharedMemory>>,
        ctx: &mut Context<EvmWiringT>,
    ) -> Result<FrameOrResultGen<Self, <Self as Frame>::FrameResult>, <Self as Frame>::Error> {
        let frame_or_result = match frame_init {
            NewFrameAction::Call(inputs) => ctx.evm.make_call_frame(&inputs)?,
            NewFrameAction::Create(inputs) => ctx.evm.make_create_frame(spec_id, &inputs)?,
            NewFrameAction::EOFCreate(inputs) => ctx.evm.make_eofcreate_frame(spec_id, &inputs)?,
        };
        let ret = match frame_or_result {
            FrameOrResult::Frame(frame) => {
                FrameOrResultGen::Frame(EthFrame::new(frame, shared_memory, spec_id))
            }
            FrameOrResult::Result(result) => FrameOrResultGen::Result(result),
        };
        Ok(ret)
    }
}

//spub trait HostTemp: TransactionGetter + BlockGetter + JournalStateGetter {}

impl<CTX, EvmWiringT: EvmWiring> Frame for EthFrame<EvmWiringT, CTX> {
    type Context = Context<EvmWiringT>;

    type Error = EVMErrorWiring<EvmWiringT>;

    type FrameInit = NewFrameAction;

    type FrameResult = FrameResult;

    fn init(
        &self,
        frame_init: Self::FrameInit,
        ctx: &mut Context<EvmWiringT>,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
        self.shared_memory.borrow_mut().new_context();
        let spec_id = self.spec_id;
        Self::init_with_context(frame_init, spec_id, self.shared_memory.clone(), ctx)
    }

    fn run(
        &mut self,
        instructions: &InstructionTables<'_, Context<EvmWiringT>>,
        ctx: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error> {
        let interpreter = self.data.interpreter_mut();

        let memory = mem::replace(
            self.shared_memory.borrow_mut().deref_mut(),
            EMPTY_SHARED_MEMORY,
        );
        let next_action = match instructions {
            InstructionTables::Plain(table) => interpreter.run(memory, table, ctx),
            InstructionTables::Boxed(table) => interpreter.run(memory, table, ctx),
        };
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
        ctx.evm.take_error().map_err(EVMError::Database)?;

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
