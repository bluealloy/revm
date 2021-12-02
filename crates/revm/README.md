# revm - Revolutionary Machine

Is **Rust Ethereum Virtual Machine** with great name that is focused on **speed** and **simplicity**. It gets ispiration from `SputnikVM` (got opcodes/machine from here), `OpenEthereum` and `Geth` with a help from [wolflo/evm-opcodes](https://github.com/wolflo/evm-opcodes). This is probably one of the fastest implementation of EVM, from const EVM Spec to optimistic changelogs for subroutines to merging `eip2929` in EVM state so that it can be accesses only once are some of the things that are improving the speed of execution. 

Here is list of things that i would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of the most important things and most decision are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

## Usage

Please check `bins/revm-test` for simple use case.

All ethereum state tests can be found `bins/revm-ethereum-tests` and can be run with `cargo run --release -- all`

Example with creating simple set/get smartcontract and calling create and two calls:
```rust
    let caller = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
    // InMemoryDB is db that implements Database trait.
    // add one account and some eth for testing.
    let mut evm = revm::new();
    evm.database(InMemoryDB::new());
    evm.db().unwrap().insert_cache(
        caller.clone(),
        AccountInfo {
            nonce: 1,
            balance: U256::from(10000000),
            code: None,
            code_hash: KECCAK_EMPTY,
        },
    );

    // execution globals block hash/gas_limit/coinbase/timestamp..
    evm.env.tx.caller = caller.clone();

    evm.env.tx.transact_to = TransactTo::create();
    evm.env.tx.data = Bytes::from(hex::decode("608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100d9565b60405180910390f35b610073600480360381019061006e919061009d565b61007e565b005b60008054905090565b8060008190555050565b60008135905061009781610103565b92915050565b6000602082840312156100b3576100b26100fe565b5b60006100c184828501610088565b91505092915050565b6100d3816100f4565b82525050565b60006020820190506100ee60008301846100ca565b92915050565b6000819050919050565b600080fd5b61010c816100f4565b811461011757600080fd5b5056fea2646970667358221220404e37f487a89a932dca5e77faaf6ca2de3b991f93d230604b1b8daaef64766264736f6c63430008070033").unwrap());

    let (_, out, _) = evm.transact();
    let contract_address = match out {
        TransactOut::Create(_, Some(add)) => add,
        _ => panic!("not gona happen"),
    };

    evm.env.tx.transact_to = TransactTo::Call(contract_address);
    evm.env.tx.data =
        hex::decode("6057361d0000000000000000000000000000000000000000000000000000000000000015")
            .unwrap()
            .into();
    evm.transact();

    evm.env.tx.transact_to = TransactTo::Call(contract_address);
    evm.env.tx.data = hex::decode("2e64cec1").unwrap().into();

    let (_, out, _) = evm.transact();

    println!("get value (StaticCall): {:?}\n", out);
```
## Status of project

I just started this project as a hobby to kill some time. Presenty it has good structure and I would like to finish it and make it functional but we will see how far we will go. If you want to use this project be free to contact me and we can talk. There are a lot of things that still needs to be done, here are some of TODO's that could be added:

- integrate ethereum state tests: 98% Berlin test passed.
- Need to cleanup code and do some refactoring
- Write a lot of comments and explanations.
- Add MemoryCache for Database interface.
- Write a lot of rust tests
- wasm interface
- C++ interface
## Project structure:

The structure of the project is getting crystallized and we can see few parts that are worthy to write about:
- `Spec` contains a specification of Ethereum standard. It is made as a trait so that it can be optimized away by the compiler
- `opcodes` have one main function `eval` and takes `Machine`, `EVM Host`, `Spec` and `opcode` and depending on opcode it does calculation or for various opcodes it call `Host` for subroutine handling. This is where execution happens and where we cancluate gas consumption.
- `machine` contains memory and execution stack of smart contracts. It calls opcode for execution and contains `step` function. It reads the contract, extracts opcodes and handles memory.
- `subroutine` for various calls/creates we need to have separate `machine` and separate accessed locations. This is place where all of this is done, additionaly, it contains all caches of accessed accounts/slots/code. EIP2929 related access is integrated into state memory. Getting inside new call `subroutine` creates checkpoint that contain needed information that can revert state if subcall reverts or needs to be discardet. Changeset is made so it is optimistic that means that we dont do any work if call is finished successfully and only do something when it fials. 
- `EVM`- Is main entry to the lib,it  implements `Host` and connects `subroutine` and `machine` and does `subroutine checkpoint` switches.


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
