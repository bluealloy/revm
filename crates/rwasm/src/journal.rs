use byteorder::{ByteOrder, LittleEndian};
use fluentbase_sdk::{Bytes32, LowLevelAPI, LowLevelSDK};
use fluentbase_types::{ExitCode, POSEIDON_EMPTY};
use revm_primitives::{Address, Bytes, B256, U256};

pub(crate) const JZKT_CODESIZE_FIELD: u32 = 0;
pub(crate) const JZKT_NONCE_FIELD: u32 = 1;
pub(crate) const JZKT_CODE_HASH_FIELD: u32 = 2;
pub(crate) const JZKT_BALANCE_FIELD: u32 = 3;

/// Compression flags for upper fields, we compress
/// only code hash and balance fields (0b1100)
pub(crate) const JZKT_COMPRESSION_FLAGS: u32 = 12;

/// EIP-170: Contract code size limit
///
/// By default this limit is 0x6000 (~24kb)
pub(crate) const MAX_CODE_SIZE: u32 = 0x6000;

pub(crate) struct Account {
    pub(crate) address: Address,
    pub(crate) code_size: u64,
    pub(crate) nonce: u64,
    pub(crate) code_hash: B256,
    pub(crate) balance: U256,
}

pub(crate) type AccountCheckpoint = (u32, u32);

impl Default for Account {
    fn default() -> Self {
        Self {
            address: Address::ZERO,
            code_size: 0,
            nonce: 0,
            code_hash: POSEIDON_EMPTY,
            balance: U256::ZERO,
        }
    }
}

impl Account {
    pub(crate) fn new(address: Address) -> Self {
        Self {
            address,
            ..Default::default()
        }
    }

    pub(crate) fn new_from_jzkt(address: Address) -> Self {
        let mut result = Self::new(address);
        let address_word = address.into_word();
        // code size and nonce
        let mut buffer = [0u8; 32];
        LowLevelSDK::jzkt_get(
            address_word.as_ptr(),
            JZKT_CODESIZE_FIELD,
            buffer.as_mut_ptr(),
        );
        result.code_size = LittleEndian::read_u64(&buffer);
        LowLevelSDK::jzkt_get(address_word.as_ptr(), JZKT_NONCE_FIELD, buffer.as_mut_ptr());
        result.nonce = LittleEndian::read_u64(&buffer);
        // code hash
        LowLevelSDK::jzkt_get(
            address_word.as_ptr(),
            JZKT_CODE_HASH_FIELD,
            result.code_hash.as_mut_ptr(),
        );
        // balance
        let balance_mut = unsafe { result.balance.as_le_slice_mut() };
        LowLevelSDK::jzkt_get(
            address_word.as_ptr(),
            JZKT_BALANCE_FIELD,
            balance_mut.as_mut_ptr(),
        );
        result
    }

    pub(crate) fn write_to_jzkt(&self) {
        let mut values: [Bytes32; 4] = [[0u8; 32]; 4];
        // code size and nonce
        LittleEndian::write_u64(
            &mut values[JZKT_CODESIZE_FIELD as usize][..],
            self.code_size,
        );
        LittleEndian::write_u64(&mut values[JZKT_NONCE_FIELD as usize][..], self.nonce);
        // code hash and balance
        values[JZKT_CODE_HASH_FIELD as usize].copy_from_slice(self.code_hash.as_slice());
        values[JZKT_BALANCE_FIELD as usize].copy_from_slice(&self.balance.to_be_bytes::<32>());
        // update jzkt state
        let address_word = self.address.into_word();
        LowLevelSDK::jzkt_update(
            address_word.as_ptr(),
            JZKT_COMPRESSION_FLAGS,
            values.as_ptr(),
            32 * values.len() as u32,
        );
    }

    pub(crate) fn inc_nonce(&mut self) -> u64 {
        let prev_nonce = self.nonce;
        self.nonce += 1;
        assert_ne!(self.nonce, u64::MAX);
        prev_nonce
    }

    pub(crate) fn load_bytecode(&self) -> Bytes {
        let mut bytecode = vec![0u8; self.code_size as usize];
        LowLevelSDK::jzkt_preimage_copy(self.code_hash.as_ptr(), bytecode.as_mut_ptr());
        bytecode.into()
    }

    pub(crate) fn change_bytecode(&mut self, code: &Bytes) {
        let address_word = self.address.into_word();
        // refresh code hash
        LowLevelSDK::crypto_poseidon(
            code.as_ptr(),
            code.len() as u32,
            self.code_hash.as_mut_ptr(),
        );
        self.code_size = code.len() as u64;
        // write new changes into ZKT
        self.write_to_jzkt();
        // make sure preimage of this hash is stored
        let r = LowLevelSDK::jzkt_update_preimage(
            address_word.as_ptr(),
            JZKT_CODE_HASH_FIELD,
            code.as_ptr(),
            code.len() as u32,
        );
        assert!(r);
    }

    pub(crate) fn checkpoint() -> AccountCheckpoint {
        LowLevelSDK::jzkt_checkpoint()
    }

    pub(crate) fn commit() -> B256 {
        let mut root = B256::ZERO;
        LowLevelSDK::jzkt_commit(root.as_mut_ptr());
        root
    }

    pub(crate) fn rollback(checkpoint: AccountCheckpoint) {
        LowLevelSDK::jzkt_rollback(checkpoint.0, checkpoint.1);
    }

    pub(crate) fn create_account_checkpoint(
        caller: &mut Account,
        callee: &mut Account,
        amount: U256,
    ) -> Result<AccountCheckpoint, ExitCode> {
        let checkpoint: AccountCheckpoint = Self::checkpoint();
        // make sure there is no creation collision
        if callee.code_hash != POSEIDON_EMPTY || callee.nonce != 0 {
            LowLevelSDK::jzkt_rollback(checkpoint.0, checkpoint.1);
            return Err(ExitCode::CreateCollision);
        }
        // change balance from caller and callee
        caller.balance.checked_sub(amount).ok_or_else(|| {
            LowLevelSDK::jzkt_rollback(checkpoint.0, checkpoint.1);
            ExitCode::InsufficientBalance
        })?;
        callee.balance = callee.balance.checked_add(amount).ok_or_else(|| {
            LowLevelSDK::jzkt_rollback(checkpoint.0, checkpoint.1);
            ExitCode::OverflowPayment
        })?;
        // change nonce (we are always on spurious dragon)
        caller.nonce = 1;
        Ok(checkpoint)
    }

    pub(crate) fn sub_balance(&mut self, amount: U256) -> Result<(), ExitCode> {
        self.balance = self
            .balance
            .checked_sub(amount)
            .ok_or(ExitCode::InsufficientBalance)?;
        Ok(())
    }

    pub(crate) fn sub_balance_saturating(&mut self, amount: U256) {
        self.balance = self.balance.saturating_sub(amount);
    }

    pub(crate) fn add_balance(&mut self, amount: U256) -> Result<(), ExitCode> {
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or(ExitCode::OverflowPayment)?;
        Ok(())
    }

    pub(crate) fn add_balance_saturating(&mut self, amount: U256) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub(crate) fn transfer(
        from: &mut Account,
        to: &mut Account,
        amount: U256,
    ) -> Result<(), ExitCode> {
        // update balances
        from.sub_balance(amount)?;
        to.add_balance(amount)?;
        // commit new balances into jzkt
        from.write_to_jzkt();
        to.write_to_jzkt();
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn is_not_empty(&self) -> bool {
        self.nonce != 0 || self.code_hash != POSEIDON_EMPTY
    }
}
