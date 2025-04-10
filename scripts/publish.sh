#!/bin/bash

# stop on error
set -e

cargo publish --package revm-primitives
cargo publish --package revm-bytecode
cargo publish --package revm-state
cargo publish --package revm-database-interface
cargo publish --package revm-context-interface 
cargo publish --package revm-interpreter
cargo publish --package revm-precompile
cargo publish --package revm-database
cargo publish --package revm-context
cargo publish --package revm-handler
cargo publish --package revm-inspector
cargo publish --package revm
cargo publish --package revm-statetest-types
cargo publish --package revme
cargo publish --package op-revm
