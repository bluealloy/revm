# Revme

Is a binary that allows running statetest and eof validation.

```bash, ignore
$: revme --help
Usage: revme <COMMAND>

Commands:
  statetest       Execute Ethereum state tests
  evm             Run arbitrary EVM bytecode
  bytecode        Print the structure of an EVM bytecode
  bench           Run bench from specified list
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Running eth tests

Eth tests are a suite of tests from the Ethereum Foundation that are used to test EVM implementations.
Part of these tests are included in the revm repository in the `tests` folder.

Test suites for the latest hardforks can be found in EEST releases https://github.com/ethereum/execution-spec-tests/releases, and there are additional tests that cover older hardforks in https://github.com/ethereum/legacytests

Revm can run statetest type of tests with `revme` using the following command:
`cargo run --release -p revme -- statetest folder_path`

For running EEST tests, we can use  the `./scripts/run-tests.sh.`

For legacy tests, we need to first to download the repo `git clone https://github.com/ethereum/legacytests` and run then run it with `cargo run --release -p revme -- statetest legacytests/Cancun/GeneralStateTests `
All statetest that can be run by revme can be found in the `GeneralStateTests` folder.

## Understanding State Tests

State tests are JSON files that test EVM implementations. Each test contains:

- **Environment**: Block information like number, timestamp, gas limit, and coinbase
- **Pre-state**: Initial accounts with their balances, nonces, code, and storage
- **Transaction**: The transaction to execute, including sender, recipient, value, and data
- **Post-state**: Expected results after execution for different Ethereum versions

A state test can have multiple expected results for different hardforks. For example, a test might have different outcomes for Berlin, London, and Cancun hardforks.

### State Test Structure

Here's what a typical state test looks like:

```json
{
  "testname": {
    "env": {
      "currentNumber": "1",
      "currentTimestamp": "1000",
      "currentGasLimit": "1000000",
      "currentCoinbase": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba"
    },
    "pre": {
      "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
        "balance": "1000000000000000000",
        "nonce": "0",
        "code": "",
        "storage": {}
      }
    },
    "transaction": {
      "gasLimit": ["21000"],
      "gasPrice": "10",
      "nonce": "0",
      "to": "0x095e7baea6a6c7c4c2dfeb977efac326af552d87",
      "value": ["100000"],
      "data": [""],
      "secretKey": "0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"
    },
    "post": {
      "Cancun": [{
        "hash": "0x...",
        "indexes": { "data": 0, "gas": 0, "value": 0 }
      }]
    }
  }
}
```

### Running State Tests

When you run state tests with revme, it:

1. Sets up the environment and pre-state
2. Executes the transaction
3. Compares the result with the expected post-state
4. Reports any mismatches

State tests are essential for ensuring REVM correctly implements Ethereum's behavior across all hardforks.
