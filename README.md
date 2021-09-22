

Hobby project implementing performant EVM with rust. The focus is on speed.

The structure of the project is getting crystallized and we can see few parts that are worthy to write about:
- `Spec` contains a specification of Ethereum standard. It is made as a trait so that it can be optimized away by the compiler
- `opcodes` have one main function `eval` and takes `Machine`, `EVM Handler`, `Spec` and `u8nopcode` and depending on opcode does calculation or for various calls it call `Handler` for subroutine handling. This is where execution happens and where has is calculated for a few of simple opcodes.
- `machine` contains memory and execution stack of smart contracts. It calls opcode for execution and contains `step` function. It reads the contract, extracts opcodes and handles memory.
- `subroutine` for various calls/crates we need to have separate `machine` and separate accessed locations. This is all done inside `subroutine` additionaly it contains all caches of accessed accounts/slots/code.
- `EVM`- It implements `Handler` and navigates connects `subroutine` and `machine` and does `subroutine checkpoint` switches. It represents the entry point of the program.