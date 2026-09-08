# Comparison including extension-less revm — 2026-09-08

This follow-up reruns all five storage builds alongside two actual extension-less
revisions. Populated extensions are 32 bytes. The comparison includes an absent
field, not an empty Bytes value or a zero-sized generic extension.

## Results

Median time in **microseconds**, lower is better. Populated cases carry 32 bytes
in extension-bearing builds and no payload in the extension-less builds.
Empty and populated rows are separately compiled fixtures; small timing differences
between them in the no-field builds are not effects of retaining a payload.

| Workload | No field: main | No field: base | Bytes | Wrapped Bytes | ThinArc | EcoBytes | Option<Arc<[u8]>> |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Clone/drop 4,096 accounts, empty | 7.60 | 7.63 | 30.74 | 30.81 | 8.34 | 9.61 | 8.52 |
| copy_without_code, 4,096 accounts, empty | 5.45 | 5.44 | 26.99 | 26.95 | 6.29 | 6.97 | 8.67 |
| Clone/drop 4,096 accounts, populated | 7.09 | 7.07 | 55.78 | 55.78 | 35.19 | 35.41 | 34.91 |
| copy_without_code, 4,096 accounts, populated | 5.45 | 5.44 | 63.39 | 63.41 | 34.08 | 33.69 | 36.62 |
| Cold journal loads, 4,096 accounts, populated | 349.12 | 346.73 | 439.58 | 447.53 | 429.81 | 442.06 | 425.28 |
| Resident journal loads, 4,096 accounts, populated | 60.88 | 60.92 | 61.52 | 61.12 | 61.89 | 62.93 | 62.26 |
| 256 transfers, distinct destinations, BAL on, empty | 198.38 | 198.73 | 298.69 | 228.34 | 206.75 | 217.72 | 209.65 |
| 256 transfers, distinct destinations, BAL off, populated | 175.00 | 173.34 | 222.94 | 215.60 | 195.30 | 203.61 | 198.48 |
| 256 transfers, distinct destinations, BAL on, populated | 199.03 | 198.43 | 281.37 | 256.48 | 230.32 | 239.72 | 235.43 |
| 1,000 nested calls, populated | 371.16 | 360.94 | 328.53 | 330.80 | 360.84 | 361.86 | 362.03 |
| 1,000 same-account calls, populated | 194.59 | 190.31 | 172.56 | 174.73 | 189.37 | 190.48 | 188.46 |
| 1,000 value-transferring calls, populated | 200.87 | 200.43 | 180.58 | 182.58 | 187.20 | 192.60 | 190.23 |
| EXTCODEHASH_50, empty | 1.98 | 2.01 | 2.68 | 2.69 | 2.50 | 2.54 | 2.52 |

The absent-field cloning baseline is cheap: these fixtures have no bytecode and
no extension ownership to clone. Compact storage recovers much of Bytes' cost,
but does not eliminate the cost of supporting shared populated extensions.
Populated vector cloning is about **5x** the extension-less time for compact
storage and **7.9x** for Bytes. ThinArc's paired increase is +394.1 to +399.0%,
versus +681.6 to +691.6% for Bytes. Populated copy_without_code is roughly
6.2–6.7x for compact storage versus 11.6x for Bytes.

Empty-account cloning is much closer to upstream with compact storage: ThinArc
is +9.8%, EcoBytes +26.8% and Arc +11.6%, versus +304.1% for Bytes. This is
distinct from claiming zero overhead for default-chain accounts.

### All populated block combinations

Median paired time change relative to **extension-less main**:

| BAL | Destinations | Operation | Bytes | Wrapped Bytes | ThinArc | EcoBytes | Arc |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| off | repeated | transfer | +13.8% | +15.5% | +6.3% | +9.0% | +6.6% |
| off | repeated | storage increment | +14.8% | +13.6% | +6.5% | +12.5% | +12.3% |
| off | distinct | transfer | +26.3% | +24.3% | +11.2% | +16.6% | +15.1% |
| off | distinct | storage increment | +21.1% | +21.0% | +10.8% | +16.2% | +14.0% |
| on | repeated | transfer | +36.0% | +21.7% | +10.6% | +12.5% | +12.2% |
| on | repeated | storage increment | +30.3% | +17.6% | +9.7% | +16.1% | +13.8% |
| on | distinct | transfer | +40.6% | +28.8% | +15.5% | +21.6% | +17.6% |
| on | distinct | storage increment | +29.3% | +19.6% | +11.7% | +17.6% | +16.1% |

Every extension-bearing build is slower than main in every populated block
combination in every paired round. ThinArc has the lowest median overhead among
the populated storage candidates in all eight combinations. This is synthetic
in-memory block processing, not a measurement of full-node throughput.

For the empty distinct-destination BAL transfer block, ThinArc reduces Bytes'
+49.1% overhead to +4.6%. Wrapped Bytes alone reduces it to +14.3%, reinforcing
that explicit empty equality can recover much of that workload's cost without
changing storage. Not all gains can be attributed to inline size.

### Calls and other controls

The earlier call regressions were **relative to Bytes**, not extension-less revm.
In this run, populated nested calls have median changes against main of -9.8%
for Bytes, -3.6% for ThinArc, -2.5% for EcoBytes and -2.5% for Arc. ThinArc's
range is -5.1 to -1.2%; Arc's is -4.0 to +0.5%. Compact storage is not uniformly
faster than main in every call pairing, although all three populated call
fixtures have lower median times for every storage variant.

Against Bytes, the compact variants' populated nested-call slowdowns repeat in
all five rounds: medians +8.9%, +8.8% and +11.4% for ThinArc/EcoBytes/Arc.
Same-account medians are +9.6%, +10.3% and +9.3%; ThinArc has one slightly faster
pairing (-0.4%). Value-call medians are +3.5%, +6.4% and +5.6%, with mixed-sign
ranges. Thus the nested-call tradeoff remains, but not every earlier repeatability
claim holds in this fresh, seven-way run.

The original empty nested/same-account call controls are approximately flat
against main for compact storage (mixed-sign ranges), while Bytes is 6–7% faster.
Whole-program code generation/layout changes can produce non-monotonic results;
these tests do not prove that carrying extension data inherently improves calls.
Main versus base also has mixed-sign call ranges, so their modest median
differences cannot be confidently attributed to the intervening gas-accounting fix.

EXTCODEHASH_50 still regresses substantially: Bytes +34.7%, ThinArc +26.5%,
EcoBytes +29.0% and Arc +26.7% against main, consistently across all rounds.
The compact changes therefore do **not** restore the original opcode performance.
Interpreter analysis, burntpix and snailtracer have small, mixed-sign changes.
Batched transaction commits are approximately flat with compact storage;
individual commits range from ThinArc -1.1% (noisy) to EcoBytes +5.1%.

### Bottom line

ThinArc remains the strongest inline-size candidate and recovers much of Bytes'
account-copy and block overhead. It does not return revm to its extension-less
cost: populated blocks retain about 6–15% median overhead here, populated copying
still carries substantial ownership cost, and EXTCODEHASH remains about 27% slower.
Conversely, the call slowdowns relative to Bytes should not be presented as a
general regression relative to extension-less main. The choice remains workload
dependent; these measurements do not justify an unconditional production switch.

## Method

- `main`: `0837b5eb8bc61b86eb3fd3ab38a412e1405571a9`, fetched over HTTPS on
  2026-09-08. This is current extension-less upstream.
- `base`: `1382fa0`, the PR's extension-less merge base. Main has additional
  call-frame gas-accounting changes, so this is the matched revision control.
- Bytes, wrapped Bytes, ThinArc, EcoBytes and Arc: the same experimental builds
  as the preceding report, based on `6da6cdc`. Their binaries are reused, but
  **all timing samples are new** and interleaved with the extension-less builds.

All seven builds use the same Cargo.lock, dependency features, Rust 1.96.1 and
profiling profile (optimized, thin LTO). Runs are serial, pinned to CPU 2 on the
same AMD EPYC 4585PX host with performance governor. No native CPU-target or libc
dispatch override; SMT sibling and machine are not exclusively reserved.

Five rounds, 100 ms warmup, 300 ms measurement, 30 Criterion samples per case.
Build order rotates/reverses between rounds, with identical sorted suite order.
No builds run during timing. Absolute times are medians of the five per-round
Criterion means; percentage changes are medians of paired round ratios, so the
two need not divide exactly. Positive changes mean slower.

The extension-less fixtures discard extension input only during untimed setup.
The account/journal/block/subcall operations otherwise execute the corresponding
workload on the original upstream account types. Labels containing `payload=32`
identify matched fixtures, not payloads secretly stored outside AccountInfo.
Extension-only update/equality/replay and scalar storage operations have no
extension-less equivalent; they are omitted, not benchmarked as no-ops.

The run repeats the prior focused cases plus interpreter and subcall controls:
44 cases per extension-bearing build and 31 per extension-less build, per round.
Raw samples are in `/tmp/revm-repr-nofield-comparison`. Aggregate data and exact
invocation settings are in [data](data): `nofield-measurements.csv`,
`nofield-comparison.csv` and `nofield-config.json`.

## Memory layout

| Type, bytes | No field (main/base) | Bytes | Wrapped Bytes | ThinArc | EcoBytes | Option<Arc<[u8]>> |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Extension field | absent | 32 | 32 | 8 | 16 | 16 |
| AccountInfo | 88 | 120 | 120 | 96 | 104 | 104 |
| Option<AccountInfo> | 96 | 120 | 120 | 104 | 112 | 112 |
| Cow<AccountInfo> | 96 | 120 | 120 | 104 | 112 | 112 |
| AccountInfoLoad | 104 | 128 | 128 | 112 | 120 | 120 |
| Account | 136 | 168 | 168 | 144 | 152 | 152 |
| AccountInfoBal | 72 | 96 | 96 | 96 | 96 | 96 |
| AccountBal | 96 | 120 | 120 | 120 | 120 | 120 |
| BundleAccount | 232 | 280 | 280 | 248 | 264 | 264 |

No field has zero extension allocations or reference-count operations. Empty
extensions also allocate nothing, but still change enclosing layouts and account
operations. Populated shared extensions add refcount work that the absent-field
baseline never needs. Allocation semantics for the four storage types are
unchanged from the [previous report](results.md#allocations-and-construction-semantics).

## Validation

Both extension-less builds passed whole-workspace, all-target, all-feature Clippy
with warnings denied, their 30 state tests, and account/subcall benchmark smoke
tests. The five extension-bearing builds are unchanged from their validated
versions in the preceding report. Preparation modifies benchmark infrastructure,
not the extension-less runtime sources. No production representation is changed.
