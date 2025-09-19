# Custom Frame with FrameTr Implementation Example

This example demonstrates how to create custom frames that properly implement the `FrameTr` trait and integrate with REVM's EVM trait system.

## Overview

REVM uses frames to manage the execution context of EVM operations. This example shows how to:

1. **Implement FrameTr trait**: Create custom frames that properly implement the `FrameTr` trait
2. **Extend frame functionality**: Add custom tracking and metrics to frames
3. **Integrate with EvmTr**: Demonstrate how custom frames work within the EVM trait system
4. **Use trait objects**: Show polymorphic frame handling capabilities

## Key Features

- **Gas metering**: Track gas consumption per frame
- **Call depth tracking**: Monitor the depth of nested calls
- **Execution timing**: Measure how long each frame takes to execute
- **Statistics collection**: Aggregate metrics across multiple frame executions
- **Trait compliance**: Proper implementation of REVM's `FrameTr` trait
- **Type safety**: Full integration with REVM's type system

## Key Components

### CustomFrame

A custom frame implementation that extends REVM's standard frame structure:
- Implements the `FrameTr` trait properly
- Adds execution tracking (gas usage, timing, call depth)
- Provides logging and debugging capabilities
- Maintains compatibility with REVM's frame system

### CustomEvm

A simplified EVM implementation that demonstrates:
- Working with custom frames that implement `FrameTr`
- Frame stack management
- Statistics collection
- Integration with REVM's trait system

### FrameManager & FrameFactory

Utilities for creating and managing custom frames:
- Automated frame creation with tracking
- Interpreter setup and configuration
- Frame tagging and identification

## Running the Example

```bash
cargo run -p example-cframe
```

## Running Tests

```bash
cargo test -p example-cframe
```

## Example Output

The example demonstrates several key concepts:

1. **FrameTr Implementation**: Shows that CustomFrame properly implements the trait
2. **Frame Factory Usage**: Automated frame creation with custom tracking
3. **EVM Integration**: Custom frames working within a frame stack
4. **Trait Objects**: Using CustomFrame as polymorphic FrameTr objects
5. **Statistics Collection**: Comprehensive metrics gathering across executions

## Use Cases

This pattern is essential for:

- **Custom EVM Variants**: Building specialized EVMs like Optimism's op-revm
- **Debugging & Profiling**: Advanced execution tracing and performance analysis
- **Transaction Simulation**: Detailed execution monitoring and gas estimation
- **Research & Development**: Experimental EVM features and optimizations
- **Compliance & Auditing**: Detailed execution logs and metrics collection

## Architecture Integration

This example shows the correct way to extend REVM's frame system:

1. **Trait Implementation**: Properly implementing `FrameTr` for type safety
2. **EVM Integration**: Working within REVM's `EvmTr` ecosystem
3. **Polymorphism**: Using trait objects for flexible frame handling
4. **Extension Points**: Adding custom functionality without breaking compatibility

The approach demonstrated here is used by major REVM-based projects for extending EVM functionality while maintaining compatibility with the core REVM architecture.