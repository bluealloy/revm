use crate::{
    execution,
    handler::{EthHandler, EvmTrait},
    EthFrame, Frame, FrameOrResult, FrameResult, ItemOrResult,
};
use auto_impl::auto_impl;
use context::{Cfg, JournalEntry, JournaledState};
use context_interface::{result::ResultAndState, ContextTrait, Database, Transaction};
use interpreter::{
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, EOFCreateInputs, FrameInput,
    InitialAndFloorGas, Interpreter, InterpreterTypes,
};
use primitives::{Address, Log, U256};
use state::EvmState;
use std::{vec, vec::Vec};

/// EVM [Interpreter] callbacks.
#[auto_impl(&mut, Box)]
pub trait Inspector<CTX, INTR: InterpreterTypes> {
    /// Called before the interpreter is initialized.
    ///
    /// If `interp.instruction_result` is set to anything other than [interpreter::InstructionResult::Continue] then the execution of the interpreter
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
    /// Setting `interp.instruction_result` to anything other than [interpreter::InstructionResult::Continue] alters the execution
    /// of the interpreter.
    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        let _ = interp;
        let _ = context;
    }

    /// Called when a log is emitted.
    #[inline]
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: Log) {
        let _ = interp;
        let _ = context;
        let _ = log;
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [interpreter::InstructionResult::Continue] overrides the result of the call.
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
    #[inline]
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    #[inline]
    fn last_journal(&self) -> &[JournalEntry] {
        self.journal.last().expect("Journal is never empty")
    }

    #[inline]
    fn evm_state(&self) -> &EvmState {
        &self.state
    }

    #[inline]
    fn evm_state_mut(&mut self) -> &mut EvmState {
        &mut self.state
    }
}

pub trait InspectorFrame {
    type IT: InterpreterTypes;
    type FrameInput;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    fn frame_input(&self) -> &FrameInput;
}

impl<CTX, ERROR, IT> InspectorFrame for EthFrame<CTX, ERROR, IT>
where
    IT: InterpreterTypes,
{
    type IT = IT;
    type FrameInput = FrameInput;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }

    fn frame_input(&self) -> &FrameInput {
        &self.input
    }
}

pub trait EthInspectorHandler: EthHandler
where
    Self::Evm:
        EvmTrait<Inspector: Inspector<<<Self as EthHandler>::Evm as EvmTrait>::Context, Self::IT>>,
    Self::Frame: InspectorFrame<IT = Self::IT>,
{
    type IT: InterpreterTypes;

    fn inspect_run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        // enable instruction inspection
        evm.enable_inspection(true);
        let exec_result = self.inspect_execution(evm, &init_and_floor_gas);
        // disable instruction inspection
        evm.enable_inspection(false);
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

        self.last_frame_result(evm, &mut frame_result)?;
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
        mut frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let (ctx, inspector) = evm.ctx_inspector();
        if let Some(output) = frame_start(ctx, inspector, &mut frame_input) {
            return Ok(ItemOrResult::Result(output));
        }
        let mut ret = self.frame_init_first(evm, frame_input.clone());

        // only if new frame is created call initialize_interp hook.
        if let Ok(ItemOrResult::Item(frame)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            inspector.initialize_interp(frame.interpreter(), context);
        } else if let Ok(ItemOrResult::Result(result)) = &mut ret {
            let (context, inspector) = evm.ctx_inspector();
            frame_end(context, inspector, &frame_input, result);
        }
        ret
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
                ItemOrResult::Item(mut init) => {
                    let (context, inspector) = evm.ctx_inspector();
                    if let Some(output) = frame_start(context, inspector, &mut init) {
                        output
                    } else {
                        match self.frame_init(frame, evm, init)? {
                            ItemOrResult::Item(mut new_frame) => {
                                // only if new frame is created call initialize_interp hook.
                                let (context, inspector) = evm.ctx_inspector();
                                inspector.initialize_interp(new_frame.interpreter(), context);
                                frame_stack.push(new_frame);
                                continue;
                            }
                            // Dont pop the frame as new frame was not created.
                            ItemOrResult::Result(mut result) => {
                                let (context, inspector) = evm.ctx_inspector();
                                frame_end(context, inspector, frame.frame_input(), &mut result);
                                result
                            }
                        }
                    }
                }
                ItemOrResult::Result(mut result) => {
                    let (context, inspector) = evm.ctx_inspector();
                    frame_end(context, inspector, frame.frame_input(), &mut result);

                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                return Ok(result);
            };

            self.frame_return_result(frame, evm, result)?;
        }
    }
}

fn frame_start<CTX, INTR: InterpreterTypes>(
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, INTR>,
    frame_input: &mut FrameInput,
) -> Option<FrameResult> {
    match frame_input {
        FrameInput::Call(i) => {
            if let Some(output) = inspector.call(context, i) {
                return Some(FrameResult::Call(output));
            }
        }
        FrameInput::Create(i) => {
            if let Some(output) = inspector.create(context, i) {
                return Some(FrameResult::Create(output));
            }
        }
        FrameInput::EOFCreate(i) => {
            if let Some(output) = inspector.eofcreate(context, i) {
                return Some(FrameResult::EOFCreate(output));
            }
        }
    }
    None
}

fn frame_end<CTX, INTR: InterpreterTypes>(
    context: &mut CTX,
    inspector: &mut impl Inspector<CTX, INTR>,
    frame_input: &FrameInput,
    frame_output: &mut FrameResult,
) {
    match frame_output {
        FrameResult::Call(outcome) => {
            let FrameInput::Call(i) = frame_input else {
                panic!("FrameInput::Call expected");
            };
            inspector.call_end(context, i, outcome);
        }
        FrameResult::Create(outcome) => {
            let FrameInput::Create(i) = frame_input else {
                panic!("FrameInput::Create expected");
            };
            inspector.create_end(context, i, outcome);
        }
        FrameResult::EOFCreate(outcome) => {
            let FrameInput::EOFCreate(i) = frame_input else {
                panic!("FrameInput::EofCreate expected");
            };
            inspector.eofcreate_end(context, i, outcome);
        }
    }
}

// INSTRUCTIONS FOR INSPECTOR

// pub struct InspectorInstructionProvider<WIRE: InterpreterTypes, HOST> {
//     instruction_table: Rc<[InspectorInstruction<WIRE, HOST>; 256]>,
// }

// impl<WIRE, HOST> Clone for InspectorInstructionProvider<WIRE, HOST>
// where
//     WIRE: InterpreterTypes,
// {
//     fn clone(&self) -> Self {
//         Self {
//             instruction_table: self.instruction_table.clone(),
//         }
//     }
// }

// impl<WIRE, HOST> InspectorInstructionProvider<WIRE, HOST>
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
