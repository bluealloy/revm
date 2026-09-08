# Comparing account-extension representations

These are **experimental patches**, not a production representation change. The
preparation script only accepts a clean, detached worktree distinct from this
checkout. The shared wrapper template preserves the byte-oriented JSON/postcard
encoding, lexicographic ordering and hashing. Tests check these semantics.

Representations:

- `none`: actual extension-less source, with a benchmark-only setup shim that
  discards extension inputs. Use an extension-less revision, not the PR revision.
- `bytes`: current AccountInfo and BAL implementation, unchanged.
- `wrapped`: a Bytes newtype with explicit empty-equality handling; an attribution
  control for the wrapper, not a smaller representation.
- `thinarc`: normalized `Option<triomphe::ThinArc<(), u8>>`.
- `ecobytes`: `ecow::EcoBytes`.
- `arc`: normalized `Option<alloc::sync::Arc<[u8]>>`, not Arc<Bytes> or Arc<Vec<u8>>.

All experimental builds include the same pinned dependencies and features. The
four wrappers handle empty equality explicitly before comparing their underlying
storage. They otherwise retain each type's native equality, including its shared
pointer shortcut when available. Ordering/hashing use the byte slice, not a thin
allocation's length header. Empty inputs are normalized; empty optional values
never allocate or increment a refcount. Serialization writes hex for human-readable
formats and raw bytes otherwise. Deserialization uses the existing Bytes decoder,
then converts; this convenient compatibility bridge is not an optimized decoder.

## Reproduce

From the revm repository containing these scripts:

```sh
for variant in bytes wrapped thinarc ecobytes arc; do
  git worktree add --detach /tmp/revm-repr-$variant 6da6cdc
  python3 scripts/account-extension-variants/prepare.py /tmp/revm-repr-$variant $variant
done
```

The script copies the current account benchmark into each worktree and installs
the storage microbenchmark, empty/32-byte subcall benchmark and allocation-counting
example. Resolve the two added
dependencies once in the bytes worktree, then copy that lockfile to the others:

```sh
cargo metadata --manifest-path /tmp/revm-repr-bytes/Cargo.toml --format-version 1 > /tmp/repr-metadata.json
for variant in wrapped thinarc ecobytes arc; do
  cp /tmp/revm-repr-bytes/Cargo.lock /tmp/revm-repr-$variant/Cargo.lock
done
```

In each worktree, build with the same toolchain and profile:

```sh
cargo +nightly fmt --all
cargo bench --locked -p revme --profile profiling \
  --bench evm --bench account_extension --bench account_extension_storage \
  --no-run --message-format=json > /tmp/repr-VARIANT-build.jsonl
cargo test --locked --profile profiling -p revm-state --features serde
cargo build --locked --profile profiling -p revme --example account_extension_allocations
cargo bench --locked --profile profiling -p revme \
  --bench account_extension --bench account_extension_storage -- --test
cargo bench --locked --profile profiling -p revme \
  --bench account_extension_subcalls --no-run --message-format=json > /tmp/repr-VARIANT-calls-build.jsonl
cargo bench --locked --profile profiling -p revme --bench account_extension_subcalls -- --test
RUSTFLAGS=-Dwarnings cargo clippy --workspace --all-targets --all-features
target/profiling/examples/account_extension_allocations
```

Use a distinct build-output filename per variant. Do not build while timing:

```sh
python3 scripts/compare-account-benchmarks.py --baseline bytes \
  --build bytes=/tmp/repr-bytes-build.jsonl \
  --build wrapped=/tmp/repr-wrapped-build.jsonl \
  --build thinarc=/tmp/repr-thinarc-build.jsonl \
  --build ecobytes=/tmp/repr-ecobytes-build.jsonl \
  --build arc=/tmp/repr-arc-build.jsonl \
  --output /tmp/repr-comparison --cpu 2 --rounds 5 \
  --warmup 0.1 --measurement 0.3 --samples 30 \
  --filter 'EXTCODEHASH_50|transact_.*1000|^populated32/|^populated/.*/32$|^block/256/payload=32|^block/256/payload=0/bal=true/distinct=true/storage=false$|^account/(equal/empty|clone/4096|copy_without_code/4096)$|^storage/(clone|construct_slice|construct_vec|equal_shared|equal_independent)/32$'
```

The output contains raw Criterion samples, per-round means and paired ratios.
Negative time changes indicate improvements. The scripts do not modify CPU
frequency, SMT settings, or libc dispatch. Use a fresh output directory each time.

For the dedicated subcall confirmation, use the five `calls-build.jsonl` files
instead, omit `--filter`, set `--measurement 0.4`, and choose a new output directory.
For the original controls, use the original build files with
`--filter '^analysis$|^burntpix$|^snailtracer$|^subcall_'` and `--measurement 0.4`.
See [results](results.md) for the measured outcomes and saved CSVs.

### Extension-less comparison

The follow-up includes current main (`0837b5e`, fetched 2026-09-08) and the PR's
merge base (`1382fa0`). Both have an actual 88-byte AccountInfo without the field.
Main also contains an unrelated call-frame gas-accounting fix, so the merge-base
control distinguishes that revision difference from extension overhead.

```sh
git worktree add --detach /tmp/revm-repr-nofield-main 0837b5e
git worktree add --detach /tmp/revm-repr-nofield-base 1382fa0
for revision in main base; do
  python3 scripts/account-extension-variants/prepare.py /tmp/revm-repr-nofield-$revision none
  cp /tmp/revm-repr-bytes/Cargo.lock /tmp/revm-repr-nofield-$revision/Cargo.lock
done
```

Build these with `--bench evm --bench account_extension --bench
account_extension_subcalls`, using the same profiling settings above. There is no
storage benchmark for an absent field. Include all four benchmark targets when
building extension-bearing variants, or combine their two existing compiler logs:

```sh
jq -c . /tmp/repr-bytes-build.jsonl /tmp/repr-bytes-calls-build.jsonl > /tmp/repr-bytes-combined-build.jsonl
```

Pass all seven builds to the comparison script with `--baseline main`. The exact
filter and settings are recorded in `data/nofield-config.json`. All timing samples
are fresh; previous-run timings are not reused. Extension-only BAL writes and
storage operations are omitted for `none`, not timed as no-ops. The `payload=32`
names identify corresponding fixtures: the extension-less versions execute the
same transactions but cannot retain that payload. Zero-/32-byte setup is outside
the timed loops.

## Scope

The populated working assumption is **32-byte extensions**. Empty extensions are
retained as the default-chain control. The account suite still contains older
256-/4,096-byte fixtures, but the comparison filters exclude those payload sizes.

Block-shaped tests populate each funded account independently, then execute 256
transactions with BAL off/on, one versus 256 destinations, and transfers versus
storage increments. Bundle/BAL outputs are drained each iteration. The new
`populated32` cases cover account cloning, code-free copying, journal loads and
replay of extension writes. Original opcode and transfer benchmarks retain empty
extensions; they test the effect of representation size on the default path.
The dedicated subcall target copies the existing nested/same-account/value-call
fixtures, populates all their explicit accounts with either zero or 32 bytes, and
checks transaction success before timing.

Construction from a slice includes allocating the final buffer and copying the
payload. Construction from Vec excludes creating the input Vec but includes any
copy/allocation needed to convert it. The wrappers intentionally do not claim
zero-copy conversion from Bytes or Vec. The allocation example reports allocation
and reallocation calls, not heap bytes or allocator size classes; input setup and
printing are excluded. The first and second clones are counted separately because
Bytes may allocate ownership metadata only on the first clone.

These prototypes are not drop-in public API proposals: for example, their setter
returns the experimental wrapper instead of Bytes. They isolate internal storage
choices and compatibility behavior needed for performance comparison.
