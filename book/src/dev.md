# Building from source

It requires running
```bash
git clone https://github.com/bluealloy/revm.git
cd revm
cargo build --release
```

**_Note:_** This project tends to use the newest rust version, so if you're encountering a build error try running rustup update first.

**_Note:_** `clang` is required for building revm with `c-kzg` or `secp256k1` feature flags as they depend on `C` libraries. If you don't have it installed, you can install it with `apt install clang`.