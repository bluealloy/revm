# revm-gnark-bn254

Gnark-crypto BN254 backend for REVM precompiles.

## Requirements

- Go 1.21 or later
- Make
- C compiler (for CGO)

## Building

This crate builds a Go library using CGO and links it statically.

```bash
cargo build
```

## Usage

This crate is designed to be used by `revm-precompile` with the `gnark` feature:

```toml
[dependencies]
revm-precompile = { version = "*", features = ["gnark"] }
```

## Cross-compilation

Cross-compiling requires setting up the Go cross-compilation toolchain for your target platform.

## License

MIT


