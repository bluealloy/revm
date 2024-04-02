#!/bin/bash

cargo publish --package revm-primitives
cargo publish --package revm-precompiles
cargo publish --package revm-interpreter
cargo publish --package revm
cargo publish --package revme

echo "All crates published"
