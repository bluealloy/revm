#!/usr/bin/env bash
set -eo pipefail

# Usage: ./asm_both.sh "OPCODE"
cargo asm -F asm -p revm-interpreter "cx::$1::" > a.s
cargo asm -F asm -p revm-interpreter "cx::$1::" --llvm > a.ll
cargo asm -F asm -p revm-interpreter "tail::$1::" > b.s
cargo asm -F asm -p revm-interpreter "tail::$1::" --llvm > b.ll
