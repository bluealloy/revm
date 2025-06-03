//! EIP-7825: Transaction Gas Limit Cap  
//! Introduce a protocol-level cap on the maximum gas used by a transaction to 30 million.

/// Transaction gas limit cap.
///
/// # Rationale from EIP
///
/// The proposed cap of 30 million gas is based on the typical size of Ethereum blocks today,
/// which often range between 30-40 million gas. This value is large enough to allow complex
/// transactions, such as contract deployments and advanced DeFi interactions, while still
/// reserving space for other transactions within a block.
pub const TX_GAS_LIMIT_CAP: u64 = 30_000_000;
