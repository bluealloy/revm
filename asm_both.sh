#!/usr/bin/env bash
set -eo pipefail

# Usage: ./asm_both.sh "name" "opcode"
cargo asm -p revm-interpreter "$1::" > a.s
cargo asm -p revm-interpreter "$1::" --llvm > a.ll
cargo asm -p revm-interpreter "tail_call_instr::<$2," > b.s
cargo asm -p revm-interpreter "tail_call_instr::<$2," --llvm > b.ll
