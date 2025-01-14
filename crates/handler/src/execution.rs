use crate::frame::FrameContext;

use super::{frame_data::FrameResult, EthFrame, EthPrecompileProvider};
use bytecode::EOF_MAGIC_BYTES;
use context_interface::{
    result::InvalidTransaction, BlockGetter, Cfg, CfgGetter, ErrorGetter, JournalDBError,
    JournalGetter, Transaction, TransactionGetter,
};
use handler_interface::{util::FrameOrFrameResult, ExecutionHandler, Frame as FrameTrait};
use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter},
    CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, EOFCreateInputs, EOFCreateKind,
    FrameInput, Gas,
};
use primitives::TxKind;
use specification::hardfork::SpecId;
use std::boxed::Box;

#[derive(Default)]
pub struct EthExecution<
    CTX,
    ERROR,
    FRAME = EthFrame<CTX, ERROR, EthInterpreter<()>, FrameContext<CTX, EthInterpreter<()>, ERROR>>,
> {
    _phantom: core::marker::PhantomData<(CTX, FRAME, ERROR)>,
}

impl<CTX, ERROR, FRAME> ExecutionHandler for EthExecution<CTX, ERROR, FRAME>
where
    CTX: EthExecutionContext,
    ERROR: EthExecutionError<CTX>,
    FRAME:
        FrameTrait<Context = CTX, Error = ERROR, FrameInit = FrameInput, FrameResult = FrameResult>,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type ExecResult = FrameResult;

    fn init_first_frame(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut <Self::Frame as FrameTrait>::FrameContext,
        gas_limit: u64,
    ) -> Result<FrameOrFrameResult<Self::Frame>, Self::Error> {
        // Make new frame action.
        let spec = context.cfg().spec().into();
        let tx = context.tx();
        let input = tx.input().clone();

        let init_frame: FrameInput = match tx.kind() {
            TxKind::Call(target_address) => FrameInput::Call(Box::new(CallInputs {
                input,
                gas_limit,
                target_address,
                bytecode_address: target_address,
                caller: tx.caller(),
                value: CallValue::Transfer(tx.value()),
                scheme: CallScheme::Call,
                is_static: false,
                is_eof: false,
                return_memory_offset: 0..0,
            })),
            TxKind::Create => {
                // If first byte of data is magic 0xEF00, then it is EOFCreate.
                if spec.is_enabled_in(SpecId::OSAKA) && input.starts_with(&EOF_MAGIC_BYTES) {
                    FrameInput::EOFCreate(Box::new(EOFCreateInputs::new(
                        tx.caller(),
                        tx.value(),
                        gas_limit,
                        EOFCreateKind::Tx { initdata: input },
                    )))
                } else {
                    FrameInput::Create(Box::new(CreateInputs {
                        caller: tx.caller(),
                        scheme: CreateScheme::Create,
                        value: tx.value(),
                        init_code: input,
                        gas_limit,
                    }))
                }
            }
        };
        FRAME::init_first(context, frame_context, init_frame)
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        _frame_context: &mut <Self::Frame as FrameTrait>::FrameContext,
        mut frame_result: <Self::Frame as FrameTrait>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error> {
        let instruction_result = frame_result.interpreter_result().result;
        let gas = frame_result.gas_mut();
        let remaining = gas.remaining();
        let refunded = gas.refunded();

        // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
        *gas = Gas::new_spent(context.tx().gas_limit());

        if instruction_result.is_ok_or_revert() {
            gas.erase_cost(remaining);
        }

        if instruction_result.is_ok() {
            gas.record_refund(refunded);
        }

        Ok(frame_result)
    }
}

impl<CTX, ERROR, FRAME> EthExecution<CTX, ERROR, FRAME> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

pub trait EthExecutionContext:
    TransactionGetter
    + ErrorGetter<Error = JournalDBError<Self>>
    + BlockGetter
    + JournalGetter
    + CfgGetter
{
}

impl<
        T: TransactionGetter
            + ErrorGetter<Error = JournalDBError<T>>
            + BlockGetter
            + JournalGetter
            + CfgGetter,
    > EthExecutionContext for T
{
}

pub trait EthExecutionError<CTX: JournalGetter>:
    From<InvalidTransaction> + From<JournalDBError<CTX>>
{
}

impl<CTX: JournalGetter, T: From<InvalidTransaction> + From<JournalDBError<CTX>>>
    EthExecutionError<CTX> for T
{
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::handler::mainnet::refund;
//     use interpreter::InstructionResult;
//     use primitives::Bytes;
//     use specification::hardfork::CancunSpec;
//     use context_interface::{default::EnvWiring, DefaultEthereumWiring};

//     /// Creates frame result.
//     fn call_last_frame_return(instruction_result: InstructionResult, gas: Gas) -> Gas {
//         let mut env = Envcontext_interface::<DefaultEthereumWiring>::default();
//         env.tx.gas_limit = 100;

//         let mut context = Context::builder();
//         context.evm.inner.env = Box::new(env);
//         let mut first_frame = FrameResult::Call(CallOutcome::new(
//             InterpreterResult {
//                 result: instruction_result,
//                 output: Bytes::new(),
//                 gas,
//             },
//             0..0,
//         ));
//         last_frame_return::<DefaultEthereumWiring, CancunSpec>(&mut context, &mut first_frame).unwrap();
//         refund::<DefaultEthereumWiring, CancunSpec>(&mut context, first_frame.gas_mut(), 0);
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
