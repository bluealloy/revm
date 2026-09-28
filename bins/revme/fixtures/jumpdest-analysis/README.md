# CREATE jumpdest analysis

Blockchain fixtures for [execution-specs PR #3631](https://github.com/ethereum/execution-specs/pull/3631),
pinned to [bd87e232119fc298475260c4d27a77506c1fda01](https://github.com/ethereum/execution-specs/blob/bd87e232119fc298475260c4d27a77506c1fda01/tests/benchmark/compute/scenario/test_mix_operations.py).

Run all patterns from the repository root:

```sh
cargo run --release -p revme -- btest bins/revme/fixtures/jumpdest-analysis
```

Run the random STOP/JUMPDEST pattern alone:

```sh
cargo run --release -p revme -- btest bins/revme/fixtures/jumpdest-analysis/random_stop_jumpdest.json
```

Each fixture executes an Amsterdam block with a 60,000,000-gas workload,
split into four transactions to respect the 16,777,216 transaction gas cap.
The loop repeatedly executes CREATE with 128 KiB of initcode. That initcode
jumps directly to its final JUMPDEST and then stops, making analysis the
dominant work. The loop mixes each returned CREATE address into memory so
successive initcodes differ. It eventually runs out of gas, reverting the
contract creations; transaction fees and sender nonces still persist.

The four random patterns use Python's `random.Random(0).choices`, exactly as
upstream. Their bytes are copied from two 64 KiB pre-allocated contracts using
EXTCODECOPY, avoiding the calldata floor. The nine periodic controls tile a
1 KiB calldata window. CREATE targets are pre-funded with one wei to avoid
Amsterdam's new-account state-gas charge.

| Fixture | Initcode body |
| --- | --- |
| `random_stop_jumpdest_push1.json` | Uniform STOP, JUMPDEST, PUSH1 |
| `random_stop_jumpdest.json` | Uniform STOP, JUMPDEST |
| `random_jumpdest_push1.json` | Uniform JUMPDEST, PUSH1 |
| `random_stop_jumpdest_2push1.json` | STOP, JUMPDEST, PUSH1 with weights 1:1:2 |
| `00.json`, `5b.json` | Repeated STOP or JUMPDEST |
| `605b.json`, `615b5b.json` | Repeated PUSH1 or PUSH2 with JUMPDEST immediates |
| `605b5b.json`, `615b5b5b.json` | PUSH1 or PUSH2 followed by JUMPDEST |
| `e65b.json`, `e75b.json`, `e85b.json` | Repeated DUPN, SWAPN or EXCHANGE tiles |

These are execution workloads, not isolated bytecode-analysis microbenchmarks.
REVME also parses fixtures, initializes state, and validates receipts and
post-state. Use the same release binary and machine when comparing patterns.

## Regeneration

With `uv`, `curl`, and `tar` installed:

```sh
./scripts/generate-jumpdest-analysis.sh
```

The script downloads the pinned upstream source into `target/`, installs its
locked Python dependencies, and uses EELS to compute expected results
independently of REVM. `JOBS` controls filler parallelism (default: 4).
Filling is CPU-intensive because the Python reference implementation performs
the full analysis for every CREATE.

The only test-source adaptation is
`include_full_post_state_in_output=True`: upstream benchmarks normally retain
only the post-state hash, whereas REVME's blockchain runner uses `postState`
for account checks. The script separates the generated fixtures into one JSON
file per pattern and adds provenance under `_info`; it does not change the
bytecode, transaction budgets, allocations, receipts, or expected state.
