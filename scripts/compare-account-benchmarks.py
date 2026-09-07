#!/usr/bin/env python3
"""Compare prebuilt Criterion binaries serially on one CPU, retaining raw samples.

Build each checkout with identical Cargo.lock, benchmark sources, toolchain, and profile:
  cargo bench -p revme --profile profiling --bench evm --bench account_extension \
      --no-run --message-format=json > BUILD.jsonl
Then pass --build main=BUILD.jsonl --build base=BUILD.jsonl --build head=BUILD.jsonl.
This script does not build or change the checkouts. No third-party Python packages required.
"""
import argparse
import csv
import json
import os
from pathlib import Path
import random
import statistics
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cpu", type=int, default=2)
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--filter", default="")
    parser.add_argument("--warmup", type=float, default=0.1)
    parser.add_argument("--measurement", type=float, default=0.2)
    parser.add_argument("--samples", type=int, default=20)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / "config.json").write_text(json.dumps(vars(args), default=str, indent=2))
    builds = {}
    for spec in args.build:
        label, path = spec.split("=", 1)
        binaries = {}
        for line in Path(path).read_text().splitlines():
            record = json.loads(line)
            if record.get("reason") == "compiler-artifact" and record.get("executable"):
                if "bench" in record["target"]["kind"]:
                    binaries[record["target"]["name"]] = record["executable"]
        assert binaries, f"No benchmark executables in {path}"
        builds[label] = binaries
    rows = []
    labels = list(builds)
    for repeat in range(args.rounds):
        # Rotate the starting revision; reverse every other round to reduce order bias.
        order = labels[repeat % len(labels):] + labels[:repeat % len(labels)]
        if repeat % 2:
            order.reverse()
        for label in order:
            for suite, binary in builds[label].items():
                folder = args.output / f"round-{repeat}" / label / suite
                folder.mkdir(parents=True)
                command = ["taskset", "-c", str(args.cpu), binary, "--bench", "--noplot",
                           "--warm-up-time", str(args.warmup), "--measurement-time",
                           str(args.measurement), "--sample-size", str(args.samples)]
                if args.filter:
                    command.append(args.filter)
                print(f"round={repeat} revision={label} suite={suite}", flush=True)
                with (folder / "run.log").open("w") as log:
                    subprocess.run(command, env={**os.environ, "CRITERION_HOME": str(folder.resolve())},
                                   stdout=log, stderr=subprocess.STDOUT, check=True)
                for path in folder.glob("**/new/estimates.json"):
                    estimates = json.loads(path.read_text())
                    meta = json.loads(path.with_name("benchmark.json").read_text())
                    rows.append({"round": repeat, "revision": label, "suite": suite,
                                 "benchmark": meta["full_id"],
                                 "mean_ns": estimates["mean"]["point_estimate"]})
                with (args.output / "measurements.csv").open("w") as output:
                    writer = csv.DictWriter(output, fieldnames=["round", "revision", "suite", "benchmark", "mean_ns"])
                    writer.writeheader()
                    writer.writerows(rows)
    summaries = []
    rng = random.Random(17)
    for suite, name in sorted({(r["suite"], r["benchmark"]) for r in rows}):
        for baseline in labels:
            if baseline == "head":
                continue
            pairs = []
            for repeat in range(args.rounds):
                values = {r["revision"]: r["mean_ns"] for r in rows
                          if r["round"] == repeat and r["suite"] == suite and r["benchmark"] == name}
                if baseline in values and "head" in values:
                    pairs.append(values["head"] / values[baseline])
            if not pairs:
                continue
            bootstrap = sorted(statistics.median(rng.choices(pairs, k=len(pairs))) for _ in range(2000))
            summaries.append({"suite": suite, "benchmark": name, "baseline": baseline,
                              "pairs": len(pairs), "median_change_pct": (statistics.median(pairs)-1)*100,
                              "min_change_pct": (min(pairs)-1)*100, "max_change_pct": (max(pairs)-1)*100,
                              "bootstrap_low_pct": (bootstrap[50]-1)*100,
                              "bootstrap_high_pct": (bootstrap[1949]-1)*100})
    with (args.output / "comparison.csv").open("w") as output:
        if summaries:
            writer = csv.DictWriter(output, fieldnames=list(summaries[0]))
            writer.writeheader()
            writer.writerows(summaries)


if __name__ == "__main__":
    main()
