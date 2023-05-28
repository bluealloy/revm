# Interpreter

The interpreter is concerned with the execution of the evm opcodes and serves as the event loop to step through the opcodes. The interpreter is concerned with attributes like gas, contracts, memory, stack, and returning execution results.

The interpreter struct is defined as:

```rust
pub struct Interpreter {
    /// Instruction pointer.
    pub instruction_pointer: *const u8,
    /// Return is main control flag, it tell us if we should continue interpreter or break from it
    pub instruction_result: InstructionResult,
    /// left gas. Memory gas can be found in Memory field.
    pub gas: Gas,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// After call returns, its return data is saved here.
    pub return_data_buffer: Bytes,
    /// Return value.
    pub return_range: Range<usize>,
    /// Is interpreter call static.
    pub is_static: bool,
    /// Contract information and invoking data
    pub contract: Contract,
    /// Memory limit. See [`crate::CfgEnv`].
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
}
```

The interpreter implements the following core methods in addition to a number of getters and constructors. The primary methods are `step`, `run`, and `run_inspect`:

```rust
impl Interpreter {
    ///...
    /// Execute next instruction
    #[inline(always)]
    pub fn step<H: Host, SPEC: Spec>(&mut self, host: &mut H) {
        // step.
        let opcode = unsafe { *self.instruction_pointer };
        // Safety: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };
        eval::<H, SPEC>(opcode, self, host);
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        while self.instruction_result == InstructionResult::Continue {
            self.step::<H, SPEC>(host)
        }
        self.instruction_result
    }

    /// loop steps until we are finished with execution
    pub fn run_inspect<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        while self.instruction_result == InstructionResult::Continue {
            // step
            let ret = host.step(self);
            if ret != InstructionResult::Continue {
                return ret;
            }
            self.step::<H, SPEC>(host);

            // step ends
            let ret = host.step_end(self, self.instruction_result);
            if ret != InstructionResult::Continue {
                return ret;
            }
        }
        self.instruction_result
    }
}
```
