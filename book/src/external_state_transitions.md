# External State Transitions (EIP-4788 & EIP-2935)

Some Ethereum Improvement Proposals (EIPs) require state transitions that are not triggered by regular user transactions, but are instead performed by the client using special system calls (such as `system_call`). These transitions are part of the EVM state changes, but are initiated by the client at specific block boundaries (pre- or post-block hooks), as required by the EIP.

- [EIP-4788: Beacon block root in the EVM](https://eips.ethereum.org/EIPS/eip-4788)
- [EIP-2935: Add `blockHash` and `blockNumber` to the EVM](https://eips.ethereum.org/EIPS/eip-2935)

## What are external state transitions?

External state transitions refer to updates to the Ethereum state that are not performed by regular user transactions, but are instead performed by the client using system calls at block boundaries. These are typically required for EIPs that introduce new system contracts or require special state updates at block boundaries.

## EIP-4788: Beacon block root in the EVM

EIP-4788 requires that the root of each beacon chain block is committed to the execution layer and made available in the EVM via a special contract. This is achieved by the client calling a system contract at a fixed address (`0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02`) with the beacon root as input, at the start of each block. The contract maintains a ring buffer of recent roots.

- The system call is performed by the client, not by EVM transaction execution.
- If the contract does not exist, the call fails silently.
- See [EIP-4788](https://eips.ethereum.org/EIPS/eip-4788) for full details.
- Example implementation in Reth: [reth#4457](https://github.com/paradigmxyz/reth/pull/4457)

## EIP-2935: Add blockHash and blockNumber to the EVM

EIP-2935 introduces a system contract that stores recent block hashes, allowing contracts to query them. The client is responsible for updating this contract at each block, by calling a system contract at a fixed address (`0x0000F90827F1C53a10cb7A02335B175320002935`) with the new block hash.

- The system call is performed by the client, not by EVM transaction execution.
- See [EIP-2935](https://eips.ethereum.org/EIPS/eip-2935) for full details.
- Example implementation in Reth: [reth#7818](https://github.com/paradigmxyz/reth/pull/7818)

## How does this affect REVM users?

- To perform these block state transitions, the client or test harness should use the system call mechanism (`system_call`) provided by REVM.
- REVM itself does not automatically perform these transitions; it expects the client to initiate them at the appropriate block boundaries, as specified by the EIPs.
- If you are building a full Ethereum client or a test harness, you are responsible for performing these system calls at the appropriate block boundaries, as specified in the EIPs.
- If you are only using REVM for transaction execution, you may need to ensure that the state of these system contracts is kept up to date externally.

## References
- [EIP-4788: Beacon block root in the EVM](https://eips.ethereum.org/EIPS/eip-4788)
- [EIP-2935: Add blockHash and blockNumber to the EVM](https://eips.ethereum.org/EIPS/eip-2935)
- [reth#4457: EIP-4788 implementation](https://github.com/paradigmxyz/reth/pull/4457)
- [reth#7818: EIP-2935 implementation](https://github.com/paradigmxyz/reth/pull/7818) 
