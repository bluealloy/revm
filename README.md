

Hoobby project implementing performant EVM with rust. Focus is on speed.



Structure of project is getting cristalized and we can see few part that are worty to write about:
- `Spec` contains specification of ethereum standard. It is made as trait so that it can be optimized away by compiler
- `opcodes` have one main function `eval` and takes `Machine`, `EVM Handler`, `Spec` and `u8nopcode` and depending on opcode does calculation or for various calls it call `Handler` for subrutine handling. This is where execution happen and where has is calculated for few of simple opcodes.
- `machine` contains memory and execution stack of smart contracts. It calls opcode for execution and contains `step`ing functionality. It reads contract, extracts opcodes and handles memory.
- `subrutine` for various calls/crates we need to have separate `machine` and separate accessed location. This is all done inside `subrutine` additionaly it contains all caches of accessed accounts/slots/code.
- `EVM`- It implements `Handler` and navigates connects `subrutine` and `machine` and does `subrutine checkpoint` switches. It represent entry point of the program.