#!/usr/bin/env python3
"""Run fixed-work account_extension_perf examples with Linux hardware counters.

Pass --binary main=PATH --binary base=PATH --binary head=PATH.
Each example must be built with the same toolchain, lockfile, and profiling profile.
Raw counters and timed-loop durations are retained; perf counters include process setup.
"""
import argparse
import csv
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--rounds", type=int, default=5)
    parser.add_argument("--cpu", type=int, default=2)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    binaries = dict(spec.split("=", 1) for spec in args.binary)
    labels = list(binaries)
    with (args.output / "measurements.csv").open("w") as output:
        writer = csv.writer(output)
        writer.writerow(["round", "revision", "case", "iterations", "ns_per_iteration"])
        for repeat in range(args.rounds):
            order = labels[repeat % len(labels):] + labels[:repeat % len(labels)]
            if repeat % 2:
                order.reverse()
            for case, iterations in [("extcodehash", 1_000_000), ("commit", 5000), ("batch", 5000)]:
                for label in order:
                    prefix = args.output / f"{repeat}-{label}-{case}"
                    print(prefix, flush=True)
                    result = subprocess.run(
                        ["perf", "stat", "-x,", "-e", "cycles,instructions,branches,branch-misses",
                         "-o", str(prefix.with_suffix(".perf.csv")), "--", "taskset", "-c",
                         str(args.cpu), binaries[label], case, str(iterations)],
                        env={**os.environ, "LC_ALL": "C"}, text=True, capture_output=True, check=True,
                    )
                    prefix.with_suffix(".log").write_text(result.stdout + result.stderr)
                    _, count, elapsed = result.stdout.strip().split(",")
                    writer.writerow([repeat, label, case, count, int(elapsed) / int(count)])
                    output.flush()


if __name__ == "__main__":
    main()
