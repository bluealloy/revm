### Revm

[![CI](https://github.com/bluealloy/revm/actions/workflows/ci.yml/badge.svg)][gh-ci]
[![License](https://img.shields.io/badge/License-MIT-orange.svg)][mit-license]
![revm](https://img.shields.io/crates/v/revm.svg)
[![Chat][tg-badge]][tg-url]

Revm is a highly efficient and stable implementation of the Ethereum Virtual Machine (EVM) written in Rust.

![banner](https://raw.githubusercontent.com/bluealloy/revm/refs/heads/main/assets/logo/revm-banner.png)

[mit-license]: https://opensource.org/license/mit/
[gh-ci]: https://github.com/bluealloy/revm/actions/workflows/ci.yml
[tg-url]: https://t.me/+Ig4WDWOzikA3MzA0
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue

Known for its robustness, it stands as one of the most popular libraries and a critical component of the Ethereum ecosystem. Revm plays a crucial role across various projects, being widely utilized by almost all tooling and block builders. It is integrated into Reth, multiple Layer 2 variants and other clients and serving as a standard for zkVMs.

Revm offers two primary applications: firstly, it functions as an executor where users can set up block info and process mainnet transactions; secondly, it acts as a framework that facilitates the extension and support of different EVM variants such as op-revm.

### How to use:

Here is a straightforward example of using the Execution API: It allows us to create an Ethereum Virtual Machine (EVM) and execute transactions. Additionally, it can be utilized to generate traces with the inspector or more complex example of foundry cheatcodes.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);

// or you can use powerful inspection tool to trace it
let mut evm = evm.with_inspector(tracer);
let out = evm.inspect_with_tx(tx);
```

The Evm Framework API is somewhat complex to use, but this document provides a detailed explanation. It enables users to extend logic, incorporate various context types, and offers built-in support for inspection. For a practical example, you can refer to the [op-revm crate](https://github.com/op-rs/op-revm).

### Users:

As previously noted, there are several groups of projects that utilize this technology:

* **Major block builders**.
* **Clients**: [Reth](https://github.com/paradigmxyz/reth), [Helios](https://github.com/a16z/helios), [Trin](https://github.com/ethereum/trin),..
* **Tooling**: [Foundry](https://github.com/foundry-rs/foundry/), [Hardhat](https://github.com/NomicFoundation/hardhat),..
* **L2s**: [Optimism](https://github.com/bluealloy/revm/tree/main/crates/op-revm), [Coinbase](https://www.base.org/), [Scroll](https://github.com/scroll-tech/revm),..
* **zkVM**: [Risc0](https://github.com/risc0/risc0-ethereum), [Succinct](https://github.com/succinctlabs/rsp),..

The full list of projects that use Revm is available in the [awesome-revm](https://bluealloy.github.io/revm/awesome.html) section of the book.

### How to, dev section

The [book](https://bluealloy.github.io/revm/) and [`Architecture and API`](https://bluealloy.github.io/revm/architecture.html) page is the best starting resource.

Some quick links can be found here. Some point to code documentation or the book. code docs are there to explain usage of a particular part of the code where the book is to get more of an overview of the architecture or how components/projects fit together.

* How to build and use revm can be found here. [book](https://bluealloy.github.io/revm/dev.html)
* Architecture overview can be seen here. [book](https://bluealloy.github.io/revm/architecture.html)
* Structure of the project (list of crates and their versions) can be seen here. [book](https://github.com/bluealloy/revm/tree/main/crates)
* How to use Revm Framework can be found in MyEvm example. [book](https://github.com/bluealloy/revm/tree/main/examples/my_evm)
* Release procedure and changelogs explanation. [book](https://bluealloy.github.io/revm/release_procedure.html)
* How to use revme (Revm binary with few commands) can be found here. [code](https://github.com/bluealloy/revm/tree/main/bins/revme)
* How to run Ethereum test can be found here: [book](https://bluealloy.github.io/revm/revme.html#running-eth-tests)
* If there is more need for explanations please open a PR request.

### Community:
For questions please open a github issue or join the public telegram group: [https://t.me/+Ig4WDWOzikA3MzA0](https://t.me/+Ig4WDWOzikA3MzA0)

### Licence
Revm is licensed under MIT Licence.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in these crates by you, shall be licensed as above, without any additional terms or conditions.

### Security

For any security questions or findings, please reach out to me directly via email at dragan0rakita@gmail.com or contact me on Keybase under the username @draganrakita.
