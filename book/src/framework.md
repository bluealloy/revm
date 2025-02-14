# Revm Framework

Creating new variant of Revm is done by using a framework that is provided by the project. The framework is a set of traits that can be overridden to create a new logic that is needed. Overriding those traits can add new precompile or omit/change some logic of the EVM.

The main traits that are used for creating a new variant are:

Steps to implement a new variant:
1. Include Revm
2. Add main Evm type `MyEvm` and give it a Context, Precompile, Instructions and optionally Inspector.
3. Implement EvmTr for `MyEvm`.
4. Create a empty `MyEvmHandler` type and implementHandlerTr for it. Override any logic that you want.
5. Now you can use `MyEvmHandler` to run your own `MyEvm` instance. Other steps are more for agronomics and easier usage.
6. Create a `MyContextBuilder` trait that will build your context with default types you want to use.
7. Create a `MyEvmBuilder` trait that will build your Evm with default instructions and precompiles.
8. Implement `ExecuteEvm`, `ExecuteCommitEvm` to have same interface as any other variant of EVM.

