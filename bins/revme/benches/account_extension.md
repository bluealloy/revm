# Account extension performance investigation

The fixtures in `account_extension.rs` use APIs available before the extension was
added. Copy the **same source** into each checkout when comparing revisions. The
nonempty-extension cases use serde during setup and skip revisions that ignore the
field; those cases are payload-scaling measurements, not before/after comparisons.

## Reproduce

Build each checkout with the same Rust toolchain and `Cargo.lock`:

```sh
cargo bench -p revme --profile profiling --bench evm --bench account_extension \
  --no-run --message-format=json > /tmp/head-build.jsonl
cargo build -p revme --profile profiling --example account_extension_perf
cargo bench -p revm-precompile --profile profiling --bench bench \
  --no-run --message-format=json > /tmp/head-crypto-build.jsonl
```

Repeat for `main` and optionally the merge base, changing the output paths. Do not
build while measuring. Then, from this checkout:

```sh
python3 scripts/compare-account-benchmarks.py \
  --build main=/tmp/main-build.jsonl --build base=/tmp/base-build.jsonl \
  --build head=/tmp/head-build.jsonl --output /tmp/account-comparison \
  --cpu 2 --rounds 7 --warmup 0.1 --measurement 0.3 --samples 30
```

Use `--filter REGEX` for targeted runs. Use the crypto build JSON files to run the
existing precompile controls. Each output directory must be new. The runner pins
processes to one CPU, serializes execution, rotates/reverses revision order, and
retains logs, Criterion samples, per-round means, and paired comparisons. Its
bootstrap interval resamples *rounds*, not individual Criterion samples; with a
small number of rounds, the full paired range is more informative than a claim of
statistical significance. Positive changes mean **more time**, not less throughput.

For fixed-work confirmation independent of Criterion's adaptive iteration counts:

```sh
python3 scripts/compare-account-perf.py \
  --binary main=/path/to/main/target/profiling/examples/account_extension_perf \
  --binary base=/path/to/base/target/profiling/examples/account_extension_perf \
  --binary head=/path/to/head/target/profiling/examples/account_extension_perf \
  --output /tmp/account-perf --cpu 2 --rounds 5
```

This requires Linux `taskset` and permission to use `perf`. It records cycles,
instructions, branches, and branch misses. Hardware counters include process and
fixture setup; the example's elapsed time excludes setup. One iteration means one
transaction for `extcodehash`, or 1,000 transactions for `commit` and `batch`.
The batching boundary intentionally matches the existing benchmark, including
committing after transaction zero.

## Coverage and why it matters

The account path is not limited to an opcode checking whether an account is empty:

1. `Database::basic` returns `AccountInfo`. An in-memory/cache-backed database
   clones it. `JournalInner::load_account_mut_optional` creates an `Account` and
   its original-info snapshot. Reusing a journal across transactions refreshes
   that snapshot on the first access in the next transaction.
2. Host account loads return `AccountInfoLoad`, which contains a `Cow<AccountInfo>`.
   A borrowed load need not clone, but increasing the owned variant changes the
   result layout and generated calling/move/drop code. BALANCE, EXTCODESIZE,
   EXTCODEHASH, and call-family instructions share this machinery. The journal
   computes state-clear-aware emptiness; EXTCODEHASH also checks `is_empty` itself.
3. Checkpoints record transfers and storage changes. Reverting a nested call does
   not necessarily copy whole accounts, so it needs a separate control rather
   than assuming every state-related operation regresses equally.
4. `State::commit` updates the BAL, if enabled, then the cache and transition
   state. The no-hook path consumes accounts; the hook path applies borrowed
   accounts before passing ownership to the hook. Bundling and reverting preserve
   current/original account infos in additional structures.
5. BAL account updates clone original information. `AccountInfoBal::update` now
   clones and compares extension bytes even when both are empty and no extension
   write is recorded. Replay adds another lookup; `BalWrites::get` uses linear
   search below five writes and binary search otherwise. Canonical Ethereum BAL
   conversion does not include the extension, but handles larger internal records.

| Fixtures | Coverage |
| --- | --- |
| Existing `evm` suite | 151 cases: the three CodSpeed alerts, opcode samples, transfers, nested/same-account/value subcalls, bytecode analysis, interpreter-heavy workloads, inspection, EVM construction |
| Existing precompile suite | 73 crypto-heavy controls that do not operate on accounts |
| Account primitives | Empty/funded emptiness and identity, hash, clone/drop, `copy_without_code`, contiguous balance scans at 64/4,096/65,536 accounts, JSON encode/decode |
| Journal | Newly loaded versus already-resident accounts, 64/4,096-account working sets, transfer/checkpoint reverts |
| Opcode transactions | 256 BALANCE/EXTCODESIZE/EXTCODEHASH operations; one reused versus 256 distinct addresses; missing versus funded accounts |
| Block-shaped execution | 256 transactions; one versus 256 destinations; transfers versus slot-zero increments; BAL off/on; commit, transition merge, bundle extraction |
| State transitions | 256 accounts with four changed storage slots; update/create/destroy; state hook off/on; commit, merge, bundle revert |
| BAL primitives | Unchanged versus changed updates, replay across linear/binary search sizes, clone/drop, canonical conversion |
| Nonempty extensions | Independently allocated 32/256/4,096-byte equal payloads and last-byte differences; clone and unchanged/changed BAL updates |

Opcode and block-shaped fixtures explicitly select Cancun, avoiding unrelated
changes to default-fork accounting. A successful execution is checked before
timing. Their databases remain in memory and evolve across iterations; these are
steady-state execution workloads, not disk-I/O or full-node replay benchmarks.
BAL and bundle outputs are drained each block to keep growth bounded. State
transition setup is excluded with `iter_batched`; it exercises commit processing
for created/selfdestructed accounts, not CREATE/SELFDESTRUCT bytecode execution.
Clone benchmarks use infos without loaded bytecode to isolate account data and
include destruction of their outputs; their large ratios should not be applied
unchanged to accounts with additional code-cloning work. BAL-update microbenchmarks
include present-info construction and vector allocation/drop when writes occur.

## Experiment: 2026-09-07

Revisions measured before adding these benchmark files:

| Label | Revision | Purpose |
| --- | --- | --- |
| main | `0837b5eb8bc61b86eb3fd3ab38a412e1405571a9` | Freshly fetched `origin/main` |
| base | `1382fa0322d92af08aa323dcd6140ba48ff5b9ab` | Merge base; isolates the extension changes from subsequent main changes |
| head | `550dfd3cbe8eecfe916d81e1df5b048ee5c54cdc` | PR #3894, concrete `Bytes` extension after the generic experiment was reverted |

`main` includes an unrelated state-gas-refill fix after the merge base. Comparing
only main/head would therefore not completely isolate this PR. All checkouts used
the PR's identical lockfile and identical fixture source within each experiment.

Environment: AMD EPYC 4585PX (Zen 5), CPU 2 pinned, SMT sibling 18; Linux x86-64;
performance governor; Rust 1.96.1 (`31fca3adb`, LLVM 22.1.2); glibc
2.39-0ubuntu8.8; `profiling` profile (optimized, thin LTO, debug info), without
`target-cpu=native`. The machine was not exclusively reserved and its SMT sibling
was not isolated. Revision order was alternated to reduce, not eliminate, noise.

The initial broad screen used three rounds, 50 ms warmup, 100 ms measurement, and
10 samples. The focused confirmation used seven rounds, 100 ms warmup, 300 ms
measurement, and 30 samples. Six state-transition fixtures were added between the
broad and focused builds; within each run, all revisions used identical fixtures.
Fixed-work confirmation used five rounds. Raw results live in
`/tmp/revm-perf-{broad,focused,fixed,crypto}` on the investigation machine; the
adjacent `account_extension_results/` directory retains aggregate results and the
fixed-work per-round times in git. `noavx512.csv` is the separate libc diagnostic,
not part of the default-environment results.

The [CodSpeed comment](https://github.com/bluealloy/revm/pull/3894#issuecomment-5570155823)
reported efficiency changes of -9.39% for EXTCODEHASH_50, -6.34% for committing
1,000 transactions, and -4.61% for committing every 40 transactions. Its simulation
run also warned that the comparison used different runtime environments. Native
wall times below test repeatability on this host; they do not reproduce CodSpeed's
simulated absolute times, and time increases are not numerically interchangeable
with efficiency decreases.

## Results

### The three CodSpeed alerts

All changes below compare head to main. Ranges are the smallest/largest paired
time increases across rounds, not confidence intervals.

| Workload | Criterion, seven rounds | Fixed work, five rounds | Retired instructions, fixed work |
| --- | ---: | ---: | ---: |
| EXTCODEHASH_50 | +34.1% (+33.2 to +35.5%) | +37.5% (+35.6 to +38.8%) | +21.7% |
| Commit each of 1,000 transactions | +4.2% (+3.3 to +8.1%) | +11.1% (+10.8 to +13.3%) | +5.7% |
| Commit every 40 transactions | +1.7% (-1.9 to +4.0%) | +5.2% (+4.1 to +10.3%) | +3.7% |

**EXTCODEHASH and individual commits clearly repeat.** Batched commits are too
noisy to establish a slowdown from Criterion alone, but are slower in all five
fixed-work rounds, with extra retired instructions. The fixed-work comparisons
against the merge base are also positive in every round: median +38.3%, +10.7%,
and +4.9%, respectively. These are not just unrelated changes on main.

Criterion median times were 1.98 to 2.65 microseconds for EXTCODEHASH_50,
438 to 458 microseconds for individual commits, and 346 to 352 microseconds
for batched commits. The percentage column uses the median of paired ratios,
so it need not equal the ratio of those independently computed median times.

The fixed-work harness includes transaction cloning in its timed loop, whereas
the existing Criterion fixtures batch-clone inputs outside timing. Its binary
also has different code layout. It corroborates the *direction* of the regression;
the differing percentages are a warning against treating one harness as a
universal throughput estimate. Counter medians show increased cycles of 37.7%,
11.0%, and 5.0%, respectively; increased instruction counts make CPU frequency
alone an inadequate explanation.

The original EXTCODEHASH sample also executes CREATE and reuses its journal
between transactions. The new opcode-only fixtures independently regress:
about +81% for 256 accesses to one address, +39% for distinct funded accounts,
and +28% for distinct missing accounts. All seven paired rounds are slower.

### Additional slow paths

Focused results, again head versus main with **empty extensions**:

| Workload | Median increase | Interpretation |
| --- | ---: | --- |
| Clone/drop 4,096 account infos | +344% (4.44x) | 6.84 to 30.48 microseconds; larger handles plus clone/drop work |
| `copy_without_code`, 4,096 accounts | +398% (4.98x) | 5.42 to 26.96 microseconds; excluding code does not exclude extension work |
| Scan balances, 65,536 accounts | +13.9% | Larger records matter without touching the extension |
| Cold journal load, 4,096 accounts | +17.3% | Database/account snapshot and larger working-set costs |
| Warm journal load, 4,096 accounts | +1.1%, range -0.8 to +11.7% | Not a consistently demonstrated regression |
| BAL replay, 16 prior balance writes | +41.1% | Additional empty extension lookup and account clone/drop; tiny absolute operation |
| 256-transaction blocks, BAL off | +11.9 to +16.6% | Four destination/storage combinations; all seven rounds slower |
| Same block variants, BAL on | +36.7 to +60.5% | Additional per-account extension clone/compare; libc-sensitive |
| Commit/merge/revert updated accounts | +10.2 to +11.1% | Hook on/off both regress |
| Commit/merge/revert created accounts | +11.5 to +13.8% | Synthetic state-commit creation path |
| Commit/merge/revert destroyed accounts | +35.1 to +37.9% | Synthetic state-commit destruction path; 256 accounts, four slots each |

The largest microbenchmarks are **not** representative block percentages:
empty account equality grows from 1.08 to 82.06 ns (about 76x); 1,024 unchanged
BAL updates grow from 2.64 to 96.52 microseconds (about 36.5x). When balance
writes are also recorded, the same BAL-update fixture grows from 11.25 to
100.92 microseconds (about 8.8x). See the libc diagnosis below before generalizing
these magnitudes to another system.

The three-round screen additionally found BAL clone/drop around +60%, JSON
encoding/decoding around +34%/+35%, and repeated-address BALANCE around +64%.
These did not receive the seven-round confirmation and should be treated as
screening results. JSON now includes the empty extension field, so that comparison
also includes increased encoded data, not merely a layout change.

There is no uniform slowdown: the screen's journal transfer/checkpoint-revert
cases were approximately flat, some nested-call fixtures improved, and compute
controls such as snailtracer were approximately flat. Record layout and generated
code can produce improvements as well as regressions. Adding the state-transition
fixtures changed some block timings between the broad and focused binaries;
the BAL regressions were present in both, but their exact magnitude is sensitive
to the linked benchmark program.

All 73 existing crypto cases were measured across three rounds on all three
revisions. The largest median time increase against main was 1.02%; the median
across cases was -0.07%. This is a useful negative control: the account-path
regressions are not mirrored by a broad machine-wide crypto slowdown.
One modexp case improved by 5.46%; the controls are not all numerically identical.

Nonempty payloads show a different cost shape. On head, cloning a 32-byte or
4,096-byte owned extension takes about 21 ns in both cases (shared ownership,
not a deep payload copy). Comparing independent equal payloads grows from
1.75 to 26.34 ns; recording a changed extension grows from 27.32 to 48.03 ns.
The last-byte-difference fixture intentionally requires examining the whole
payload. These are head-only measurements, not a claim of compatibility or
relative slowdown against accounts that could not carry extensions.

## Memory and code-level costs

Observed `size_of` values (bytes, this target):

| Type | main/base | head | Increase |
| --- | ---: | ---: | ---: |
| AccountInfo | 88 | 120 | 36.4% |
| Account | 136 | 168 | 23.5% |
| AccountInfoBal | 72 | 96 | 33.3% |
| AccountBal | 96 | 120 | 25.0% |
| BundleAccount | 232 | 280 | 20.7% |

An empty `Bytes` does not allocate a payload, but it still occupies 32 bytes.
Empty clone/drop uses the bytes handle's implementation; populated owned bytes
also involve shared ownership. Larger account records and original snapshots
increase copying, cache footprint, and memory traffic. These costs remain even if
extension equality is optimized. Actual process RSS also includes hash-table
capacity, boxes, storage, and allocator overhead; these sizes are not an RSS model.

### Empty equality has a libc-dependent amplification

On this host, `perf record` attributed 99.87% of cycles in the isolated empty
account equality benchmark to glibc's `__memcmp_evex_movbe`. Annotated samples
concentrated at its masked vector load/compare sequence for lengths at most 32.
The new extension comparison reaches byte-slice equality, which the compiler
lowers to a libc comparison even for a runtime length of zero. This is not a
payload allocation or a 4-KiB comparison. It is also not evidence that the same
absolute penalty occurs with every CPU, libc, or compiler.

A standalone optimized Rust probe compared two runtime-empty slices 10 million
times with `black_box` on both inputs and the result. With the default empty
slice's pointer (`0x1`), equality took 79.24 ns; with zero-length slices backed by
a mapped stack array, it took 5.26 ns. Disabling glibc's AVX-512 selection made
both approximately 1.13 ns. This isolates the empty-slice comparison from revm
and strongly implicates the masked access to the empty slice's sentinel address
as the local amplifier. It does not require reading any extension payload.

The probe's core is:

```rust
let mapped = [42u8; 1];
for empty in [&[][..], &mapped[..0]] {
    for _ in 0..10_000_000 {
        std::hint::black_box(
            std::hint::black_box(empty) == std::hint::black_box(empty),
        );
    }
}
```

Rerunning four actual fixtures for three rounds with the same binaries and
`GLIBC_TUNABLES=glibc.cpu.hwcaps=-AVX512VL,-AVX512BW` gave:

| Fixture | Default head time | Diagnostic head time | Diagnostic head/main difference |
| --- | ---: | ---: | ---: |
| Empty account equality | 82.06 ns | 1.61 ns | +49.6% |
| 1,024 unchanged BAL updates | 96.52 us | 14.86 us | +208.1% (3.08x) |
| BAL-enabled 256 transfers to one destination | 233.99 us | 204.54 us | +12.6% |
| EXTCODEHASH_50 | 2.65 us | 2.92 us | +35.5% |

Changing libc dispatch affects other comparisons too: for example, the baseline
BAL block changed from 146.35 to 180.45 us. Consequently this is **not** an estimate
of the speedup from an empty-extension fast-path patch. It establishes that much
of the extreme empty-equality/BAL result depends on the libc path, while a
substantial residual cost remains. EXTCODEHASH is independently slower with either
path. Its default-environment profile put 34.1% of cycles in the EXTCODEHASH
instruction, 15.9% in its account-load helper and 9.7% in journal account loading;
libc comparison was not a leading sampled hotspot there.

## What to investigate next

No production implementation was changed in this investigation. The evidence
suggests three distinct optimization targets:

1. Avoid clone/compare work for two empty extensions in BAL updates, and avoid
   sending empty-extension identity checks to libc. Recheck equality, ordering,
   hashing and BAL semantics before implementing fast paths; an empty-to-nonempty
   or nonempty-to-empty transition must still be tracked.
2. Reduce account cloning and the cost of enlarged host account-load results.
   Empty equality alone cannot fix the independently reproduced opcode slowdown
   or the extra retired instructions.
3. Evaluate account/snapshot memory footprint with a production-sized state and
   full-node replay, including BAL-enabled blocks. The observed record growth is
   unconditional even when the extension payload is empty.

## Limits

These measurements cover one CPU family, compiler, feature configuration and
allocator/libc combination, primarily in-memory, single-threaded execution. They
do not establish whole-node throughput, multi-threaded refcount contention,
disk-backed database performance, larger-than-LLC account sets, every fork, or
arbitrary downstream SDK extension workloads. Full-node replay and another
microarchitecture are the appropriate follow-ups, not multiplying a microbenchmark
ratio by total node runtime. A short-screen regression is a lead, not proof that
every chain or every workload pays that percentage.
