#!/usr/bin/env bash
# Generate the benchmark from ethereum/execution-specs#3631 using EELS.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REVISION="bd87e232119fc298475260c4d27a77506c1fda01"
OUTPUT="$REPO_ROOT/bins/revme/fixtures/jumpdest-analysis"

if ! command -v uv >/dev/null 2>&1; then
    echo "uv is required to install the upstream Python environment." >&2
    exit 1
fi

mkdir -p "$REPO_ROOT/target"
SOURCE_DIR="$(mktemp -d "$REPO_ROOT/target/jumpdest-analysis.XXXXXX")"
curl --fail --location --retry 3 \
    "https://api.github.com/repos/ethereum/execution-specs/tarball/$REVISION" \
    --output "$SOURCE_DIR/source.tar.gz"
tar -xzf "$SOURCE_DIR/source.tar.gz" --strip-components=1 -C "$SOURCE_DIR"
cd "$SOURCE_DIR"

uv sync --locked --python 3.12 --no-default-groups --group test

# Benchmark fixtures normally omit the full post-state. REVME's blockchain
# runner checks account expectations from postState, so retain it here.
.venv/bin/python - <<'PY'
from pathlib import Path

source = Path("tests/benchmark/compute/scenario/test_mix_operations.py")
text = source.read_text()
call = "    benchmark_test(\n"
assert text.count(call) == 1
source.write_text(text.replace(call, call + "        include_full_post_state_in_output=True,\n"))
PY

PYTHONUNBUFFERED=1 .venv/bin/fill \
    tests/benchmark/compute/scenario/test_mix_operations.py \
    --fork Amsterdam --gas-benchmark-values 60 \
    -k 'not blockchain_test_engine' -n "${JOBS:-4}" \
    --output filled --no-html

# Keep only runnable fixtures, with one file per pattern for easy comparison.
.venv/bin/python - "$OUTPUT" "$REVISION" <<'PY'
import json
import re
import sys
from pathlib import Path

output = Path(sys.argv[1])
revision = sys.argv[2]
fixtures = {}
for path in Path("filled/blockchain_tests").rglob("*.json"):
    fixtures.update(json.loads(path.read_text()))
assert len(fixtures) == 13, f"Expected 13 patterns, got {len(fixtures)}"
output.mkdir(parents=True, exist_ok=True)
for name, fixture in fixtures.items():
    pattern = re.search(r"-blockchain_test-(.+)-benchmark-gas-value_60M", name)
    assert pattern is not None, name
    assert fixture["network"] == "Amsterdam"
    assert fixture["postState"], "Full post-state is required"
    assert len(fixture["blocks"]) == 1
    block = fixture["blocks"][0]
    assert len(block["transactions"]) == 4
    assert int(block["blockHeader"]["gasUsed"], 16) == 60_000_000
    fixture["_info"]["url"] = (
        f"https://github.com/ethereum/execution-specs/blob/{revision}/"
        "tests/benchmark/compute/scenario/test_mix_operations.py"
    )
    fixture["_info"]["revmAdaptation"] = "Enable include_full_post_state_in_output only."
    destination = output / f"{pattern[1]}.json"
    destination.write_text(json.dumps({name: fixture}, indent=2) + "\n")
    print(destination)
PY

echo "Run: cargo run --release -p revme -- btest $OUTPUT"
