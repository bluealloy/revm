#!/usr/bin/env bash
set -eo pipefail

zargo=(cargo)
# Usage: ./asm_both.sh "OPCODE"
"${zargo[@]}" asm -F asm -p revm-interpreter "cx::$1::" > a.s
"${zargo[@]}" asm -F asm -p revm-interpreter "cx::$1::" --llvm > a.ll
"${zargo[@]}" asm -F asm -p revm-interpreter "tail::$1::" > b.s
"${zargo[@]}" asm -F asm -p revm-interpreter "tail::$1::" --llvm > b.ll
