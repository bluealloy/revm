# Account extension representation comparison — 2026-09-08

## Main results: 32-byte extensions

Median time changes against current Bytes, five focused rounds:

| Workload | ThinArc | EcoBytes | Option<Arc<[u8]>> |
| --- | ---: | ---: | ---: |
| Clone/drop 4,096 account infos | -36.9% | -36.6% | -37.4% |
| copy_without_code, 4,096 accounts | -46.3% | -46.9% | -42.2% |
| 256-transaction blocks, BAL off | -6.2 to -10.2% | -2.8 to -6.5% | -3.3 to -9.0% |
| 256-transaction blocks, BAL on | -13.4 to -18.1% | -9.1 to -16.5% | -11.1 to -17.1% |
| BAL replay, 16 extension writes | -15.3% | -19.6% | -14.6% |
| Unchanged extension BAL update | -8.5% | -4.4% | -8.5% |
| Changed extension BAL update | -4.7% | -1.0% | -6.2% |

Block ranges above span the four destination/storage workload combinations, not
confidence intervals. ThinArc and Arc improved every populated block variant in
every focused round. EcoBytes had one effectively flat pairing (+0.05%) in the
repeated-destination storage workload. The clone improvements were particularly
stable: ThinArc's paired range was -37.17 to -36.74%, EcoBytes -36.77 to -36.16%,
and Arc -37.62 to -37.30%. Account cloning took 55.75 us for Bytes versus
35.20/35.33/34.90 us for ThinArc/EcoBytes/Arc.

For 256 transfers to distinct destinations with BAL, independent median times were
278.48 us (Bytes), 232.97 us (ThinArc), 242.55 us (EcoBytes), and 234.28 us (Arc).
ThinArc and Arc are close enough that the measurements do not establish one as
universally faster. Ratios of independently computed medians need not equal the
median of paired ratios used in the percentage tables.

Cold/resident journal-load results were substantially noisier than cloning.
ThinArc cold loads had a median -3.8% with a range of -9.1 to +0.02%; its resident
loads had -0.6% with -7.6 to +2.1%. Do not treat these as guaranteed improvements.

### Construction is the tradeoff

Median times for the isolated 32-byte storage value:

| Operation | Bytes | ThinArc | EcoBytes | Option<Arc<[u8]>> |
| --- | ---: | ---: | ---: | ---: |
| Copy from slice, including result drop | 6.76 ns | 8.16 ns | 8.19 ns | 16.24 ns |
| Convert existing Vec, excluding output drop | 3.68 ns | 24.39 ns | 26.21 ns | 25.05 ns |
| Steady clone/drop | 8.77 ns | 7.58 ns | 7.29 ns | 10.00 ns |

Converting an already allocated Vec was about 6.6x/7.1x/6.8x as expensive for the
compact variants. That is a roughly 21–23 ns absolute increase, not an equivalent
percentage loss in block throughput. Fewer lifetime allocations do not imply
faster construction: fresh Bytes need not initialize shared ownership until cloned,
while the other types construct reference-counted storage immediately. Arc's scalar
clone/drop is slower even though cloning vectors of account infos is much faster;
scalar and embedded-account benchmarks measure different generated code.

### Empty-extension control and attribution

The wrapped Bytes control retains a 120-byte AccountInfo but explicitly avoids
comparing two empty byte slices. Empty-account equality fell by about 98.6% for
that control and all three alternatives. In the empty, distinct-destination BAL
block, wrapped Bytes improved 23.2%, compared with 28.7%/25.9%/28.9% for the
compact representations. Much of that benefit does not require changing storage.

Wrapped Bytes does not explain the populated cloning gain: its 4,096-account clone
time changed only -0.24%. It nevertheless improves populated BAL blocks by about
8–12%, partly because those workloads can also process empty default accounts and
because wrapping changes generated code. Compare to both controls, not just Bytes.

For the original empty-extension EXTCODEHASH_50 benchmark, ThinArc improved 6.6%
(paired range -7.7 to -5.8%), EcoBytes 4.8% (-5.8 to +1.8%), and Arc 4.4%
(-5.2 to -3.6%). Wrapped Bytes was flat. The individual-commit benchmark was noisy;
batched commits improved 2.3% for ThinArc consistently, while EcoBytes/Arc were
approximately flat. These runs compare to current Bytes, not origin/main, and do
not establish that the original pre-extension performance has been restored.

### Counterexamples: not a blanket speedup

Five longer rounds of the original control benchmarks showed:

| Workload (empty extensions) | ThinArc | EcoBytes | Arc |
| --- | ---: | ---: | ---: |
| burntpix | +1.9% | +1.5% | +1.0%, noisy |
| snailtracer | +3.9% | +1.8%, noisy | +0.6%, noisy |
| 1,000 nested subcalls | +7.9% | +7.3% | +9.0% |
| 1,000 calls to the same account | +8.4% | +9.7% | +9.6% |
| 1,000 value-transferring subcalls | approximately flat | +7.9% | +9.2% |

ThinArc's snailtracer range was +1.1 to +4.6%. Nested and same-account subcalls
were slower for all three alternatives in every control round. The wrapped Bytes
control did not consistently show those call regressions. Account representation
changes affect whole-program code generation and host result layouts; this
investigation does not isolate the cause of each control regression.

The dedicated follow-up confirms the regressions with **32-byte extensions**:

| Workload | ThinArc | EcoBytes | Arc |
| --- | ---: | ---: | ---: |
| 1,000 nested subcalls | +11.9% | +12.2% | +10.6% |
| 1,000 calls to the same account | +10.7% | +9.7% | +9.4% |
| 1,000 value-transferring subcalls | +4.6% | +7.4% | +5.0% |

Every compact variant was slower in every paired round for all three populated
subcall cases. ThinArc's nested-call range was +9.7 to +12.8%; its value-call
range was +3.6 to +4.9%. Wrapped Bytes had mixed-sign ranges and median changes
of +1.2%, +0.2% and +0.8%, respectively. The separate binary's empty controls also
regressed, although magnitudes differ from the original control binary. In
particular, ThinArc's value calls were flat in the original binary and +4.3% in
this one. Whole-program/code-layout sensitivity warrants caution about exact
percentages; the repeated nested/same-account regressions are not confined to
empty extensions.

## Question and method

Compare the PR's current Bytes representation against ThinArc, EcoBytes, and
Option<Arc<[u8]>>, assuming populated extensions are **32 bytes**. Empty extensions
are the default-chain control. The `wrapped` Bytes build controls for explicit
empty equality and the experimental wrapper without reducing inline size.

All five builds start at `6da6cdc736be6e209902e4de8f5301dbfc40b33b`, with identical
account benchmark source, pinned dependencies, lockfile and features. Only the
experimental storage wrapper, AccountInfo field and BAL element type differ.
The comparison does not replace bytecode Bytes or other byte buffers.

Environment: AMD EPYC 4585PX, CPU 2 pinned, performance governor, Rust 1.96.1,
profiling profile (optimized, thin LTO, debug info), native x86-64 Linux/glibc
2.39. No target-cpu=native or GLIBC_TUNABLES override. The machine was not
exclusively reserved and the SMT sibling was not isolated. Runs were serialized,
revision order rotated/reversed, and no builds ran during measurement.

The broad screen used three rounds, 50 ms warmup, 150 ms measurement and 20
samples. The focused confirmation used five rounds, 100 ms warmup, 300 ms
measurement and 30 samples. Focused suite order was sorted identically across
builds; broad suite order followed compiler-artifact emission. Comparisons use
the median of paired per-round Criterion mean ratios. Negative percentages mean
less time. Round ranges describe repeatability, not population confidence bounds.
The broad filter selected 98 cases per build; focused confirmation selected 32.
Six existing controls were separately rerun for five rounds with 400 ms measurement
and 100 ms warmup. A separate six-case subcall binary repeats the existing call
fixtures with both empty and 32-byte payloads, including success checks before
timing, also over five rounds at those settings.

Raw samples/logs are retained at `/tmp/revm-repr-broad`,
`/tmp/revm-repr-focused`, `/tmp/revm-repr-controls` and `/tmp/revm-repr-subcalls`.
Aggregate CSVs accompany this report in [data](data): comparisons and per-round
means for each run, plus allocation counts and layout output for every variant. The
[README](README.md) documents reproducible preparation, builds and invocation.

## Memory layout

Measured sizes in bytes on this target:

| Type | Bytes | Wrapped Bytes | ThinArc | EcoBytes | Option<Arc<[u8]>> |
| --- | ---: | ---: | ---: | ---: | ---: |
| Extension field | 32 | 32 | 8 | 16 | 16 |
| AccountInfo | 120 | 120 | 96 | 104 | 104 |
| Option<AccountInfo> | 120 | 120 | 104 | 112 | 112 |
| Cow<AccountInfo> | 120 | 120 | 104 | 112 | 112 |
| AccountInfoLoad | 128 | 128 | 112 | 120 | 120 |
| Account | 168 | 168 | 144 | 152 | 152 |
| AccountInfoBal | 96 | 96 | 96 | 96 | 96 |
| AccountBal | 120 | 120 | 120 | 120 | 120 |
| BundleAccount | 280 | 280 | 248 | 264 | 264 |

The BAL container itself does not shrink: it still contains a Vec for extension
writes. Its populated elements shrink. BundleAccount's change is not simply twice
the AccountInfo change because Option layout/niches also matter. These are type
sizes, not measured RSS or allocator footprint.

The compact representations lose the niche that lets the Bytes-based AccountInfo
fit its Option/Cow discriminator without additional space. They remain smaller,
but these enums now need eight bytes beyond AccountInfo itself. Host account-load
results therefore change layout too; shrinking the field is not simply removing
24 bytes from every enclosing type. This is a structural observation, not proof
that it causes the observed subcall regressions.

## Allocations and construction semantics

Allocation/reallocation calls measured with the actual experimental storage types:

| Operation, 32-byte payload | Bytes / wrapped | ThinArc / EcoBytes / Arc |
| --- | ---: | ---: |
| Copy from borrowed slice | 1 | 1 |
| First clone after that construction | 1 | 0 |
| Subsequent clone | 0 | 0 |
| Convert existing Vec (input allocation excluded) | 0 | 1 |
| First clone after Vec conversion | 1 | 0 |

Empty slice/Vec construction and cloning used zero allocations for all variants.
The current Bytes construction path promotes shared ownership on its first clone;
it is incorrect to describe every Bytes clone as allocating. Once shared, all
these representations clone without payload copying.

The Vec conversion distinction matters in reth-core: its existing encoder first
produces a Vec. The compact prototypes must allocate and copy that payload into
their own header-plus-data allocation; Bytes can adopt it. Including the input
Vec, conversion plus a first clone totals two allocation calls for either approach.
To obtain a true single-allocation encoding path with compact storage, encode
directly into the final allocation. That encoder integration is not part of these
revm benchmarks. Slice-constructor and steady-clone timings include result drop;
the batched Vec-conversion timing excludes output drop but includes freeing the
consumed input when conversion copies it. Steady clone timings do not include
first-clone promotion.

EcoBytes' 15-byte inline capacity provides **no small-payload allocation advantage
at 32 bytes**. Its populated path is reference counted, like the other candidates.

## Coverage and limitations

Block tests use 256 transactions with 32-byte extensions on the caller and each
funded destination, independently allocated per account. They cover one versus
256 destinations, transfers versus slot-zero increments, BAL off/on, state
commit, transition merging and bundle extraction. Absent/system-default accounts
can still have empty extensions. The fixtures drain outputs to bound growth.

Additional populated cases cover 4,096-account clone/drop and copy_without_code,
cold/resident journal loads, BAL extension-history replay, changed/unchanged
extension updates, independent/shared equality, and construction from slices/Vecs.
The original opcode, transfer, state-hook and create/destroy-commit fixtures retain
empty extensions. Control workloads include interpreter-heavy execution and
nested/same-account/value subcalls. Crypto code was not changed or rebenchmarked.

The wrappers preserve JSON/postcard byte encoding, but deserialization deliberately
uses the existing Bytes decoder then converts. Serialization screening therefore
includes that conversion cost rather than an optimized compact-storage decoder.
The wrapper tests cover bytewise ordering, normalization, JSON and postcard round
trips; state tests and benchmark smoke tests execute before timing.

These are in-memory, single-threaded, synthetic workloads on one compiler/CPU.
They do not measure full-node throughput, disk-backed databases, atomic-refcount
contention between execution threads, or general SDK workloads. A representation
can improve account copying while slowing construction or another execution path.
This investigation only installs the prototypes in disposable worktrees; it does
not select a new production field type or claim public API compatibility.

## Conclusion

ThinArc best meets the inline-size objective: an eight-byte optional handle and
96-byte AccountInfo. It substantially improves copying and these block-shaped
workloads, but is not an unconditional performance improvement: populated nested
calls consistently slowed by about 12%. EcoBytes has no inline-payload advantage
at 32 bytes, and Arc uses more inline space without consistently outperforming
ThinArc. Resolve the call regressions before selecting a production replacement.
Separately, the wrapped Bytes control demonstrates that the empty-equality issue
can be addressed without changing the representation.

To minimize total allocations upstream, compact storage should be paired with
direct encoding into its final allocation. Merely converting the encoder's Vec
does not reduce the allocation count through the first clone.

All five builds passed whole-workspace, all-target, all-feature Clippy with warnings
denied. State tests passed (33 for Bytes, 34 per wrapper), as did 97 account/storage
benchmark smoke cases and six dedicated subcall smoke cases per build.
