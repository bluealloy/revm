use crate::{
    instructions::InstructionReturn,
    interpreter_types::{BytecodeTr, InputsTr, Jumps, MemoryTr, ReturnData, RuntimeFlag, StackTr},
    Gas, Instruction, InstructionResult, InstructionTable, Interpreter, InterpreterTypes,
};
use context_interface::Host;

/// Context passed to instruction implementations.
#[allow(missing_docs)]
pub trait InstructionContextTr: Sized {
    fn runtime_flag(&self) -> &impl RuntimeFlag;
    fn stack(&mut self) -> &mut impl StackTr;
    fn input(&mut self) -> &mut impl InputsTr;
    fn bytecode(&mut self) -> &mut impl BytecodeTr;
    fn return_data(&mut self) -> &mut impl ReturnData;

    fn gas(&self) -> &Gas;
    fn remaining_gas(&self) -> u64 {
        self.gas().remaining()
    }
    #[must_use]
    fn record_gas_cost(&mut self, cost: u64) -> bool;
    fn record_refund(&mut self, refund: i64);

    fn halt(&mut self, result: InstructionResult) -> InstructionReturn;

    fn memory(&mut self) -> &mut impl MemoryTr;
    #[must_use]
    fn resize_memory(&mut self, offset: usize, len: usize) -> bool;

    fn host(&mut self) -> &mut (impl Host + ?Sized);
}

/// Default implementation of [`InstructionContextTr`].
pub struct InstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    /// Reference to the interpreter containing execution state (stack, memory, gas, etc).
    pub interpreter: &'a mut Interpreter<ITy>,
    /// Reference to the host interface for accessing external blockchain state.
    pub host: &'a mut H,
}

impl<H: ?Sized, ITy: InterpreterTypes> std::fmt::Debug for InstructionContext<'_, H, ITy> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionContext")
            .field("host", &"<host>")
            .field("interpreter", &"<interpreter>")
            .finish()
    }
}

impl<'a, H: ?Sized, ITy: InterpreterTypes> InstructionContext<'a, H, ITy> {
    /// Create a new instruction context.
    #[inline]
    pub fn new(interpreter: &'a mut Interpreter<ITy>, host: &'a mut H) -> Self {
        Self { interpreter, host }
    }

    /// Reborrows the context.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> InstructionContext<'b, H, ITy> {
        InstructionContext {
            interpreter: self.interpreter,
            host: self.host,
        }
    }

    /// Executes the instruction in this context.
    #[inline]
    pub fn call(&mut self, f: Instruction<ITy, H>) -> InstructionReturn {
        f(self.interpreter, self.host, core::ptr::null_mut())
    }

    #[inline]
    pub(crate) fn pre_step(&mut self) -> u8 {
        let opcode = self.interpreter.bytecode.opcode();
        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.interpreter.bytecode.relative_jump(1);
        opcode
    }

    #[inline]
    pub(crate) fn step(
        &mut self,
        instruction_table: &InstructionTable<ITy, H>,
    ) -> InstructionReturn {
        let opcode = self.pre_step();
        self.call(instruction_table[opcode as usize])
    }
}

#[allow(refining_impl_trait)] // Keeping the `impl` in returns requires adding lifetime bounds.
impl<'a, I, H> InstructionContextTr for InstructionContext<'a, H, I>
where
    I: InterpreterTypes,
    H: Host + ?Sized,
{
    fn runtime_flag(&self) -> &I::RuntimeFlag {
        &self.interpreter.runtime_flag
    }
    fn stack(&mut self) -> &mut I::Stack {
        &mut self.interpreter.stack
    }
    fn input(&mut self) -> &mut I::Input {
        &mut self.interpreter.input
    }
    fn bytecode(&mut self) -> &mut I::Bytecode {
        &mut self.interpreter.bytecode
    }
    fn return_data(&mut self) -> &mut I::ReturnData {
        &mut self.interpreter.return_data
    }

    fn gas(&self) -> &Gas {
        &self.interpreter.gas
    }
    fn record_gas_cost(&mut self, cost: u64) -> bool {
        self.interpreter.gas.record_cost(cost)
    }
    fn record_refund(&mut self, refund: i64) {
        self.interpreter.gas.record_refund(refund);
    }

    fn halt(&mut self, result: InstructionResult) -> InstructionReturn {
        self.interpreter.halt(result);
        InstructionReturn::halt()
    }

    fn memory(&mut self) -> &mut I::Memory {
        &mut self.interpreter.memory
    }
    fn resize_memory(&mut self, offset: usize, len: usize) -> bool {
        self.interpreter.resize_memory(offset, len)
    }

    fn host(&mut self) -> &mut H {
        self.host
    }
}

pub(crate) struct TailInstructionContext<'a, H: ?Sized, ITy: InterpreterTypes> {
    pub(crate) inner: InstructionContext<'a, H, ITy>,
    pub(crate) ip: *const u8,
}

impl<'a, H: ?Sized, ITy: InterpreterTypes> TailInstructionContext<'a, H, ITy> {
    pub(crate) fn new(
        interpreter: &'a mut Interpreter<ITy>,
        host: &'a mut H,
        ip: *const u8,
    ) -> Self {
        Self {
            inner: InstructionContext::new(interpreter, host),
            ip,
        }
    }

    pub(crate) fn pre_step(&mut self) -> u8 {
        let opcode = unsafe { *self.ip };
        self.ip = unsafe { self.ip.add(1) };
        opcode
    }

    pub(crate) fn flush(&mut self) {
        self.inner.interpreter.bytecode.set_ip(self.ip);
    }
}

#[allow(refining_impl_trait)] // Keeping the `impl` in returns requires adding lifetime bounds.
impl<'a, H, I> crate::InstructionContextTr for TailInstructionContext<'a, H, I>
where
    H: Host + ?Sized,
    I: InterpreterTypes,
{
    fn runtime_flag(&self) -> &I::RuntimeFlag {
        self.inner.runtime_flag()
    }
    fn stack(&mut self) -> &mut I::Stack {
        self.inner.stack()
    }
    fn input(&mut self) -> &mut I::Input {
        self.inner.input()
    }
    fn bytecode(&mut self) -> &mut I::Bytecode {
        self.inner.bytecode()
    }
    fn return_data(&mut self) -> &mut I::ReturnData {
        self.inner.return_data()
    }

    fn gas(&self) -> &Gas {
        self.inner.gas()
    }
    fn record_gas_cost(&mut self, cost: u64) -> bool {
        self.inner.record_gas_cost(cost)
    }
    fn record_refund(&mut self, refund: i64) {
        self.inner.record_refund(refund);
    }

    fn halt(&mut self, result: InstructionResult) -> InstructionReturn {
        self.inner.halt(result)
    }

    fn memory(&mut self) -> &mut I::Memory {
        self.inner.memory()
    }
    fn resize_memory(&mut self, offset: usize, len: usize) -> bool {
        self.inner.resize_memory(offset, len)
    }

    fn host(&mut self) -> &mut H {
        self.inner.host()
    }
}
