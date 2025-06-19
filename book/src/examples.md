# Examples

REVM comes with several examples that show different ways to use and extend the EVM. Each example focuses on a specific feature or use case.

## Basic Examples

### Custom EVM Implementation

The [my_evm example](https://github.com/bluealloy/revm/tree/main/examples/my_evm) shows how to create a basic custom EVM. This is a good starting point if you want to:

- Learn how REVM components fit together
- Create your own EVM variant
- Understand the handler pattern

```rust,ignore
// Create a custom EVM with your own handler
let mut my_evm = MyEvm::new(Context::mainnet(), ());
let result = MyHandler::default().run(&mut my_evm);
```

### Database Components

The [database_components example](https://github.com/bluealloy/revm/tree/main/examples/database_components) demonstrates how to split your database into separate state and block hash components. This is useful when:

- You need different storage backends for different data types
- You want to implement custom caching strategies
- You're building a modular blockchain client

## Advanced Examples

### Custom Opcodes

The [custom_opcodes example](https://github.com/bluealloy/revm/tree/main/examples/custom_opcodes) shows how to add new opcodes to the EVM. In this example, a static jump opcode is implemented that:

- Reads a 16-bit offset from the bytecode
- Performs a relative jump to that offset
- Works alongside existing EVM opcodes

```rust,ignore
// Define your custom opcode
const MY_STATIC_JUMP: u8 = 0x0C;

// Add it to the instruction set
instructions.insert_instruction(
    MY_STATIC_JUMP,
    |ctx| {
        let offset = ctx.interpreter.bytecode.read_i16();
        ctx.interpreter.bytecode.relative_jump(offset as isize);
    },
);
```

### ERC20 Gas Payment

The [erc20_gas example](https://github.com/bluealloy/revm/tree/main/examples/erc20_gas) demonstrates how to modify the EVM to accept ERC20 tokens as gas payment instead of ETH. This involves:

- Creating a custom handler that modifies gas payment logic
- Transferring tokens from the sender to a treasury
- Maintaining compatibility with standard EVM execution

This example is particularly useful for:
- Layer 2 solutions that want to use custom tokens
- Private chains with different economic models
- Understanding how to modify core EVM behavior

## Running the Examples

To run any example:

```bash
# Run the my_evm example
cargo run --example my_evm

# Run the custom opcodes example
cargo run --example custom_opcodes

# Run the ERC20 gas example
cargo run --example erc20_gas
```

## Learning Path

If you're new to REVM, we recommend exploring the examples in this order:

1. **my_evm** - Understand the basic structure
2. **database_components** - Learn about the database interface
3. **custom_opcodes** - See how to extend the instruction set
4. **erc20_gas** - Understand advanced handler customization

Each example includes detailed comments explaining what's happening and why. They're designed to be both educational and practical starting points for your own projects.