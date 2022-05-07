# Rust EVM executor or short REVME

It is still work in progress and it is published for only reason to take cargo name.

This is binary crate that executed evm multiple ways. Currently there are three parts:
* statetest: takes path to folder where ethereum statetest json can be found. It recursively searches for all json files and execute them. This is how i run all https://github.com/ethereum/tests to check if revm is compliant. Example `revme statests test/GenericEvmTest/`
* debug (WIP): 
    * (WIP) Interactive debugger with ability to change any parameter of EVM in runtime, specify breakpoints and do everything what you expect from debugger.
    * (WIP) Allow inserting accounts,balances,storages.
    * Specify web3 interface to bind database, for example use infura with option `--web infura_link`.
    * (WIP) revert opcode implemented in EVM we can enable `rewind` of contract call so that you can go to start of contract call and start debugging again. We could even add `rewind opcode` besically rewind call and rerun it until program counter matches.
    * (TODO) Specify EVM environment from file or from cli.
* run (TODO): Intention is to be able to specify contract bytes and input and run it. It is useful for testing and benchmarks


It is still WIP,but  debugger can be very interesting, it gives you ability to step, modify stack/memory, set breakpoints and do everything what you would expect from standard debugger, with addition of rewinding step and contract calls. You can connect to exteranl web3 supported API and fetch for example live state from mainnet via infura, or you can set data local manupulation, either way this should be useful.

This binary will be console based and interaction will be done via console inputs, this is great showcase this is first step.

Example of commands WIP:
* `help` :)
* `step`
* `continue`
* `exit`
* `stepin`
* `stepout`
* `breakpoint <contract> <pc>`
* `rewind`
    * `rewind call`
    * `rewind opcode`
* `print`
    * `print all`
    * `print stack`
    * `print opcode`
    * `...`
* `state`
* `state <index> <value>`
* `account <address>`
* `account <address> balance <new_balance>`
* `account <address> nonce <new_nonce>`
* `storage <index>`
* `storage set <index> <value>`
* `...`