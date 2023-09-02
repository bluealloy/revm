//! Optimism-specific constants, types, and helpers.

use once_cell::sync::Lazy;
use revm_interpreter::primitives::{db::Database, hex_literal::hex, Address, U256};

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
    /// Fetches the L1 block info from the `L1Block` contract in the database.
    pub fn try_fetch<DB: Database>(db: &mut DB) -> Result<L1BlockInfo, DB::Error> {
        let l1_base_fee = db.storage(*L1_BLOCK_CONTRACT, *L1_BASE_FEE_SLOT)?;
        let l1_fee_overhead = db.storage(*L1_BLOCK_CONTRACT, *L1_OVERHEAD_SLOT)?;
        let l1_fee_scalar = db.storage(*L1_BLOCK_CONTRACT, *L1_SCALAR_SLOT)?;

        Ok(L1BlockInfo {
            l1_base_fee,
            l1_fee_overhead,
            l1_fee_scalar,
        })
    }
}
