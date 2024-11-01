use super::EthFrame;
use crate::handler::{wires::Frame as FrameTrait, ExecutionWire, FrameOrResultGen};
use bytecode::EOF_MAGIC_BYTES;
use context::{
    BlockGetter, CfgGetter, ErrorGetter, JournalStateGetter, JournalStateGetterDBError,
    TransactionGetter,
};
use core::cell::RefCell;
use interpreter::{
    interpreter::EthInterpreter, return_ok, return_revert, CallInputs, CallScheme, CallValue,
    CreateInputs, CreateScheme, EOFCreateInputs, EOFCreateKind, Gas, NewFrameAction, SharedMemory,
};
use primitives::TxKind;
use specification::hardfork::SpecId;
use std::{boxed::Box, rc::Rc};
use wiring::{journaled_state::JournaledState, result::InvalidTransaction, Transaction};

/// TODO EvmWiringT is temporary, replace it with getter traits.
pub struct EthExecution<CTX, ERROR> {
    spec_id: SpecId,
    _phantom: std::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> ExecutionWire for EthExecution<CTX, ERROR>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter
        + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalStateGetterDBError<CTX>>,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = EthFrame<CTX, EthInterpreter<()>, ERROR>;
    type ExecResult = <Self::Frame as FrameTrait>::FrameResult;

    fn init_first_frame(
        &self,
        context: &mut Self::Context,
        gas_limit: u64,
    ) -> Result<FrameOrResultGen<Self::Frame, <Self::Frame as FrameTrait>::FrameResult>, Self::Error>
    {
        // Make new frame action.
        let spec_id = self.spec_id;
        let tx = context.tx();
        let input = tx.common_fields().input().clone();

        let init_frame = match tx.kind() {
            TxKind::Call(target_address) => NewFrameAction::Call(Box::new(CallInputs {
                input,
                gas_limit,
                target_address,
                bytecode_address: target_address,
                caller: tx.common_fields().caller(),
                value: CallValue::Transfer(tx.common_fields().value()),
                scheme: CallScheme::Call,
                is_static: false,
                is_eof: false,
                return_memory_offset: 0..0,
            })),
            TxKind::Create => {
                // if first byte of data is magic 0xEF00, then it is EOFCreate.
                if self.spec_id.is_enabled_in(SpecId::PRAGUE_EOF)
                    && input.starts_with(&EOF_MAGIC_BYTES)
                {
                    NewFrameAction::EOFCreate(Box::new(EOFCreateInputs::new(
                        tx.common_fields().caller(),
                        tx.common_fields().value(),
                        gas_limit,
                        EOFCreateKind::Tx { initdata: input },
                    )))
                } else {
                    NewFrameAction::Create(Box::new(CreateInputs {
                        caller: tx.common_fields().caller(),
                        scheme: CreateScheme::Create,
                        value: tx.common_fields().value(),
                        init_code: input,
                        gas_limit,
                    }))
                }
            }
        };
        // First frame has dummy data and it is used to create shared context.
        //EthFrame::new()
        let shared_memory = Rc::new(RefCell::new(SharedMemory::new()));
        EthFrame::init_with_context(0, init_frame, spec_id, shared_memory, context)
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        mut frame_result: <Self::Frame as FrameTrait>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error> {
        let instruction_result = frame_result.interpreter_result().result;
        let gas = frame_result.gas_mut();
        let remaining = gas.remaining();
        let refunded = gas.refunded();

        // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
        *gas = Gas::new_spent(context.tx().common_fields().gas_limit());

        match instruction_result {
            return_ok!() => {
                gas.erase_cost(remaining);
                gas.record_refund(refunded);
            }
            return_revert!() => {
                gas.erase_cost(remaining);
            }
            _ => {}
        }
        Ok(frame_result)
    }
}

impl<CTX, ERROR> EthExecution<CTX, ERROR> {
    pub fn new(spec_id: SpecId) -> Self {
        Self {
            spec_id,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_boxed(spec_id: SpecId) -> Box<Self> {
        Box::new(Self::new(spec_id))
    }
}

// /// Execute frame
// #[inline]
// pub fn execute_frame<EvmWiringT: EvmWiring, SPEC: Spec>(
//     frame: &mut Frame,
//     shared_memory: &mut SharedMemory,
//     instruction_tables: &InstructionTables<'_, Context<EvmWiringT>>,
//     context: &mut Context<EvmWiringT>,
// ) -> EVMResultGeneric<InterpreterAction, EvmWiringT> {
//     let interpreter = frame.interpreter_mut();
//     let memory = mem::replace(shared_memory, EMPTY_SHARED_MEMORY);
//     let next_action = match instruction_tables {
//         InstructionTables::Plain(table) => interpreter.run(memory, table, context),
//         InstructionTables::Boxed(table) => interpreter.run(memory, table, context),
//     };
//     // Take the shared memory back.
//     *shared_memory = interpreter.take_memory();

//     Ok(next_action)
// }

// /// First frame creation.
// pub fn first_frame_creation<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     gas_limit: u64,
// ) -> EVMResultGeneric<NewFrameAction, EvmWiringT> {
//     // Make new frame action.
//     let tx = &context.evm.env.tx;

//     let input = tx.common_fields().input().clone();

//     let new_frame = match tx.kind() {
//         TxKind::Call(target_address) => NewFrameAction::Call(Box::new(CallInputs {
//             input,
//             gas_limit,
//             target_address,
//             bytecode_address: target_address,
//             caller: tx.common_fields().caller(),
//             value: CallValue::Transfer(tx.common_fields().value()),
//             scheme: CallScheme::Call,
//             is_static: false,
//             is_eof: false,
//             return_memory_offset: 0..0,
//         })),
//         TxKind::Create => {
//             // if first byte of data is magic 0xEF00, then it is EOFCreate.
//             if SPEC::enabled(SpecId::PRAGUE_EOF) && input.starts_with(&EOF_MAGIC_BYTES) {
//                 NewFrameAction::EOFCreate(Box::new(EOFCreateInputs::new(
//                     tx.common_fields().caller(),
//                     tx.common_fields().value(),
//                     gas_limit,
//                     EOFCreateKind::Tx { initdata: input },
//                 )))
//             } else {
//                 NewFrameAction::Create(Box::new(CreateInputs {
//                     caller: tx.common_fields().caller(),
//                     scheme: CreateScheme::Create,
//                     value: tx.common_fields().value(),
//                     init_code: input,
//                     gas_limit,
//                 }))
//             }
//         }
//     };

//     Ok(new_frame)
// }

// /// Handle output of the transaction
// #[inline]
// pub fn last_frame_return<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     frame_result: &mut FrameResult,
// ) -> EVMResultGeneric<(), EvmWiringT> {
//     let instruction_result = frame_result.interpreter_result().result;
//     let gas = frame_result.gas_mut();
//     let remaining = gas.remaining();
//     let refunded = gas.refunded();

//     // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
//     *gas = Gas::new_spent(context.evm.env.tx.common_fields().gas_limit());

//     match instruction_result {
//         return_ok!() => {
//             gas.erase_cost(remaining);
//             gas.record_refund(refunded);
//         }
//         return_revert!() => {
//             gas.erase_cost(remaining);
//         }
//         _ => {}
//     }
//     Ok(())
// }

// /// Handle frame sub call.
// #[inline]
// pub fn call<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     inputs: Box<CallInputs>,
// ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
//     context.evm.make_call_frame(&inputs)
// }

// #[inline]
// pub fn call_return<EvmWiringT: EvmWiring>(
//     context: &mut Context<EvmWiringT>,
//     frame: Box<CallFrame>,
//     interpreter_result: InterpreterResult,
// ) -> EVMResultGeneric<CallOutcome, EvmWiringT> {
//     context
//         .evm
//         .call_return(&interpreter_result, frame.frame_data.checkpoint);
//     Ok(CallOutcome::new(
//         interpreter_result,
//         frame.return_memory_range,
//     ))
// }

// #[inline]
// pub fn insert_call_outcome<EvmWiringT: EvmWiring>(
//     context: &mut Context<EvmWiringT>,
//     frame: &mut Frame,
//     shared_memory: &mut SharedMemory,
//     outcome: CallOutcome,
// ) -> EVMResultGeneric<(), EvmWiringT> {
//     context.evm.take_error().map_err(EVMError::Database)?;

//     frame
//         .frame_data_mut()
//         .interpreter
//         .insert_call_outcome(shared_memory, outcome);
//     Ok(())
// }

// /// Handle frame sub create.
// #[inline]
// pub fn create<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     inputs: Box<CreateInputs>,
// ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
//     context.evm.make_create_frame(SPEC::SPEC_ID, &inputs)
// }

// #[inline]
// pub fn create_return<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     frame: Box<CreateFrame>,
//     mut interpreter_result: InterpreterResult,
// ) -> EVMResultGeneric<CreateOutcome, EvmWiringT> {
//     context.evm.create_return::<SPEC>(
//         &mut interpreter_result,
//         frame.created_address,
//         frame.frame_data.checkpoint,
//     );
//     Ok(CreateOutcome::new(
//         interpreter_result,
//         Some(frame.created_address),
//     ))
// }

// #[inline]
// pub fn insert_create_outcome<EvmWiringT: EvmWiring>(
//     context: &mut Context<EvmWiringT>,
//     frame: &mut Frame,
//     outcome: CreateOutcome,
// ) -> EVMResultGeneric<(), EvmWiringT> {
//     context.evm.take_error().map_err(EVMError::Database)?;

//     frame
//         .frame_data_mut()
//         .interpreter
//         .insert_create_outcome(outcome);
//     Ok(())
// }

// /// Handle frame sub create.
// #[inline]
// pub fn eofcreate<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     inputs: Box<EOFCreateInputs>,
// ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
//     context.evm.make_eofcreate_frame(SPEC::SPEC_ID, &inputs)
// }

// #[inline]
// pub fn eofcreate_return<EvmWiringT: EvmWiring, SPEC: Spec>(
//     context: &mut Context<EvmWiringT>,
//     frame: Box<EOFCreateFrame>,
//     mut interpreter_result: InterpreterResult,
// ) -> EVMResultGeneric<CreateOutcome, EvmWiringT> {
//     context.evm.eofcreate_return::<SPEC>(
//         &mut interpreter_result,
//         frame.created_address,
//         frame.frame_data.checkpoint,
//     );
//     Ok(CreateOutcome::new(
//         interpreter_result,
//         Some(frame.created_address),
//     ))
// }

// #[inline]
// pub fn insert_eofcreate_outcome<EvmWiringT: EvmWiring>(
//     context: &mut Context<EvmWiringT>,
//     frame: &mut Frame,
//     outcome: CreateOutcome,
// ) -> EVMResultGeneric<(), EvmWiringT> {
//     context.evm.take_error().map_err(EVMError::Database)?;

//     frame
//         .frame_data_mut()
//         .interpreter
//         .insert_eofcreate_outcome(outcome);
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::handler::mainnet::refund;
//     use interpreter::InstructionResult;
//     use primitives::Bytes;
//     use specification::hardfork::CancunSpec;
//     use wiring::{default::EnvWiring, DefaultEthereumWiring};

//     /// Creates frame result.
//     fn call_last_frame_return(instruction_result: InstructionResult, gas: Gas) -> Gas {
//         let mut env = EnvWiring::<DefaultEthereumWiring>::default();
//         env.tx.gas_limit = 100;

//         let mut ctx = Context::default();
//         ctx.evm.inner.env = Box::new(env);
//         let mut first_frame = FrameResult::Call(CallOutcome::new(
//             InterpreterResult {
//                 result: instruction_result,
//                 output: Bytes::new(),
//                 gas,
//             },
//             0..0,
//         ));
//         last_frame_return::<DefaultEthereumWiring, CancunSpec>(&mut ctx, &mut first_frame).unwrap();
//         refund::<DefaultEthereumWiring, CancunSpec>(&mut ctx, first_frame.gas_mut(), 0);
//         *first_frame.gas()
//     }

//     #[test]
//     fn test_consume_gas() {
//         let gas = call_last_frame_return(InstructionResult::Stop, Gas::new(90));
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_consume_gas_with_refund() {
//         let mut return_gas = Gas::new(90);
//         return_gas.record_refund(30);

//         let gas = call_last_frame_return(InstructionResult::Stop, return_gas);
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 2);

//         let gas = call_last_frame_return(InstructionResult::Revert, return_gas);
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }

//     #[test]
//     fn test_revert_gas() {
//         let gas = call_last_frame_return(InstructionResult::Revert, Gas::new(90));
//         assert_eq!(gas.remaining(), 90);
//         assert_eq!(gas.spent(), 10);
//         assert_eq!(gas.refunded(), 0);
//     }
// }
