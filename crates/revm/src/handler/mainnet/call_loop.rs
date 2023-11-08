use revm_interpreter::SharedMemory;

use crate::{
    interpreter::{CallInputs, CreateInputs},
    primitives::Spec,
    CallStackFrame, Database, Evm, EvmContext, FrameOrResult,
};
use core::ops::Range;

///
pub fn initial_call_create() -> FrameOrResult {
    unimplemented!()
}

// Handles action for new sub call, return None if there is no need to add
// new stack frame.
// #[inline]
// pub fn sub_call<SPEC: Spec, DB: Database>(
//     evm: &mut Evm<'_, SPEC, DB>,
//     mut inputs: Box<CallInputs>,
//     curent_stake_frame: &mut CallStackFrame,
//     shared_memory: &mut SharedMemory,
//     return_memory_offset: Range<usize>,
// ) -> Option<Box<CallStackFrame>> {
//     // Call inspector if it is some.
//     if let Some(inspector) = evm.inspector.as_mut() {
//         if let Some((result, range)) = inspector.call(&mut evm.context, &mut inputs) {
//             curent_stake_frame
//                 .interpreter
//                 .insert_call_output(shared_memory, result, range);
//             return None;
//         }
//     }
//     match evm
//         .context
//         .make_call_frame(&inputs, return_memory_offset.clone())
//     {
//         FrameOrResult::Frame(new_frame) => Some(new_frame),
//         FrameOrResult::Result(mut result) => {
//             //println!("Result returned right away: {:#?}", result);
//             if let Some(inspector) = evm.inspector.as_mut() {
//                 result = inspector.call_end(&mut evm.context, result);
//             }
//             curent_stake_frame.interpreter.insert_call_output(
//                 shared_memory,
//                 result,
//                 return_memory_offset,
//             );
//             None
//         }
//     }
// }

// /// Handle Action for new sub create call, return None if there is no need
// /// to add new stack frame.
// pub fn sub_create<SPEC: Spec, DB: Database>(
//     evm: &mut Evm<'_, SPEC, DB>,
//     curent_stack_frame: &mut CallStackFrame,
//     mut inputs: Box<CreateInputs>,
// ) -> Option<Box<CallStackFrame>> {
//     // Call inspector if it is some.
//     if let Some(inspector) = evm.inspector.as_mut() {
//         if let Some((result, address)) = inspector.create(&mut evm.context, &mut inputs) {
//             curent_stack_frame
//                 .interpreter
//                 .insert_create_output(result, address);
//             return None;
//         }
//     }

//     match evm.context.make_create_frame::<SPEC>(&inputs) {
//         FrameOrResult::Frame(new_frame) => Some(new_frame),
//         FrameOrResult::Result(mut result) => {
//             let mut address = None;
//             if let Some(inspector) = evm.inspector.as_mut() {
//                 let ret = inspector.create_end(
//                     &mut evm.context,
//                     result,
//                     curent_stack_frame.created_address,
//                 );
//                 result = ret.0;
//                 address = ret.1;
//             }
//             // insert result of the failed creation of create CallStackFrame.
//             curent_stack_frame
//                 .interpreter
//                 .insert_create_output(result, address);
//             None
//         }
//     }
// }
