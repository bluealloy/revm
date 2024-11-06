use super::{frame_data::FrameResult, EthFrame};
use crate::handler::{
    wires::Frame as FrameTrait, EthPrecompileProvider, ExecutionWire, FrameOrResultGen,
    PrecompileProvider,
};
use bytecode::EOF_MAGIC_BYTES;
use context::{
    BlockGetter, CfgGetter, ErrorGetter, JournalStateGetter, JournalStateGetterDBError,
    TransactionGetter,
};
use core::cell::RefCell;
use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter},
    return_ok, return_revert, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme,
    EOFCreateInputs, EOFCreateKind, Gas, NewFrameAction, SharedMemory,
};
use precompile::PrecompileSpecId;
use primitives::TxKind;
use specification::hardfork::SpecId;
use std::{boxed::Box, rc::Rc};
use wiring::{journaled_state::JournaledState, result::InvalidTransaction, Cfg, Transaction};

/// TODO EvmWiringT is temporary, replace it with getter traits.
pub struct EthExecution<
    CTX,
    ERROR,
    FRAME = EthFrame<
        CTX,
        ERROR,
        EthInterpreter<()>,
        EthPrecompileProvider<CTX>,
        EthInstructionProvider<EthInterpreter<()>, CTX>,
    >,
> {
    _phantom: std::marker::PhantomData<(CTX, FRAME, ERROR)>,
}

impl<CTX, ERROR, FRAME> ExecutionWire for EthExecution<CTX, ERROR, FRAME>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter
        + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalStateGetterDBError<CTX>>,
    FRAME: FrameTrait<
        Context = CTX,
        Error = ERROR,
        FrameInit = NewFrameAction,
        FrameResult = FrameResult,
    >,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type ExecResult = FrameResult;

    fn init_first_frame(
        &mut self,
        context: &mut Self::Context,
        gas_limit: u64,
    ) -> Result<FrameOrResultGen<Self::Frame, <Self::Frame as FrameTrait>::FrameResult>, Self::Error>
    {
        // TODO do this in frame
        // self.precompiles
        //     .set_spec_id(PrecompileSpecId::from_spec_id(self.spec_id));
        // // wamr up precompile address.
        // for address in self.precompiles.warm_addresses() {
        //     context.journal().warm_account(address);
        // }

        // Make new frame action.
        let spec = context.cfg().spec().into();
        let tx = context.tx();
        let input = tx.common_fields().input().clone();

        let init_frame: NewFrameAction = match tx.kind() {
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
                if spec.is_enabled_in(SpecId::PRAGUE_EOF) && input.starts_with(&EOF_MAGIC_BYTES) {
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
        //let shared_memory = Rc::new(RefCell::new(SharedMemory::new()));
        //EthFrame::init_with_context(0, init_frame, spec_id, shared_memory, context)
        FRAME::init_first(context, init_frame)
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
        Ok(frame_result.into())
    }
}

impl<CTX, ERROR> EthExecution<CTX, ERROR> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

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
