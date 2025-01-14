use crate::util::FrameOrFrameResult;
pub use crate::{Frame, FrameOrResultGen};
pub use std::{vec, vec::Vec};

pub trait ExecutionHandler {
    type Context;
    type Error;
    type Frame: Frame<Context = Self::Context, Error = Self::Error>;
    type ExecResult;

    /// Execute call.
    fn init_first_frame(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as ExecutionHandler>::Frame as Frame>::FrameContext,
        gas_limit: u64,
    ) -> Result<FrameOrFrameResult<Self::Frame>, Self::Error>;

    /// Execute create.
    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as ExecutionHandler>::Frame as Frame>::FrameContext,
        frame_result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error>;

    fn run(
        &self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as ExecutionHandler>::Frame as Frame>::FrameContext,
        frame: Self::Frame,
    ) -> Result<Self::ExecResult, Self::Error> {
        let mut frame_stack: Vec<<Self as ExecutionHandler>::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = frame.run(context, frame_context)?;

            let mut result = match call_or_result {
                FrameOrResultGen::Frame(init) => match frame.init(context, frame_context, init)? {
                    FrameOrResultGen::Frame(new_frame) => {
                        frame_stack.push(new_frame);
                        continue;
                    }
                    // Dont pop the frame as new frame was not created.
                    FrameOrResultGen::Result(result) => result,
                },
                FrameOrResultGen::Result(result) => {
                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                Self::Frame::final_return(context, frame_context, &mut result)?;
                return self.last_frame_result(context, frame_context, result);
            };
            frame.return_result(context, frame_context, result)?;
        }
    }
}
