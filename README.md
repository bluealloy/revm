# revm - Revolutionary Machine

Is **Rust Ethereum Virtual Machine** with great name that is focused on **speed** and **simplicity**. It gets ispiration from SputnikVM (opcodes/machine are copied from here), OpenEthereum and Geth. This is probably one of the fasted implementation of EVM, from const EVM Spec to optimistic changelogs for subroutines to merging eip2929 in EVM state so that it can be accesses only once are some of the things that are improving the speed of execution. 

Here is list of things that i would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of the most important things and most decision are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

## Status of project

I just started this project as a hobby to kill some time. Presenty it has good structure and I would like to finish it and make it functional but we will see how far we will go. There are a lot of things that still needs to be done, here are some of TODO's that could be added:

- integrate ethereum consensus tests
- Write a lot of comments and explanations.
- Add MemoryCache for Database interface.
- Write a lot of rust tests
- wasm interface
- C++ interface

## Project structure:

The structure of the project is getting crystallized and we can see few parts that are worthy to write about:
- `Spec` contains a specification of Ethereum standard. It is made as a trait so that it can be optimized away by the compiler
- `opcodes` have one main function `eval` and takes `Machine`, `EVM Handler`, `Spec` and `opcode` and depending on opcode it does calculation or for various opcodes it call `Handler` for subroutine handling. This is where execution happens and where we cancluate gas consumption.
- `machine` contains memory and execution stack of smart contracts. It calls opcode for execution and contains `step` function. It reads the contract, extracts opcodes and handles memory.
- `subroutine` for various calls/creates we need to have separate `machine` and separate accessed locations. This is place where all of this is done, additionaly, it contains all caches of accessed accounts/slots/code. EIP2929 related access is integrated into state memory. Getting inside new call `subroutine` creates checkpoint that contain needed information that can revert state if subcall reverts or needs to be discardet. Changeset is made so it is optimistic that means that we dont do any work if call is finished successfully and only do something when it fials. 
- `EVM`- Is main entry to the lib,it  implements `Handler` and connects `subroutine` and `machine` and does `subroutine checkpoint` switches.


### Subroutine

Changelogs are created in every subroutine and represent action that needs to happen so that present state can be reverted to state before subroutine. It contains list of accounts with original values that can be used to revert current state to state before this subroutine started.

Depending on subroutine and if account was previously loaded/destryoyed, accounts in changelog can be:
- LoadedCold -> when reverting, remove account from state.
- Dirty(_) -> account is already hot, and in this subroutine we are changing it. Field needed for that are:
        - original_slot: HashMap<H256,SlotChangeLog>:
            SlotChangeLog can be: ColdLoad or Dirty(H256)
        - info: (original balance/nonce/code)
        - was_cold: bool
- Destroyed(Account) -> swap all Info and Storage from current state