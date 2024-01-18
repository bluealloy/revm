use crate::{
    db::Database,
    interpreter::{
        return_ok, return_revert, CallInputs, CreateInputs, CreateOutcome, Gas, InstructionResult,
        InterpreterResult, SharedMemory,
    },
    primitives::{Env, Spec, TransactTo},
    CallStackFrame, Context, FrameData, FrameOrResult,
};
use alloc::boxed::Box;
use core::ops::Range;

/// Creates first frame.
#[inline]
pub fn create_first_frame<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas_limit: u64,
) -> FrameOrResult {
    // call inner handling of call/create
    match context.evm.env.tx.transact_to {
        TransactTo::Call(_) => context.evm.make_call_frame(
            &CallInputs::new(&context.evm.env.tx, gas_limit).unwrap(),
            0..0,
        ),
        TransactTo::Create(_) => context.evm.make_create_frame(
            SPEC::SPEC_ID,
            &CreateInputs::new(&context.evm.env.tx, gas_limit).unwrap(),
        ),
    }
}

/// Helper function called inside [`first_frame_return`]
#[inline]
pub fn frame_return_with_refund_flag<SPEC: Spec>(
    env: &Env,
    call_result: InstructionResult,
    returned_gas: Gas,
    refund_enabled: bool,
) -> Gas {
    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    let mut gas = Gas::new(env.tx.gas_limit);
    gas.record_cost(env.tx.gas_limit);

    match call_result {
        return_ok!() => {
            gas.erase_cost(returned_gas.remaining());
            gas.record_refund(returned_gas.refunded());
        }
        return_revert!() => {
            gas.erase_cost(returned_gas.remaining());
        }
        _ => {}
    }
    // Calculate gas refund for transaction.
    // If config is set to disable gas refund, it will return 0.
    // If spec is set to london, it will decrease the maximum refund amount to 5th part of
    // gas spend. (Before london it was 2th part of gas spend)
    if refund_enabled {
        // EIP-3529: Reduction in refunds
        gas.set_final_refund::<SPEC>()
    };

    gas
}

/// Handle output of the transaction
#[inline]
pub fn first_frame_return<SPEC: Spec>(
    env: &Env,
    call_result: InstructionResult,
    returned_gas: Gas,
) -> Gas {
    frame_return_with_refund_flag::<SPEC>(env, call_result, returned_gas, true)
}

/// Handle frame return.
#[inline]
pub fn frame_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    child_stack_frame: Box<CallStackFrame>,
    parent_stack_frame: Option<&mut Box<CallStackFrame>>,
    shared_memory: &mut SharedMemory,
    result: InterpreterResult,
) -> Option<InterpreterResult> {
    match child_stack_frame.frame_data {
        FrameData::Create { created_address } => {
            let result = context.evm.create_return::<SPEC>(
                result,
                created_address,
                child_stack_frame.checkpoint,
            );
            let Some(parent_stack_frame) = parent_stack_frame else {
                return Some(result);
            };
            let create_outcome = CreateOutcome::new(result, Some(created_address));
            parent_stack_frame
                .interpreter
                .insert_create_outcome(create_outcome)
        }
        FrameData::Call {
            return_memory_range,
        } => {
            let result = context
                .evm
                .call_return(result, child_stack_frame.checkpoint);
            let Some(parent_stack_frame) = parent_stack_frame else {
                return Some(result);
            };

            parent_stack_frame.interpreter.insert_call_output(
                shared_memory,
                result,
                return_memory_range,
            )
        }
    }
    None
}

/// Handle frame sub call.
#[inline]
pub fn sub_call<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: Box<CallInputs>,
    curent_stack_frame: &mut CallStackFrame,
    shared_memory: &mut SharedMemory,
    return_memory_offset: Range<usize>,
) -> Option<Box<CallStackFrame>> {
    match context
        .evm
        .make_call_frame(&inputs, return_memory_offset.clone())
    {
        FrameOrResult::Frame(new_frame) => Some(new_frame),
        FrameOrResult::Result(result) => {
            curent_stack_frame.interpreter.insert_call_output(
                shared_memory,
                result,
                return_memory_offset,
            );
            None
        }
    }
}

/// Handle frame sub create.
#[inline]
pub fn sub_create<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    curent_stack_frame: &mut CallStackFrame,
    inputs: Box<CreateInputs>,
) -> Option<Box<CallStackFrame>> {
    match context.evm.make_create_frame(SPEC::SPEC_ID, &inputs) {
        FrameOrResult::Frame(new_frame) => Some(new_frame),
        FrameOrResult::Result(result) => {
            let create_outcome = CreateOutcome::new(result, None);
            // insert result of the failed creation of create CallStackFrame.
            curent_stack_frame
                .interpreter
                .insert_create_outcome(create_outcome);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use revm_interpreter::primitives::CancunSpec;

    use super::*;

    #[test]
    fn test_consume_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let gas = first_frame_return::<CancunSpec>(&env, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    // TODO
    #[test]
    fn test_consume_gas_with_refund() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let mut return_gas = Gas::new(90);
        return_gas.record_refund(30);

        let gas = first_frame_return::<CancunSpec>(&env, InstructionResult::Stop, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 2);

        let gas = first_frame_return::<CancunSpec>(&env, InstructionResult::Revert, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_revert_gas() {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let gas = first_frame_return::<CancunSpec>(&env, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spend(), 10);
        assert_eq!(gas.refunded(), 0);
    }
}
