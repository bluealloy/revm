use crate::inspector::Inspector;
use either::Either;
use interpreter::{
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, InterpreterTypes,
};
use primitives::{Address, Log, U256};

impl<CTX, INTR: InterpreterTypes, FI, FR, L, R> Inspector<CTX, INTR, FI, FR> for Either<L, R>
where
    L: Inspector<CTX, INTR, FI, FR>,
    R: Inspector<CTX, INTR, FI, FR>,
{
    #[inline]
    fn initialize_interp(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        match self {
            Either::Left(inspector) => inspector.initialize_interp(interp, context),
            Either::Right(inspector) => inspector.initialize_interp(interp, context),
        }
    }

    #[inline]
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        match self {
            Either::Left(inspector) => inspector.step(interp, context),
            Either::Right(inspector) => inspector.step(interp, context),
        }
    }

    #[inline]
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {
        match self {
            Either::Left(inspector) => inspector.step_end(interp, context),
            Either::Right(inspector) => inspector.step_end(interp, context),
        }
    }

    #[inline]
    fn log(&mut self, context: &mut CTX, log: Log) {
        match self {
            Either::Left(inspector) => inspector.log(context, log),
            Either::Right(inspector) => inspector.log(context, log),
        }
    }

    #[inline]
    fn log_full(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX, log: Log) {
        match self {
            Either::Left(inspector) => inspector.log_full(interp, context, log),
            Either::Right(inspector) => inspector.log_full(interp, context, log),
        }
    }

    #[inline]
    fn frame_start(&mut self, context: &mut CTX, frame_input: &mut FI) -> Option<FR> {
        match self {
            Either::Left(inspector) => inspector.frame_start(context, frame_input),
            Either::Right(inspector) => inspector.frame_start(context, frame_input),
        }
    }

    #[inline]
    fn frame_end(&mut self, context: &mut CTX, frame_input: &FI, frame_result: &mut FR) {
        match self {
            Either::Left(inspector) => inspector.frame_end(context, frame_input, frame_result),
            Either::Right(inspector) => inspector.frame_end(context, frame_input, frame_result),
        }
    }

    #[inline]
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        match self {
            Either::Left(inspector) => inspector.call(context, inputs),
            Either::Right(inspector) => inspector.call(context, inputs),
        }
    }

    #[inline]
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        match self {
            Either::Left(inspector) => inspector.call_end(context, inputs, outcome),
            Either::Right(inspector) => inspector.call_end(context, inputs, outcome),
        }
    }

    #[inline]
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        match self {
            Either::Left(inspector) => inspector.create(context, inputs),
            Either::Right(inspector) => inspector.create(context, inputs),
        }
    }

    #[inline]
    fn create_end(
        &mut self,
        context: &mut CTX,
        inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        match self {
            Either::Left(inspector) => inspector.create_end(context, inputs, outcome),
            Either::Right(inspector) => inspector.create_end(context, inputs, outcome),
        }
    }

    #[inline]
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        match self {
            Either::Left(inspector) => inspector.selfdestruct(contract, target, value),
            Either::Right(inspector) => inspector.selfdestruct(contract, target, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noop::NoOpInspector;
    use interpreter::interpreter::EthInterpreter;

    #[derive(Default)]
    struct DummyInsp;

    impl<CTX> Inspector<CTX, EthInterpreter> for DummyInsp {}

    #[test]
    fn test_either_inspector_type_check() {
        // This test verifies that Either<NoOpInspector, NoOpInspector>
        // implements the Inspector trait as required by the issue
        fn _requires_inspector<T: Inspector<(), EthInterpreter>>(inspector: T) -> T {
            inspector
        }

        let left_inspector = Either::<NoOpInspector, DummyInsp>::Left(NoOpInspector);
        let right_inspector = Either::<NoOpInspector, DummyInsp>::Right(DummyInsp);

        // These calls should compile successfully, proving that the Inspector trait is implemented
        let _left = _requires_inspector(left_inspector);
        let _right = _requires_inspector(right_inspector);
    }

    /// An inspector whose `frame_start`/`frame_end` hooks are observable via counters.
    #[derive(Default)]
    struct FrameHookCounter {
        frame_start_calls: u32,
        frame_end_calls: u32,
    }

    impl<CTX> Inspector<CTX, EthInterpreter> for FrameHookCounter {
        fn frame_start(
            &mut self,
            _context: &mut CTX,
            _frame_input: &mut interpreter::FrameInput,
        ) -> Option<handler::FrameResult> {
            self.frame_start_calls += 1;
            None
        }

        fn frame_end(
            &mut self,
            _context: &mut CTX,
            _frame_input: &interpreter::FrameInput,
            _frame_result: &mut handler::FrameResult,
        ) {
            self.frame_end_calls += 1;
        }
    }

    #[test]
    fn test_either_forwards_frame_start_and_frame_end() {
        let mut wrapped: Either<FrameHookCounter, FrameHookCounter> =
            Either::Left(FrameHookCounter::default());
        let mut ctx = ();
        let mut frame_input = interpreter::FrameInput::Empty;

        let _ =
            Inspector::<(), EthInterpreter>::frame_start(&mut wrapped, &mut ctx, &mut frame_input);

        let Either::Left(inner) = &wrapped else {
            unreachable!("wrapped is always Either::Left in this test")
        };
        assert_eq!(
            inner.frame_start_calls, 1,
            "Either<L, R> must forward frame_start to the wrapped inspector"
        );

        let mut frame_result = handler::FrameResult::Call(interpreter::CallOutcome::new(
            interpreter::InterpreterResult {
                result: interpreter::InstructionResult::Stop,
                output: Default::default(),
                gas: Default::default(),
            },
            0..0,
        ));
        Inspector::<(), EthInterpreter>::frame_end(
            &mut wrapped,
            &mut ctx,
            &frame_input,
            &mut frame_result,
        );

        let Either::Left(inner) = &wrapped else {
            unreachable!("wrapped is always Either::Left in this test")
        };
        assert_eq!(
            inner.frame_end_calls, 1,
            "Either<L, R> must forward frame_end to the wrapped inspector"
        );
    }
}
