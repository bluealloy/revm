#!/bin/bash

# stop on error
set -e

cargo publish --package revm-primitives
cargo publish --package revm-precompile
cargo publish --package revm-interpreter
cargo publish --package revm
cargo publish --package revme

echo "All crates published"
