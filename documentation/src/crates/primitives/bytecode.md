# bytecode

Entrusted with tasks related to EVM bytecode, handling operations like parsing, interpretation, and transformation.
This module defines structures and methods to manipulate Ethereum bytecode and manage its state. It's built around three main components: `JumpMap`, `BytecodeState`, and `Bytecode`.

The `JumpMap` structure stores a map of valid `jump` destinations within a given Ethereum bytecode sequence. It is essentially an `Arc` (Atomic Reference Counter) wrapping a `BitVec` (bit vector), which can be accessed and modified using the defined methods, such as `as_slice()`, `from_slice()`, and `is_valid()`.

The `BytecodeState` is an enumeration, capturing the three possible states of the bytecode: `Raw`, `Checked`, and `Analysed`. In the `Checked` and `Analysed` states, additional data is provided, such as the length of the bytecode and, in the `Analysed` state, a `JumpMap`.

The `Bytecode` struct holds the actual bytecode, its hash, and its current state (`BytecodeState`). It provides several methods to interact with the bytecode, such as getting the length of the bytecode, checking if it's empty, retrieving its state, and converting the bytecode to a checked state. It also provides methods to create new instances of the `Bytecode` struct in different states.
