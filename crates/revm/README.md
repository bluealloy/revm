# REVM benchmarks

## Standard benchmarks

This executes the standard benchmarks for the REVM crate.

Execute the following command to run the standard benchmarks:

`cargo bench --bench bench`

## Bytecode execution benchmark
This is a command line tool to benchmark the execution of any given bytecode.

Execute the following command to run the bytecode execution benchmark:

`echo "60FF600052610FFF600020" | cargo bench --bench=bytecode_benchmark`

## Debug
For debug information set environmental variable CRITERION_DEBUG=1.