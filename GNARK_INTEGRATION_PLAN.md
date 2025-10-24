# Gnark-crypto Integration Plan for REVM

## Recommended Architecture: Separate Optional Crate

### Directory Structure

```
revm/
├── crates/
│   ├── precompile/              # Existing crate (unchanged mostly)
│   │   ├── src/
│   │   │   └── bn254/
│   │   │       ├── arkworks.rs
│   │   │       ├── substrate.rs
│   │   │       └── gnark.rs     # Just FFI declarations
│   │   ├── Cargo.toml
│   │   └── build.rs (optional)
│   │
│   └── precompile-gnark/        # NEW: Go bridge crate (optional)
│       ├── gnark-ffi/           # Go code
│       │   ├── go.mod
│       │   ├── go.sum
│       │   ├── wrapper.go       # CGO wrapper
│       │   ├── wrapper_test.go  # Go tests
│       │   ├── Makefile         # Build automation
│       │   └── .gitignore
│       ├── src/
│       │   └── lib.rs           # Rust bindings
│       ├── build.rs             # Builds Go library
│       ├── Cargo.toml
│       └── README.md
```

### Implementation Steps

#### Step 1: Create the Go FFI Library

**File: `crates/precompile-gnark/gnark-ffi/go.mod`**
```go
module github.com/bluealloy/revm/gnark-ffi

go 1.21

require github.com/consensys/gnark-crypto v0.14.0
```

**File: `crates/precompile-gnark/gnark-ffi/wrapper.go`**
```go
package main

/*
#include <stdint.h>
*/
import "C"
import (
    "encoding/binary"
    "math/big"
    "unsafe"

    "github.com/consensys/gnark-crypto/ecc/bn254"
    "github.com/consensys/gnark-crypto/ecc/bn254/fp"
    "github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

// Error codes
const (
    OK                      = 0
    ERR_INVALID_G1_POINT    = -1
    ERR_INVALID_G2_POINT    = -2
    ERR_PAIRING_FAILED      = -3
    ERR_POINT_NOT_ON_CURVE  = -4
)

//export gnark_bn254_g1_add
func gnark_bn254_g1_add(p1_bytes *C.uint8_t, p2_bytes *C.uint8_t, out *C.uint8_t) C.int {
    p1_slice := unsafe.Slice(p1_bytes, 64)
    p2_slice := unsafe.Slice(p2_bytes, 64)

    var p1, p2 bn254.G1Affine
    if err := decodeG1Point(p1_slice, &p1); err != nil {
        return ERR_INVALID_G1_POINT
    }
    if err := decodeG1Point(p2_slice, &p2); err != nil {
        return ERR_INVALID_G1_POINT
    }

    var result bn254.G1Affine
    result.Add(&p1, &p2)

    encodeG1Point(&result, unsafe.Slice(out, 64))
    return OK
}

//export gnark_bn254_g1_mul
func gnark_bn254_g1_mul(point_bytes *C.uint8_t, scalar_bytes *C.uint8_t, out *C.uint8_t) C.int {
    point_slice := unsafe.Slice(point_bytes, 64)
    scalar_slice := unsafe.Slice(scalar_bytes, 32)

    var point bn254.G1Affine
    if err := decodeG1Point(point_slice, &point); err != nil {
        return ERR_INVALID_G1_POINT
    }

    // Parse scalar as big-endian
    scalar := new(big.Int).SetBytes(scalar_slice)

    var result bn254.G1Affine
    result.ScalarMultiplication(&point, scalar)

    encodeG1Point(&result, unsafe.Slice(out, 64))
    return OK
}

//export gnark_bn254_pairing_check
func gnark_bn254_pairing_check(pairs_data *C.uint8_t, num_pairs C.int, result *C.uint8_t) C.int {
    pairSize := 192 // 64 (G1) + 128 (G2)
    data := unsafe.Slice(pairs_data, int(num_pairs)*pairSize)

    g1Points := make([]bn254.G1Affine, num_pairs)
    g2Points := make([]bn254.G2Affine, num_pairs)

    for i := 0; i < int(num_pairs); i++ {
        offset := i * pairSize
        g1_bytes := data[offset : offset+64]
        g2_bytes := data[offset+64 : offset+192]

        if err := decodeG1Point(g1_bytes, &g1Points[i]); err != nil {
            return ERR_INVALID_G1_POINT
        }
        if err := decodeG2Point(g2_bytes, &g2Points[i]); err != nil {
            return ERR_INVALID_G2_POINT
        }
    }

    ok, err := bn254.PairingCheck(g1Points, g2Points)
    if err != nil {
        return ERR_PAIRING_FAILED
    }

    if ok {
        *result = 1
    } else {
        *result = 0
    }

    return OK
}

// Helper functions
func decodeG1Point(bytes []byte, point *bn254.G1Affine) error {
    // Big-endian encoding: x (32 bytes) | y (32 bytes)
    var x, y fp.Element
    x.SetBytes(bytes[0:32])
    y.SetBytes(bytes[32:64])

    // Handle point at infinity
    if x.IsZero() && y.IsZero() {
        point.X.SetZero()
        point.Y.SetZero()
        return nil
    }

    point.X = x
    point.Y = y

    if !point.IsOnCurve() {
        return fmt.Errorf("point not on curve")
    }
    if !point.IsInSubGroup() {
        return fmt.Errorf("point not in subgroup")
    }

    return nil
}

func decodeG2Point(bytes []byte, point *bn254.G2Affine) error {
    // G2 encoding: x_imag | x_real | y_imag | y_real (32 bytes each)
    var x0, x1, y0, y1 fp.Element
    x1.SetBytes(bytes[0:32])   // imaginary part
    x0.SetBytes(bytes[32:64])  // real part
    y1.SetBytes(bytes[64:96])
    y0.SetBytes(bytes[96:128])

    // Handle point at infinity
    if x0.IsZero() && x1.IsZero() && y0.IsZero() && y1.IsZero() {
        point.X.SetZero()
        point.Y.SetZero()
        return nil
    }

    point.X.A0 = x0
    point.X.A1 = x1
    point.Y.A0 = y0
    point.Y.A1 = y1

    if !point.IsOnCurve() {
        return fmt.Errorf("point not on curve")
    }
    if !point.IsInSubGroup() {
        return fmt.Errorf("point not in subgroup")
    }

    return nil
}

func encodeG1Point(point *bn254.G1Affine, out []byte) {
    xBytes := point.X.Bytes()
    yBytes := point.Y.Bytes()
    copy(out[0:32], xBytes[:])
    copy(out[32:64], yBytes[:])
}

func main() {}
```

**File: `crates/precompile-gnark/gnark-ffi/Makefile`**
```makefile
.PHONY: all clean test

LIB_NAME = libgnark_bn254
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Linux)
    EXT = .a
    SOEXT = .so
endif
ifeq ($(UNAME_S),Darwin)
    EXT = .a
    SOEXT = .dylib
endif
ifeq ($(OS),Windows_NT)
    EXT = .lib
    SOEXT = .dll
endif

all: $(LIB_NAME)$(EXT)

$(LIB_NAME)$(EXT): wrapper.go
	go mod download
	go build -buildmode=c-archive -o $(LIB_NAME)$(EXT) wrapper.go
	@echo "Built $(LIB_NAME)$(EXT)"

test:
	go test -v .

clean:
	rm -f $(LIB_NAME)$(EXT) $(LIB_NAME).h $(LIB_NAME)$(SOEXT)

.DEFAULT_GOAL := all
```

**File: `crates/precompile-gnark/gnark-ffi/.gitignore`**
```
*.a
*.so
*.dylib
*.dll
*.h
*.lib
```

#### Step 2: Create the Rust Wrapper Crate

**File: `crates/precompile-gnark/Cargo.toml`**
```toml
[package]
name = "revm-precompile-gnark"
version = "0.1.0"
description = "Gnark-crypto BN254 backend for REVM precompiles"
authors.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true

[dependencies]
# Just the error types
primitives = { workspace = true, default-features = false }

[build-dependencies]
# For build.rs if needed
which = "6.0"

[features]
default = ["std"]
std = ["primitives/std"]

[lib]
# This crate provides a C-compatible library
crate-type = ["lib", "staticlib"]
```

**File: `crates/precompile-gnark/build.rs`**
```rust
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let gnark_ffi_dir = manifest_dir.join("gnark-ffi");

    // Check if Go is available
    if which::which("go").is_err() {
        panic!("Go compiler not found. Please install Go 1.21 or later.");
    }

    // Build the Go library using Make
    let status = Command::new("make")
        .current_dir(&gnark_ffi_dir)
        .status()
        .expect("Failed to execute make");

    if !status.success() {
        panic!("Failed to build gnark FFI library");
    }

    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", gnark_ffi_dir.display());
    println!("cargo:rustc-link-lib=static=gnark_bn254");

    // Platform-specific linker flags
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=dl");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=ws2_32");
            println!("cargo:rustc-link-lib=dylib=userenv");
        }
        _ => {}
    }

    // Rerun if wrapper changes
    println!("cargo:rerun-if-changed=gnark-ffi/wrapper.go");
    println!("cargo:rerun-if-changed=gnark-ffi/go.mod");
}
```

**File: `crates/precompile-gnark/src/lib.rs`**
```rust
//! Gnark-crypto BN254 implementation for REVM precompiles
//!
//! This crate provides FFI bindings to the gnark-crypto Go library
//! for BN254 elliptic curve operations.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[link(name = "gnark_bn254", kind = "static")]
extern "C" {
    pub fn gnark_bn254_g1_add(
        p1: *const u8,
        p2: *const u8,
        out: *mut u8,
    ) -> i32;

    pub fn gnark_bn254_g1_mul(
        point: *const u8,
        scalar: *const u8,
        out: *mut u8,
    ) -> i32;

    pub fn gnark_bn254_pairing_check(
        pairs_data: *const u8,
        num_pairs: i32,
        result: *mut u8,
    ) -> i32;
}

// Re-export for use in precompile crate
pub use primitives;
```

**File: `crates/precompile-gnark/README.md`**
```markdown
# revm-precompile-gnark

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
```

#### Step 3: Update Main Precompile Crate

**Update: `crates/precompile/Cargo.toml`**
```toml
# Add to [dependencies]
revm-precompile-gnark = { path = "../precompile-gnark", optional = true }

# Add to [features]
gnark = ["dep:revm-precompile-gnark", "std"]  # Note: requires std
```

**Update: `crates/precompile/src/bn254/gnark.rs`**
```rust
//! BN254 precompile using gnark-crypto Go library via FFI

use super::{G1_LEN, G2_LEN, SCALAR_LEN};
use crate::PrecompileError;
use std::vec::Vec;

/// Performs point addition on two G1 points using gnark
#[inline]
pub(crate) fn g1_point_add(
    p1_bytes: &[u8],
    p2_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    assert_eq!(p1_bytes.len(), G1_LEN);
    assert_eq!(p2_bytes.len(), G1_LEN);

    let mut output = [0u8; 64];

    let result = unsafe {
        revm_precompile_gnark::gnark_bn254_g1_add(
            p1_bytes.as_ptr(),
            p2_bytes.as_ptr(),
            output.as_mut_ptr(),
        )
    };

    match result {
        0 => Ok(output),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        _ => Err(PrecompileError::other("gnark g1 add failed")),
    }
}

/// Performs a G1 scalar multiplication using gnark
#[inline]
pub(crate) fn g1_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    assert_eq!(point_bytes.len(), G1_LEN);
    assert_eq!(scalar_bytes.len(), SCALAR_LEN);

    let mut output = [0u8; 64];

    let result = unsafe {
        revm_precompile_gnark::gnark_bn254_g1_mul(
            point_bytes.as_ptr(),
            scalar_bytes.as_ptr(),
            output.as_mut_ptr(),
        )
    };

    match result {
        0 => Ok(output),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        _ => Err(PrecompileError::other("gnark g1 mul failed")),
    }
}

/// Pairing check using gnark
#[inline]
pub(crate) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    if pairs.is_empty() {
        return Ok(true);
    }

    // Flatten pairs into a single buffer
    let mut pairs_data = Vec::with_capacity(pairs.len() * (G1_LEN + G2_LEN));

    for (g1_bytes, g2_bytes) in pairs {
        assert_eq!(g1_bytes.len(), G1_LEN);
        assert_eq!(g2_bytes.len(), G2_LEN);
        pairs_data.extend_from_slice(g1_bytes);
        pairs_data.extend_from_slice(g2_bytes);
    }

    let mut result: u8 = 0;

    let ret = unsafe {
        revm_precompile_gnark::gnark_bn254_pairing_check(
            pairs_data.as_ptr(),
            pairs.len() as i32,
            &mut result as *mut u8,
        )
    };

    match ret {
        0 => Ok(result == 1),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        -2 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        _ => Err(PrecompileError::other("gnark pairing check failed")),
    }
}
```

**Update: `crates/precompile/src/bn254.rs`**
```rust
cfg_if::cfg_if! {
    if #[cfg(feature = "gnark")] {
        pub(crate) mod gnark;
        pub(crate) use gnark as crypto_backend;
    } else if #[cfg(feature = "bn")] {
        pub(crate) mod substrate;
        pub(crate) use substrate as crypto_backend;
    } else {
        pub(crate) use arkworks as crypto_backend;
    }
}
```

#### Step 4: Add to Workspace

**Update: Root `Cargo.toml`**
```toml
[workspace]
members = [
    # ... existing members ...
    "crates/precompile-gnark",  # ADD THIS
]

[workspace.dependencies]
# ... existing dependencies ...
revm-precompile-gnark = { path = "crates/precompile-gnark", version = "0.1.0", default-features = false }
```

### Usage

```bash
# Build with gnark backend
cargo build --features gnark -p revm-precompile

# Test with gnark backend
cargo test --features gnark -p revm-precompile bn254

# Use in downstream projects
[dependencies]
revm = { version = "*", features = ["gnark"] }
```

### CI/CD Considerations

In your GitHub Actions workflow:

```yaml
# .github/workflows/ci.yml
jobs:
  test-gnark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: '1.21'
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --features gnark -p revm-precompile-gnark
      - run: cargo test --features gnark -p revm-precompile
```

## Advantages of This Approach

1. **Clean Separation**: Go code is isolated in its own crate
2. **Optional Build**: Only builds when `gnark` feature is enabled
3. **CI Flexibility**: Easy to skip in CI jobs that don't need it
4. **Cross-platform**: Makefile handles platform differences
5. **Testable**: Go code can be tested independently
6. **Version Control**: `.gitignore` keeps build artifacts out
7. **Documentation**: Self-contained with its own README
8. **REVM Compatible**: Follows existing patterns (like `blst` feature)

## Alternative: Pre-built Binaries

For production use, consider distributing pre-built binaries:

```
crates/precompile-gnark/
├── libs/
│   ├── linux-x86_64/libgnark_bn254.a
│   ├── macos-aarch64/libgnark_bn254.a
│   └── windows-x86_64/gnark_bn254.lib
└── build.rs (checks for pre-built lib first)
```

This avoids requiring Go at build time for end users.
