# Inspector - EVM Execution Tracing

The Inspector trait is REVM's powerful mechanism for observing and tracing EVM execution. It provides hooks into every aspect of transaction execution, making it ideal for building debuggers, analyzers, and custom tooling.

## What is the Inspector?

The [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) trait defines callbacks that are invoked during EVM execution. It allows you to:

- **Step through execution**: Hook into every opcode before and after execution
- **Monitor state changes**: Track stack, memory, and storage modifications  
- **Trace calls**: Observe contract calls, creations, and their outcomes
- **Capture events**: Record logs, self-destructs, and other EVM events
- **Override behavior**: Optionally modify execution flow and results

## Core Inspector Methods

```rust,ignore
pub trait Inspector<CTX, INTR: InterpreterTypes> {
    // Opcode-level tracing
    fn step(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {}
    fn step_end(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {}
    
    // Call and creation tracing  
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> { None }
    fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {}
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> { None }
    fn create_end(&mut self, context: &mut CTX, inputs: &CreateInputs, outcome: &mut CreateOutcome) {}
    
    // Event tracing
    fn log(&mut self, interp: &mut Interpreter<INTR>, context: &mut CTX) {}
    fn log_without_inspector(&mut self, context &mut CTX, logs: Vec<Log>, logs_range: Range<usize>) {}
    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {}
}
```

## Basic Usage

### 1. Create an Inspector

```rust,ignore
#[derive(Default)]
struct MyInspector {
    gas_used: u64,
    call_count: usize,
}

impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for MyInspector {
    fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
        self.gas_used += interp.gas.spent();
    }
    
    fn call(&mut self, _context: &mut CTX, _inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.call_count += 1;
        None // Don't override the call
    }
}
```

### 2. Use with EVM

```rust,ignore
let inspector = MyInspector::default();
let mut evm = Context::mainnet()
    .with_db(db)
    .build_mainnet_with_inspector(inspector);

// Execute with inspection
let result = evm.inspect_one_tx(tx)?;
println!("Gas used: {}", evm.inspector.gas_used);
println!("Calls made: {}", evm.inspector.call_count);
```

## Advanced Features

### State Inspection
Access complete interpreter state during execution:

```rust,ignore
fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
    let pc = interp.bytecode.pc();
    let opcode = interp.bytecode.opcode();
    let stack_len = interp.stack.len();
    let memory_size = interp.memory.size();
    
    println!("PC: {}, Opcode: 0x{:02x}, Stack: {}, Memory: {}", 
             pc, opcode, stack_len, memory_size);
}
```

### Call Override
Modify execution by returning custom outcomes:

```rust,ignore
fn call(&mut self, _context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
    if inputs.target_address == SPECIAL_ADDRESS {
        // Override this call with custom logic
        return Some(CallOutcome::new(
            InterpreterResult::new(InstructionResult::Return, Bytes::from("custom")),
            0..0
        ));
    }
    None // Let normal execution continue
}
```

### Event Logging
Capture and process EVM events:

```rust,ignore
fn log(&mut self, _interp: &mut Interpreter<INTR>, _ctx: &mut CTX, log: Log) {
    println!("LOG emitted from: {:?}", log.address);
    println!("Topics: {:?}", log.topics());
    println!("Data: {}", hex::encode(log.data.data));
}
```

## Built-in Inspectors

REVM provides several ready-to-use inspectors:

- **`GasInspector`**: Tracks gas consumption throughout execution
- **`TracerEip3155`**: Generates EIP-3155 compatible execution traces
- **`NoOpInspector`**: Default no-operation inspector for when inspection is disabled

## Performance Considerations

- Inspector callbacks have minimal overhead when not implemented (empty default methods)
- Use inspection judiciously in production - detailed tracing can impact performance
- Consider batching inspector data collection for high-throughput scenarios

## Common Use Cases

- **Debuggers**: Step-by-step execution analysis
- **Gas analyzers**: Detailed gas consumption tracking  
- **Security tools**: Detecting suspicious patterns or calls
- **Development tools**: Contract interaction tracing
- **Testing frameworks**: Execution verification and state checking

The Inspector trait makes REVM very observable EVM implementations available, enabling sophisticated tooling and analysis capabilities.