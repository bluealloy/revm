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

## Memory Model

Revm, the Rust Virtual Machine, represents its memory as a vector of bytes (Vec<u8>), providing an abstraction for sequential memory operations. It allows you to perform common tasks like querying memory length, resizing memory, obtaining memory slices, and manipulating individual bytes or larger data chunks (e.g., 256-bit integers). The Memory object is initialized with a capacity of 4KiB.

A key aspect of this memory model is the safe manipulation of memory blocks through various helper methods. The resize method, for example, adjusts the memory size and ensures that new memory regions are zero-filled. The set methods allow placing data into memory at specific offsets, with the set_data method even allowing for more complex memory setting scenarios involving multiple offsets and lengths. The get_slice method retrieves a specific chunk of memory. Additionally, a utility function next_multiple_of_32 is provided for rounding up memory sizes to the nearest multiple of 32. As a general safety rule, users are expected to ensure the validity of offsets when invoking certain methods, like set_byte.

## Gas

The interpreter includes a Gas struct that keeps track of various gas measurements for Revm. It includes fields for gas limit, used gas, memory gas, and refunded gas. Here's a quick example of how to use the API exposed by the Gas struct:

```rust
let mut gas = Gas::new(1000); // Initializing with a limit of 1000
gas.record_cost(100); // Recording a cost of 100
println!("Remaining gas: {}", gas.remaining()); // Prints "Remaining gas: 900"
gas.record_refund(50); // Recording a refund of 50
println!("Refunded gas: {}", gas.refunded()); // Prints "Refunded gas: 50"
```

The Gas struct has various methods to manipulate and query its state. For instance, record_cost and record_memory methods are used to account for used gas, with the latter focusing on gas used for memory expansion. The erase_cost method is used to remove a cost that was previously added. The record_refund and gas_refund methods handle refunded gas, with gas_refund allowing for negative values. The spend method gives the total used gas while remaining returns the remaining gas available. Through these operations, the Gas struct provides a comprehensive model for gas accounting within the Revm system.
