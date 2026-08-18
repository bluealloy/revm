# Cold SSTORE accounting: pre-Amsterdam vs EIP-8038 misalignment

## Summary

SSTORE cold-access gas accounting is structurally misaligned between pre-Amsterdam
forks and Amsterdam (EIP-8038), and the misalignment is **not a test bug** — it is
consensus-enshrined EIP-2929 behavior. Aligning revm's SSTORE accounting with the
SLOAD model (warm base always charged, cold adds only the premium above warm)
reproduces Amsterdam/glamsterdam behavior **exactly**, but undercharges every cold
SSTORE on Berlin-through-Osaka by 100 gas, breaking ~8,000 mainnet-fork fixtures.

The aligned model is:

```text
SSTORE cost = sstore_static (warm base, always)          // 100
            + cold premium if cold                       // COLD_STORAGE_ACCESS - WARM_ACCESS = 2000
            + write surcharge if first change            // fork-dependent
```

This is exactly how revm prices SLOAD on every fork since Berlin, and exactly how
EIP-8038 prices SSTORE. Pre-Amsterdam SSTORE does **not** follow it: EIP-2929
charges the **full** `COLD_SLOAD_COST` (2100) *in addition to* the warm-inclusive
base costs, so on a cold access the warm base is effectively paid twice.

## Spec evidence (execution-specs, `glam-8` branch)

Osaka (`src/ethereum/forks/osaka/vm/instructions/storage.py`) — the cold charge is
**additive**, and the no-op/dirty branch still adds the warm access on top:

```python
if (target, key) not in evm.accessed_storage_keys:
    gas_cost += GasCosts.COLD_STORAGE_ACCESS          # 2100, additive

if original_value == current_value and current_value != new_value:
    if original_value == 0:
        gas_cost += GasCosts.STORAGE_SET              # 20000
    else:
        gas_cost += COLD_STORAGE_WRITE - COLD_STORAGE_ACCESS  # 5000 - 2100 = 2900
else:
    gas_cost += GasCosts.WARM_ACCESS                  # 100 — stacks with the 2100 above
```

Amsterdam (`src/ethereum/forks/amsterdam/vm/instructions/storage.py`) — the access
cost is **exclusive** (cold *replaces* warm), identical in structure to SLOAD:

```python
# Access cost: cold or warm, always charged.
if is_cold_access:
    gas_cost += GasCosts.COLD_STORAGE_ACCESS          # 2100
else:
    gas_cost += GasCosts.WARM_ACCESS                  # 100

# Write cost: charged on the first change to the slot this transaction.
if original_value == current_value and current_value != new_value:
    gas_cost += GasCosts.STORAGE_WRITE                # 10000
```

## Cost comparison

| Case | Pre-Amsterdam (spec) | Aligned model | Δ | Amsterdam (spec) | Aligned model | Δ |
|---|---|---|---|---|---|---|
| warm no-op | 100 | 100 | 0 | 100 | 100 | 0 |
| **cold no-op** | **2200** | 2100 | **−100** | 2100 | 2100 | 0 |
| warm set (0→x) | 20000 | 20000 | 0 | 10100 | 10100 | 0 |
| **cold set** | **22100** | 22000 | **−100** | 12100 | 12100 | 0 |
| warm reset (x→y) | 2900 | 2900 | 0 | 10100 | 10100 | 0 |
| **cold reset** | **5000** | 4900 | **−100** | 12100 | 12100 | 0 |

Every cold SSTORE on Berlin..Osaka is exactly 100 gas short under the aligned
model; Amsterdam matches in every case.

Note the pre-Amsterdam quirk is only in the *no-write* branch: the set/reset flat
costs (20000 / 5000-total) already bake the load in, but the `else` branch adds
`WARM_ACCESS` on top of an already-charged full `COLD_SLOAD_COST`. EIP-8038
deliberately cleans this up into `access + write`.

## Step-by-step: stipend, cold load, static and dynamic cost

revm executes SSTORE (`sstore_with_gas_accounting`, host.rs) in this order. `g` is
the frame's remaining gas when the instruction starts.

| # | Step | Amount (pre-Ams / Ams) | On failure | Slot loaded / in BAL? |
|---|---|---|---|---|
| 1 | static-call check | — | `StateChangeDuringStaticCall` | no |
| 2 | EIP-2200 stipend sentry (Istanbul+): `g <= call_stipend` → halt | 2300 / 2300 | `ReentrancySentryOOG`, frame consumes all gas | no |
| 3 | static charge `sstore_static` (the warm/load base) | 100 / 100 | OOG (unreachable: step 2 guarantees `g >= 2301`) | no |
| 4 | `skip_cold_load` check: `g − 100 < cold_storage_additional_cost` → skip the load | threshold 2000 / 2000 | `ColdLoadSkipped` → OOG halt | **no — not warmed, not in BAL** |
| 5 | journal load `sstore_skip_cold_load` | — | DB error | **yes — slot warmed (journaled) + BAL read recorded** |
| 6 | dynamic charge: cold add-on + write surcharge | cold: 2100 / 2000; set: 19900 / 10000; reset: 2800 / 10000 | OOG halt | already loaded: warm mark **reverts** with the journal, **BAL read persists** |
| 7 | EIP-8037 state gas + refill, then refunds | set: — / 245k-class state gas | state-gas OOG | same as 6 |

Two different access structures with opposite revert semantics are touched at step 5:

- **Warm marking** (`accessed_storage_keys`): journaled in revm, per-frame in EELS —
  rolled back when the frame halts or reverts. Order relative to gas charges is
  consensus-invisible.
- **BAL storage read** (Amsterdam): recorded by EELS `get_storage` into
  `tx_state.storage_reads`, which is a shared, never-rolled-back set — EELS
  `state_tracker.py`: *"reads from failed calls still appear in the BAL"*. Order
  relative to gas checks **is** consensus-visible: an access that could not be paid
  for must not be recorded.

### The recording rule and revm/EELS equivalence

EELS Amsterdam decides BAL membership *before* the state access:

```python
check_gas(evm, max(gas_cost, GasCosts.CALL_STIPEND + Uint(1)))  # gas_cost = access cost
# ... only then get_storage(...)  ->  records the BAL read
# ... write cost computed, charge_gas(gas_cost) at the end
```

So the slot is recorded iff `g >= max(access_cost, 2301)` — even if the SSTORE
later OOGs on the write surcharge (EELS charges *all* execution gas after
recording; revm charges the dynamic gas after loading — same observable result).

revm's staged pipeline reproduces this exactly. revm loads iff steps 2 and 4 pass:

```text
g >= 2301                      (sentry, step 2)
g − 100 >= cold_total − 100    (skip check, step 4: additional = cold_total − warm)
⟺  g >= max(2301, cold_total)  =  EELS's  max(access_cost, CALL_STIPEND + 1)
```

The static charge of 100 before the skip check is what makes
`cold_storage_additional_cost` the correct threshold: comparing post-static
remaining against the *premium* is identical to comparing pre-charge gas against
the *full* cold access cost. Using the full `cold_storage_cost` (2100) there would
skip loads that EELS records — off by exactly the warm base.

### Gas windows (Amsterdam, cold slot, first write: 100 + 2000 + 10000 = 12100)

| `g` at SSTORE entry | Outcome | Slot in BAL? |
|---|---|---|
| `g <= 2300` | step 2: `ReentrancySentryOOG` | no |
| `2301 <= g < 12100` | steps 3–5 succeed, step 6 OOG on write | **yes** (matches EELS) |
| `g >= 12100` | success (state gas drawn from reservoir/spill per EIP-8037) | yes |

With current constants the skip window of step 4 is **empty**: the sentry
guarantees `g − 100 >= 2201 > 2000`. It becomes live exactly when a repricing sets
`COLD_STORAGE_ACCESS > CALL_STIPEND + 1`, which the EELS comment anticipates
(*"Post-repricing the access cost can exceed the stipend, so the EIP-2200 stipend
sentry is no longer sufficient on its own"*). For a hypothetical cold access cost
`C > 2301`:

| `g` at SSTORE entry | Outcome | Slot in BAL? |
|---|---|---|
| `g <= 2300` | sentry halt | no |
| `2301 <= g < C` | step 4 skips the load → OOG | **no** (matches EELS `check_gas`) |
| `g >= C` | loaded; write may still OOG at step 6 | yes |

Pre-Amsterdam the same pipeline runs, but there is no BAL and the warm marking is
journaled, so the load/charge ordering is consensus-invisible; the skip is also
unreachable (`cold total 2200 < 2301`), leaving it purely as a
don't-touch-the-DB-on-a-doomed-frame guard.

## Experiment

revm's `sstore_dynamic_gas` was changed to charge `cold_storage_additional_cost`
(2000) for a cold access on **all** forks (no `is_amsterdam` distinction), with the
warm base charged unconditionally via `sstore_static`:

```rust
// this will be zero before berlin fork.
if is_cold {
    gas += self.cold_storage_additional_cost();
}
```

### Test results with the aligned model

| Suite | Result |
|---|---|
| workspace `cargo nextest` (410 tests) | 409 pass; 1 fail: `revm-handler system_call::tests::test_system_call` (expects 22143, got 22043 — one cold set, −100) |
| revm ee-tests EIP-8037/EIP-2780 golden tests (Amsterdam) | **all pass, snapshots unchanged** |
| `test-fixtures/devnet/state_tests` — `fork_Amsterdam` | **all pass** |
| `test-fixtures/devnet/state_tests` — pre-Amsterdam forks | **5921 / 10962 fail** (state-root mismatch) |
| `test-fixtures/main/develop/state_tests` | **2039 / 2723 fail** (state-root mismatch) |

Failure breakdown by fork (all failures are `StateRootMismatch` caused by the −100
per cold SSTORE flowing into gas-used and sender balance):

```text
devnet:  1842 Prague, 1807 Osaka, 1740 Cancun, 148 Shanghai, 130 Paris,
         128 London, 126 Berlin, 0 Amsterdam
develop: 1875 Cancun, 112 Osaka, 48 Berlin, 2 Shanghai, 2 Prague
```

## Conclusion

The hypothesis "the tests have a bug in not being aligned" does not hold for
pre-Amsterdam forks: the additive cold charge is what EIP-2929 specifies ("charge
an *additional* COLD_SLOAD_COST"), it is implemented that way in execution-specs
for every fork Berlin..Osaka, and it has been live on mainnet since April 2021 —
the 2200/22100/5000 cold costs are consensus. The fixtures encode it correctly.

The glamsterdam tests are equally correct: EIP-8038 intentionally restructures
SSTORE pricing to the exclusive `access + write` form, which is the aligned model.

Therefore revm must keep a fork distinction in the SSTORE cold charge. The chosen
encoding is the `is_amsterdam` flag in `sstore_dynamic_gas`:

```rust
if is_cold {
    gas += if is_amsterdam {
        self.cold_storage_additional_cost()          // 2000: EIP-8038 folds warm into cold
    } else {
        self.cold_storage_cost()                     // 2100: EIP-2929 full cold on top of warm base
    };
}
```

where `cold_storage_cost()` is the derived helper
`cold_storage_additional_cost() + warm_storage_read_cost()` — which on every fork
equals the *total* cost of one cold storage access (2100); the forks differ only in
whether SSTORE re-charges the warm base on a cold access.

The `skip_cold_load` sentry can use `cold_storage_additional_cost` on all forks:
pre-Amsterdam the EIP-2200 stipend sentry (`remaining > 2300`) always covers the
full cold access (2200), so the threshold is only ever exercised under
Amsterdam-style pricing, where 2000 is the exact charge.
