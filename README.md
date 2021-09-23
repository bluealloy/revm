# revm - Revolutionary Machine

Is **Rust Ethereum Virtual Machine** with great name that is focused on **speed** and **simplicity**. It gets ispiration from SputnikVM (opcodes/machine are copied from here), OpenEthereum and Geth.

I love when I see that project has list of postulates that it follows, it can give outsider good feel on how maintainers are making decision and what is important for them. Here is list of them for **revm**:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of most important things and most decision are made to complement this.
- **Simplification** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **wasm** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript.

## TODOs

project is just starting and there are a lot of things that needs to be done. Here is incomplete list of TODO's that we can implement:
- integrate ethereum consensus tests
- Write a lot of rust tests
- Write a lot of comments and explanations.
- wasm interface
- C++ interface


## Project structure:

The structure of the project is getting crystallized and we can see few parts that are worthy to write about:
- `Spec` contains a specification of Ethereum standard. It is made as a trait so that it can be optimized away by the compiler
- `opcodes` have one main function `eval` and takes `Machine`, `EVM Handler`, `Spec` and `opcode` and depending on opcode it does calculation or for various opcodes it call `Handler` for subroutine handling. This is where execution happens and where we cancluate gas consumption.
- `machine` contains memory and execution stack of smart contracts. It calls opcode for execution and contains `step` function. It reads the contract, extracts opcodes and handles memory.
- `subroutine` for various calls/creates we need to have separate `machine` and separate accessed locations. This is place where all of this is done, additionaly, it contains all caches of accessed accounts/slots/code. EIP2929 related access is integrated into state memory. Getting inside new call `subroutine` creates checkpoint that contain needed information that can revert state if subcall reverts or needs to be discardet. Changeset is made so it is optimistic that means that we dont do any work if call is finished successfully and only do something when it fials. 
- `EVM`- Is main entry to the lib,it  implements `Handler` and connects `subroutine` and `machine` and does `subroutine checkpoint` switches.