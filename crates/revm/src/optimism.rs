//! Optimism-specific constants, types, and helpers.

use bytes::Bytes;
use core::ops::Mul;
use once_cell::sync::Lazy;
use revm_interpreter::primitives::{db::Database, hex_literal::hex, Address, Spec, SpecId, U256};

const ZERO_BYTE_COST: u64 = 4;
const NON_ZERO_BYTE_COST: u64 = 16;

static L1_BASE_FEE_SLOT: Lazy<U256> = Lazy::new(|| U256::from(1));
static L1_OVERHEAD_SLOT: Lazy<U256> = Lazy::new(|| U256::from(5));
static L1_SCALAR_SLOT: Lazy<U256> = Lazy::new(|| U256::from(6));

/// The address of L1 fee recipient.
pub static L1_FEE_RECIPIENT: Lazy<Address> =
    Lazy::new(|| Address::from_slice(&hex!("420000000000000000000000000000000000001A")));

/// The address of the base fee recipient.
pub static BASE_FEE_RECIPIENT: Lazy<Address> =
    Lazy::new(|| Address::from_slice(&hex!("4200000000000000000000000000000000000019")));

/// The address of the L1Block contract.
pub static L1_BLOCK_CONTRACT: Lazy<Address> =
    Lazy::new(|| Address::from_slice(&hex!("4200000000000000000000000000000000000015")));

/// L1 block info
///
/// We can extract L1 epoch data from each L2 block, by looking at the `setL1BlockValues`
/// transaction data. This data is then used to calculate the L1 cost of a transaction.
///
/// Here is the format of the `setL1BlockValues` transaction data:
///
/// setL1BlockValues(uint64 _number, uint64 _timestamp, uint256 _basefee, bytes32 _hash,
/// uint64 _sequenceNumber, bytes32 _batcherHash, uint256 _l1FeeOverhead, uint256 _l1FeeScalar)
///
/// For now, we only care about the fields necessary for L1 cost calculation.
#[derive(Clone, Debug)]
pub struct L1BlockInfo {
    /// The base fee of the L1 origin block.
    pub l1_base_fee: U256,
    /// The current L1 fee overhead.
    pub l1_fee_overhead: U256,
    /// The current L1 fee scalar.
    pub l1_fee_scalar: U256,
}

impl L1BlockInfo {
    /// Calculate the data gas for posting the transaction on L1. Calldata costs 16 gas per non-zero
    /// byte and 4 gas per zero byte.
    ///
    /// Prior to regolith, an extra 68 non-zero bytes were included in the rollup data costs to
    /// account for the empty signature.
    pub fn data_gas<SPEC: Spec>(&self, input: &Bytes) -> U256 {
        let mut rollup_data_gas_cost = U256::from(input.iter().fold(0, |acc, byte| {
            acc + if *byte == 0x00 {
                ZERO_BYTE_COST
            } else {
                NON_ZERO_BYTE_COST
            }
        }));

        // Prior to regolith, an extra 68 non zero bytes were included in the rollup data costs.
        if !SPEC::enabled(SpecId::REGOLITH) {
            rollup_data_gas_cost += U256::from(NON_ZERO_BYTE_COST).mul(U256::from(68));
        }

        rollup_data_gas_cost
    }

    /// Calculate the gas cost of a transaction based on L1 block data posted on L2
    pub fn calculate_tx_l1_cost<SPEC: Spec>(&self, input: &Bytes, is_deposit: bool) -> U256 {
        let rollup_data_gas_cost = self.data_gas::<SPEC>(input);

        if is_deposit || rollup_data_gas_cost == U256::ZERO {
            return U256::ZERO;
        }

        rollup_data_gas_cost
            .saturating_add(self.l1_fee_overhead)
            .saturating_mul(self.l1_base_fee)
            .saturating_mul(self.l1_fee_scalar)
            .checked_div(U256::from(1_000_000))
            .unwrap_or_default()
    }
}

/// Fetches the L1 block info from the `L1Block` contract in the database.
pub fn fetch_l1_block_info<DB: Database>(db: &mut DB) -> Result<L1BlockInfo, DB::Error> {
    let l1_base_fee = db.storage(*L1_BLOCK_CONTRACT, *L1_BASE_FEE_SLOT)?;
    let l1_fee_overhead = db.storage(*L1_BLOCK_CONTRACT, *L1_OVERHEAD_SLOT)?;
    let l1_fee_scalar = db.storage(*L1_BLOCK_CONTRACT, *L1_SCALAR_SLOT)?;

    Ok(L1BlockInfo {
        l1_base_fee,
        l1_fee_overhead,
        l1_fee_scalar,
    })
}
