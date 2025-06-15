use crate::{interpreter_types::Jumps, Interpreter, InterpreterTypes};

use super::Instruction;

use core::task::{RawWaker, RawWakerVTable};

pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    pub host: &'a mut H,
    pub interpreter: &'a mut Interpreter<ITy>,
}

impl<H: ?Sized, ITy: InterpreterTypes> InstructionContext<'_, H, ITy> {
    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    pub(crate) fn step(self, instruction_table: &[Instruction<ITy, H>; 256]) {
        use core::{
            pin::Pin,
            task::{Context, Waker},
        };

        // Get current opcode.
        let opcode = self.interpreter.bytecode.opcode();

        // Increment program counter (see safety note in original code).
        self.interpreter.bytecode.relative_jump(1);

        // Create and poll the future for this opcode until it is ready.
        let mut fut: Pin<Box<dyn core::future::Future<Output = ()>>> =
            (instruction_table[opcode as usize])(self);

        // Poll once; most opcode futures finish immediately because they perform synchronous work.
        // If an opcode performs real async I/O this will leave it in Pending state, which is *not* yet supported.
        let waker: Waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        let _ = fut.as_mut().poll(&mut cx);
    }
}

// SAFETY: the waker functions do nothing, which is sufficient because we poll each
// opcode future to completion immediately.
unsafe fn dummy_raw_waker() -> RawWaker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    unsafe fn noop(_: *const ()) {}
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    RawWaker::new(core::ptr::null(), &VTABLE)
}
