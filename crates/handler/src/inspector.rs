use crate::{
    execution,
    handler::{EthHandler, EvmTypesTrait},
    EthFrame, FrameResult,
};
use auto_impl::auto_impl;
use context::{Cfg, ContextTrait, JournalEntry, JournaledState};
use context_interface::{result::ResultAndState, Database, Transaction};
use handler_interface::{Frame, FrameOrResult, ItemOrResult};
use interpreter::{
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, InitialAndFloorGas,
    Interpreter, InterpreterTypes,
};
use primitives::{Address, Log, U256};
use state::EvmState;

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<CTX, INTR: InterpreterTypes> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [revm::interpreter::InstructionResult::Continue] then the execution of the interpreter
    /// is skipped.
    #[inline]
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    #[inline]
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// Setting `interp.instruction_result` to anything other than [revm::interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: &Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [revm::interpreter::InstructionResult::Continue] overrides the result of the call.
    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a call to a contract has concluded.
    ///
    /// The returned [CallOutcome] is used as the result of the call.
    ///
    /// This allows the inspector to modify the given `result` before returning it.
    #[inline]
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract is about to be created.
    ///
    /// If this returns `Some` then the [CreateOutcome] is used to override the result of the creation.
    ///
    /// If this returns `None` then the creation proceeds as normal.
    #[inline]
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_gas,
    /// address, out)`) will alter the result of the create.
    #[inline]
    fn create_end(
        &mut self,
        context: &mut CTX,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when EOF creating is called.
    ///
    /// This can happen from create TX or from EOFCREATE opcode.
    fn eofcreate(
        &mut self,
        context: &mut CTX,
        inputs: &mut EOFCreateInputs,
    ) -> Option<CreateOutcome> {
        let _ = context;
        let _ = inputs;
        None
    }

    /// Called when eof creating has ended.
    fn eofcreate_end(
        &mut self,
        context: &mut CTX,
        inputs: &EOFCreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        let _ = context;
        let _ = inputs;
        let _ = outcome;
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        let _ = contract;
        let _ = target;
        let _ = value;
    }
}

#[auto_impl(&mut, Box)]
pub trait JournalExt {
    fn logs(&self) -> &[Log];

    fn last_journal(&self) -> &[JournalEntry];

    fn evm_state(&self) -> &EvmState;

    fn evm_state_mut(&mut self) -> &mut EvmState;
}

impl<DB: Database> JournalExt for JournaledState<DB> {
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    fn last_journal(&self) -> &[JournalEntry] {
        self.journal.last().expect("Journal is never empty")
    }

    fn evm_state(&self) -> &EvmState {
        &self.state
    }

    fn evm_state_mut(&mut self) -> &mut EvmState {
        &mut self.state
    }
}

pub trait FrameInterpreterGetter {
    type IT: InterpreterTypes;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;
}

impl<CTX, ERROR, IT> FrameInterpreterGetter for EthFrame<CTX, ERROR, IT>
where
    IT: InterpreterTypes,
{
    type IT = IT;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }
}

pub trait EthInspectorHandler: EthHandler
where
    Self::Evm: EvmTypesTrait<
        Inspector: Inspector<<<Self as EthHandler>::Evm as EvmTypesTrait>::Context, Self::IT>,
    >,
    Self::Frame: FrameInterpreterGetter<IT = Self::IT>,
{
    type IT: InterpreterTypes;

    fn enable_inspection(&mut self, enable: bool);

    fn inspect_run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        // enable instruction inspection
        self.enable_inspection(true);
        let exec_result = self.inspect_execution(evm, &init_and_floor_gas);
        // disable instruction inspection
        self.enable_inspection(false);
        self.post_execution(evm, exec_result?, init_and_floor_gas, eip7702_refund)
    }

    fn inspect_execution(
        &mut self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = evm.ctx().tx().gas_limit() - init_and_floor_gas.initial_gas;

        // Create first frame action
        let first_frame = self.inspect_create_first_frame(evm, gas_limit)?;
        let mut frame_result = match first_frame {
            ItemOrResult::Item(frame) => self.inspect_run_exec_loop(evm, frame)?,
            ItemOrResult::Result(result) => result,
        };

        self.inspect_last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

    /* EXECUTION */
    fn inspect_create_first_frame(
        &mut self,
        evm: &mut Self::Evm,
        gas_limit: u64,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let ctx = evm.ctx_ref();
        let init_frame = execution::create_init_frame(ctx.tx(), ctx.cfg().spec().into(), gas_limit);
        self.inspect_frame_init_first(evm, init_frame)
    }

    /* FRAMES */

    fn inspect_frame_init_first(
        &mut self,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        // TODO frame_start
        //if let Some(output) = evm.frame_start(&mut frame_input) {
        //    return Ok(ItemOrResult::Result(output));
        //}
        let mut ret: Result<
            ItemOrResult<<Self as EthHandler>::Frame, FrameResult>,
            <Self as EthHandler>::Error,
        > = self.frame_init_first(evm, frame_input);

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            inspector.initialize_interp(frame.interpreter(), context);
        }
        ret
    }

    fn inspect_frame_init(
        &mut self,
        frame: &Self::Frame,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        // TODO frame_start
        // if let Some(output) = context.frame_start(&mut frame_input) {
        //     return Ok(ItemOrResult::Result(output));
        // }
        let mut ret = self.frame_init(frame, evm, frame_input);

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            inspector.initialize_interp(frame.interpreter(), context);
        }
        ret
    }

    fn inspect_frame_return_result(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        // TODO
        // context.frame_end(&mut result);
        self.frame_return_result(frame, evm, result)
    }

    fn inspect_run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = self.frame_call(frame, evm)?;

            let result = match call_or_result {
                ItemOrResult::Item(init) => {
                    match self.inspect_frame_init(frame, evm, init)? {
                        ItemOrResult::Item(new_frame) => {
                            frame_stack.push(new_frame);
                            continue;
                        }
                        // Dont pop the frame as new frame was not created.
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => {
                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                return Ok(result);
            };
            self.inspect_frame_return_result(frame, evm, result)?;
        }
    }

    fn inspect_last_frame_result(
        &self,
        evm: &mut Self::Evm,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        // TODO
        //context.frame_end(frame_result);
        self.last_frame_result(evm, frame_result)
    }
}

// TODO INSTRUCTIONS FRAME START AND END

// fn frame_start(&mut self, frame_input: &mut FrameInput) -> Option<FrameResult> {
//     let insp = self.inspector.get_inspector();
//     let context = &mut self.inner;
//     match frame_input {
//         FrameInput::Call(i) => {
//             if let Some(output) = insp.call(context, i) {
//                 return Some(FrameResult::Call(output));
//             }
//         }
//         FrameInput::Create(i) => {
//             if let Some(output) = insp.create(context, i) {
//                 return Some(FrameResult::Create(output));
//             }
//         }
//         FrameInput::EOFCreate(i) => {
//             if let Some(output) = insp.eofcreate(context, i) {
//                 return Some(FrameResult::EOFCreate(output));
//             }
//         }
//     }
//     self.frame_input_stack.push(frame_input.clone());
//     None
// }

// fn frame_end(&mut self, frame_output: &mut FrameResult) {
//     let insp = self.inspector.get_inspector();
//     let context = &mut self.inner;
//     let Some(frame_input) = self.frame_input_stack.pop() else {
//         // case where call returns immediately will not push to call stack.
//         return;
//     };
//     match frame_output {
//         FrameResult::Call(outcome) => {
//             let FrameInput::Call(i) = frame_input else {
//                 panic!("FrameInput::Call expected");
//             };
//             insp.call_end(context, &i, outcome);
//         }
//         FrameResult::Create(outcome) => {
//             let FrameInput::Create(i) = frame_input else {
//                 panic!("FrameInput::Create expected");
//             };
//             insp.create_end(context, &i, outcome);
//         }
//         FrameResult::EOFCreate(outcome) => {
//             let FrameInput::EOFCreate(i) = frame_input else {
//                 panic!("FrameInput::EofCreate expected");
//             };
//             insp.eofcreate_end(context, &i, outcome);
//         }
//     }
// }

// fn inspector_selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
//     self.inspector
//         .get_inspector()
//         .selfdestruct(contract, target, value)
// }

// INSTRUCTIONS FOR INSPECTOR

// pub struct InspectorInstructionExecutor<WIRE: InterpreterTypes, HOST> {
//     instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
// }

// impl<WIRE, HOST> Clone for InspectorInstructionExecutor<WIRE, HOST>
// where
//     WIRE: InterpreterTypes,
// {
//     fn clone(&self) -> Self {
//         Self {
//             instruction_table: self.instruction_table.clone(),
//         }
//     }
// }

// impl<WIRE, HOST> InspectorInstructionExecutor<WIRE, HOST>
// where
//     WIRE: InterpreterTypes,
//     HOST: Host + JournalExtGetter + JournalGetter + InspectorCtx<IT = WIRE>,
// {
//     pub fn (base_table: InstructionTable<WIRE, HOST>) -> Self {
//         let mut table: [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256] =
//             unsafe { MaybeUninit::uninit().assume_init() };

//         for (i, element) in table.iter_mut().enumerate() {
//             let function: InspectorInstruction<WIRE, HOST> = InspectorInstruction {
//                 instruction: base_table[i],
//             };
//             *element = MaybeUninit::new(function);
//         }

//         let mut table = unsafe {
//             core::mem::transmute::<
//                 [MaybeUninit<InspectorInstruction<WIRE, HOST>>; 256],
//                 [InspectorInstruction<WIRE, HOST>; 256],
//             >(table)
//         };

//         // Inspector log wrapper
//         fn inspector_log<CTX: Host + JournalExtGetter + InspectorCtx>(
//             interpreter: &mut Interpreter<<CTX as InspectorCtx>::IT>,
//             context: &mut CTX,
//             prev: Instruction<<CTX as InspectorCtx>::IT, CTX>,
//         ) {
//             prev(interpreter, context);

//             if interpreter.control.instruction_result() == InstructionResult::Continue {
//                 let last_log = context.journal_ext().logs().last().unwrap().clone();
//                 context.inspector_log(interpreter, &last_log);
//             }
//         }

//         /* LOG and Selfdestruct instructions */
//         table[OpCode::LOG0.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 inspector_log(interp, context, log::<0, HOST>);
//             },
//         };
//         table[OpCode::LOG1.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 inspector_log(interp, context, log::<1, HOST>);
//             },
//         };
//         table[OpCode::LOG2.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 inspector_log(interp, context, log::<2, HOST>);
//             },
//         };
//         table[OpCode::LOG3.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 inspector_log(interp, context, log::<3, HOST>);
//             },
//         };
//         table[OpCode::LOG4.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 inspector_log(interp, context, log::<4, HOST>);
//             },
//         };

//         table[OpCode::SELFDESTRUCT.as_usize()] = InspectorInstruction {
//             instruction: |interp, context| {
//                 selfdestruct::<WIRE, HOST>(interp, context);
//                 if interp.control.instruction_result() == InstructionResult::SelfDestruct {
//                     match context.journal_ext().last_journal().last() {
//                         Some(JournalEntry::AccountDestroyed {
//                             address,
//                             target,
//                             had_balance,
//                             ..
//                         }) => {
//                             context.inspector_selfdestruct(*address, *target, *had_balance);
//                         }
//                         Some(JournalEntry::BalanceTransfer {
//                             from, to, balance, ..
//                         }) => {
//                             context.inspector_selfdestruct(*from, *to, *balance);
//                         }
//                         _ => {}
//                     }
//                 }
//             },
//         };

//         Self {
//             instruction_table: Rc::new(table),
//         }
//     }
// }
