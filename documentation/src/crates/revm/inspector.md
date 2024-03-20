# Inspectors

This module contains various inspectors that can be used to execute and monitor transactions on the Ethereum Virtual Machine (EVM) through the `revm` library.

## Overview

There are several built-in inspectors in this module:

- `NoOpInspector`: A basic inspector that does nothing, which can be used when you don't need to monitor transactions.
- `GasInspector`: Monitors the gas usage of transactions.
- `CustomPrintTracer`:
  Traces and prints custom messages during EVM execution.
  Available only when the `std` feature is enabled.
- `TracerEip3155`:
  This is an inspector that conforms to the [EIP-3155](https://eips.ethereum.org/EIPS/eip-3155) standard for tracing Ethereum transactions.
  It's used to generate detailed trace data of transaction execution, which can be useful for debugging, analysis, or for building tools that need to understand the inner workings of Ethereum transactions.
  This is only available when both `std` and `serde` features are enabled.

## Inspector trait

The `Inspector` trait defines methods that are called during various stages of EVM execution.
You can implement this trait to create your own custom inspectors.

Each of these methods is called at different stages of the execution of a transaction.
They can be used to monitor, debug, or modify the execution of the EVM.

For example, the `step` method is called on each step of the interpreter, and the `log` method is called when a log is emitted.

You can implement this trait for a custom database type `DB` that implements the `Database` trait.

## Usage

To use an inspector, you need to implement the `Inspector` trait.
For each method, you can decide what you want to do at each point in the EVM execution.
For example, to capture all `SELFDESTRUCT` operations, implement the `selfdestruct` method.

All methods in the `Inspector` trait are optional to implement; if you do not need specific functionality, you can use the provided default implementations.
