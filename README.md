# Revm

[![CI](https://github.com/bluealloy/revm/actions/workflows/ci.yml/badge.svg)][gh-ci]
[![License](https://img.shields.io/badge/License-MIT-orange.svg)][mit-license]
[![Chat][tg-badge]][tg-url]
[![Crates.io](https://img.shields.io/crates/v/revm.svg)](https://crates.io/crates/revm)
[![Documentation](https://docs.rs/revm/badge.svg)](https://docs.rs/revm)

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [Users](#users)
- [Development](#development)
- [Community](#community)
- [Security](#security)
- [License](#license)

## Overview

Revm is a highly efficient and stable implementation of the Ethereum Virtual Machine (EVM) written in Rust.

![banner](https://raw.githubusercontent.com/bluealloy/revm/refs/heads/main/assets/logo/revm-banner.png)

Known for its robustness, it stands as one of the most popular libraries and critical components of the Ethereum ecosystem. Revm plays a crucial role across various projects, being widely utilized by almost all tooling and block builders. It is integrated into Reth, multiple Layer 2 variants and other clients, serving as a standard for zkVMs.

## Features

- High-performance EVM implementation
- Mainnet transaction execution
- Powerful inspection and tracing capabilities
- Framework for EVM variants support
- Extensive testing and benchmarking tools

## Installation

Add revm to your `Cargo.toml`:

```toml
[dependencies]
revm = "latest_version"
```

## Usage

### Basic Example

```rust
use revm::{Context, EVM};

// Create an EVM instance with mainnet configuration
let mut evm = Context::mainnet()
    .with_block(block)
    .build_mainnet();

// Execute a transaction
let out = evm.transact(tx);

// Use the inspector for transaction tracing
let mut evm = evm.with_inspector(tracer);
let out = evm.inspect_with_tx(tx);
```

For more complex examples, including foundry cheatcodes and custom inspectors, please refer to the [documentation](https://docs.rs/revm).

## Contributing

We welcome contributions! Here's how you can help:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Before contributing, please:
- Read our contribution guidelines
- Check the issue tracker
- Join our [Telegram group][tg-url] for discussions

Revm offers two primary applications: firstly, it functions as an executor where users can set up block info and process mainnet transactions; secondly, it acts as a framework that facilitates the extension and support of different EVM variants such as revm-optimism.

### How to use:

Here is a straightforward example of using the Execution API: It allows us to create an Ethereum Virtual Machine (EVM) and execute transactions. Additionally, it can be utilized to generate traces with the inspector or more complex example of foundry cheatcodes.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);

// or you can use powerful inspection tool to trace it
let mut evm = evm.with_inspector(tracer);
let out = evm.inspect_with_tx(tx);
```

The Evm Framework API is somewhat complex to use, but this document provides a detailed explanation. It enables users to extend logic, incorporate various context types, and offers built-in support for inspection. For a practical example, you can refer to the revm-optimism crate.

### Users:

As previously noted, there are several groups of projects that utilize this technology:

* **Major block builders**.
* **Clients**: [Reth](https://github.com/paradigmxyz/reth), [Helios](https://github.com/a16z/helios), [Trin](https://github.com/ethereum/trin),..
* **Tooling**: [Foundry](https://github.com/foundry-rs/foundry/), [Hardhat](https://github.com/NomicFoundation/hardhat),..
* **L2s**: [Optimism](https://github.com/bluealloy/revm/tree/main/crates/optimism), [Coinbase](https://www.base.org/), [Scroll](https://github.com/scroll-tech/revm),..
* **zkVM**: [Risc0](https://github.com/risc0/risc0-ethereum), [Succinct](https://github.com/succinctlabs/sp1-reth),..

The full list of projects that use Revm is available in the awesome-revm section of the book.

### How to, dev section

Note that book and code docs are still in WIP stage and they are being updated!

Part of the links point to the code documentation or the book. code docs are there to explain usage of particular part of the code where book is to get more of the overview on architecture or how components/projects fit toggether.

* How to build and use revm can be found here. (code)
* Architecture overview can be seen here. (book)
* Structure of the project (list of crates) can be seen here. (book)
* How to use Revm Framework can be found here. (book)
* Release procedure and changelogs explanation. (book)
* How to use revme (Revm binary with few commands) can be found here. (code)
* How to run Ethereum test can be found here: (book)
* How to run examples and benchmark with `samply` to check performance. (book)
* If there is more explanations please open PR request for it.

## Development

> 📘 **Note**: Documentation and code docs are actively being updated!

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/bluealloy/revm.git
   cd revm
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

### Documentation

- [Architecture Overview](./docs/architecture.md)
- [Project Structure](./docs/structure.md)
- [Framework Guide](./docs/framework.md)
- [Release Procedure](./docs/releases.md)
- [Examples & Benchmarks](./docs/examples.md)

### Performance Testing

To run benchmarks with `samply`:

```bash
cargo install samply
cargo bench
```

## Users

Revm is trusted by major players in the Ethereum ecosystem:

* **Block Builders**: Leading block builders in the Ethereum network
* **Clients**: 
  - [Reth](https://github.com/paradigmxyz/reth)
  - [Helios](https://github.com/a16z/helios)
  - [Trin](https://github.com/ethereum/trin)
* **Development Tools**: 
  - [Foundry](https://github.com/foundry-rs/foundry/)
  - [Hardhat](https://github.com/NomicFoundation/hardhat)
* **Layer 2 Solutions**: 
  - [Optimism](https://github.com/bluealloy/revm/tree/main/crates/optimism)
  - [Base (Coinbase)](https://www.base.org/)
  - [Scroll](https://github.com/scroll-tech/revm)
* **zkVM Implementations**: 
  - [Risc0](https://github.com/risc0/risc0-ethereum)
  - [Succinct](https://github.com/succinctlabs/sp1-reth)

For a complete list of projects using Revm, see our [awesome-revm](./docs/awesome-revm.md) section.

## Community

- 💬 Join our [Telegram group](https://t.me/+Ig4WDWOzikA3MzA0) for discussions
- 🐛 Report issues on [GitHub Issues](https://github.com/bluealloy/revm/issues)
- 📝 Follow our [Blog](https://blog.revm.dev) for updates

## Security

For security concerns, please contact:
- Email: dragan0rakita@gmail.com
- Keybase: [draganrakita](https://keybase.io/draganrakita/)

Please report security issues responsibly through these channels rather than creating public issues.

## License

Revm is licensed under [MIT License][mit-license].

Contributions are welcome and will be licensed under the same terms.

[mit-license]: https://opensource.org/license/mit/
[gh-ci]: https://github.com/bluealloy/revm/actions/workflows/ci.yml
[tg-url]: https://t.me/+Ig4WDWOzikA3MzA0
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue
