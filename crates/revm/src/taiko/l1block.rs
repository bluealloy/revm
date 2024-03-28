use crate::primitives::{address, db::Database, Address, SpecId, U256};

const L1_DUMMY_SLOT: U256 = U256::from_limbs([1u64, 0, 0, 0]);

/// The address of the L1Block contract.
pub const L1_BLOCK_CONTRACT: Address = address!("4200000000000000000000000000000000000015");

/// L1 block info
#[derive(Clone, Debug, Default)]
pub struct L1BlockInfo {
    /// TODO(Cecilia): Reserve for BBR
    pub l1_dummy: U256,
}

impl L1BlockInfo {
    /// Try to fetch the L1 block info from the database.
    pub fn try_fetch<DB: Database>(db: &mut DB, spec_id: SpecId) -> Result<L1BlockInfo, DB::Error> {
        // Ensure the L1 Block account is loaded into the cache after Ecotone. With EIP-4788, it is no longer the case
        // that the L1 block account is loaded into the cache prior to the first inquiry for the L1 block info.
        if spec_id.is_enabled_in(SpecId::CANCUN) {
            let _ = db.basic(L1_BLOCK_CONTRACT)?;
        }

        let l1_dummy = db.storage(L1_BLOCK_CONTRACT, L1_DUMMY_SLOT)?;

        Ok(L1BlockInfo {
            l1_dummy,
            ..Default::default()
        })
    }
}
