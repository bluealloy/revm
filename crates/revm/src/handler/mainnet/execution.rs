use crate::{
    db::Database,
    frame::EOFCreateFrame,
    interpreter::{
        return_ok, return_revert, CallInputs, CreateInputs, CreateOutcome, Gas, InstructionResult,
        SharedMemory,
    },
    primitives::{EVMError, Spec},
    CallFrame, Context, CreateFrame, Frame, FrameOrResult, FrameResult,
};
use core::mem;
use revm_interpreter::{
    opcode::InstructionTables, CallOutcome, EOFCreateInputs, InterpreterAction, InterpreterResult,
    EMPTY_SHARED_MEMORY,
};
use std::boxed::Box;

/// Execute frame
#[inline]
pub fn execute_frame<SPEC: Spec, EXT, DB: Database>(
    frame: &mut Frame,
    shared_memory: &mut SharedMemory,
    instruction_tables: &InstructionTables<'_, Context<EXT, DB>>,
    context: &mut Context<EXT, DB>,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    let interpreter = frame.interpreter_mut();
    let memory = mem::replace(shared_memory, EMPTY_SHARED_MEMORY);
    let next_action = match instruction_tables {
        InstructionTables::Plain(table) => interpreter.run(memory, table, context),
        InstructionTables::Boxed(table) => interpreter.run(memory, table, context),
    };
    // Take the shared memory back.
    *shared_memory = interpreter.take_memory();

    Ok(next_action)
}

/// Handle output of the transaction
#[inline]
pub fn last_frame_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame_result: &mut FrameResult,
) -> Result<(), EVMError<DB::Error>> {
    let instruction_result = frame_result.interpreter_result().result;
    let gas = frame_result.gas_mut();
    let remaining = gas.remaining();
    let refunded = gas.refunded();

    // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
    *gas = Gas::new_spent(context.evm.env.tx.gas_limit);

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
    Ok(())
}

/// Handle frame sub call.
#[inline]
pub fn call<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: Box<CallInputs>,
) -> Result<FrameOrResult, EVMError<DB::Error>> {
    context.evm.make_call_frame(&inputs)
}

#[inline]
pub fn call_return<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: Box<CallFrame>,
    interpreter_result: InterpreterResult,
) -> Result<CallOutcome, EVMError<DB::Error>> {
    context
        .evm
        .call_return(&interpreter_result, frame.frame_data.checkpoint);
    Ok(CallOutcome::new(
        interpreter_result,
        frame.return_memory_range,
    ))
}

#[inline]
pub fn insert_call_outcome<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: &mut Frame,
    shared_memory: &mut SharedMemory,
    outcome: CallOutcome,
) -> Result<(), EVMError<DB::Error>> {
    context.evm.take_error()?;
    frame
        .frame_data_mut()
        .interpreter
        .insert_call_outcome(shared_memory, outcome);
    Ok(())
}

/// Handle frame sub create.
#[inline]
pub fn create<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: Box<CreateInputs>,
) -> Result<FrameOrResult, EVMError<DB::Error>> {
    context.evm.make_create_frame(SPEC::SPEC_ID, &inputs)
}

#[inline]
pub fn create_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: Box<CreateFrame>,
    mut interpreter_result: InterpreterResult,
) -> Result<CreateOutcome, EVMError<DB::Error>> {
    context.evm.create_return::<SPEC>(
        &mut interpreter_result,
        frame.created_address,
        frame.frame_data.checkpoint,
    );
    Ok(CreateOutcome::new(
        interpreter_result,
        Some(frame.created_address),
    ))
}

#[inline]
pub fn insert_create_outcome<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: &mut Frame,
    outcome: CreateOutcome,
) -> Result<(), EVMError<DB::Error>> {
    context.evm.take_error()?;
    frame
        .frame_data_mut()
        .interpreter
        .insert_create_outcome(outcome);
    Ok(())
}

/// Handle frame sub create.
#[inline]
pub fn eofcreate<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: Box<EOFCreateInputs>,
) -> Result<FrameOrResult, EVMError<DB::Error>> {
    context.evm.make_eofcreate_frame(SPEC::SPEC_ID, &inputs)
}

#[inline]
pub fn eofcreate_return<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: Box<EOFCreateFrame>,
    mut interpreter_result: InterpreterResult,
) -> Result<CreateOutcome, EVMError<DB::Error>> {
    context.evm.eofcreate_return::<SPEC>(
        &mut interpreter_result,
        frame.created_address,
        frame.frame_data.checkpoint,
    );
    Ok(CreateOutcome::new(
        interpreter_result,
        Some(frame.created_address),
    ))
}

#[inline]
pub fn insert_eofcreate_outcome<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    frame: &mut Frame,
    outcome: CreateOutcome,
) -> Result<(), EVMError<DB::Error>> {
    core::mem::replace(&mut context.evm.error, Ok(()))?;
    frame
        .frame_data_mut()
        .interpreter
        .insert_eofcreate_outcome(outcome);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::mainnet::refund;
    use crate::primitives::{CancunSpec, Env};
    use revm_precompile::Bytes;

    /// Creates frame result.
    fn call_last_frame_return(instruction_result: InstructionResult, gas: Gas) -> Gas {
        let mut env = Env::default();
        env.tx.gas_limit = 100;

        let mut ctx = Context::new_empty();
        ctx.evm.inner.env = Box::new(env);
        let mut first_frame = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));
        last_frame_return::<CancunSpec, _, _>(&mut ctx, &mut first_frame).unwrap();
        refund::<CancunSpec, _, _>(&mut ctx, first_frame.gas_mut(), 0);
        *first_frame.gas()
    }

    #[test]
    fn test_consume_gas() {
        let gas = call_last_frame_return(InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let mut return_gas = Gas::new(90);
        return_gas.record_refund(30);

        let gas = call_last_frame_return(InstructionResult::Stop, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 2);

        let gas = call_last_frame_return(InstructionResult::Revert, return_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_revert_gas() {
        let gas = call_last_frame_return(InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }
}
