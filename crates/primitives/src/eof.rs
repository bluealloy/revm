use crate::{keccak256, Address, B256};

/// Calculated new EOF address from salt, address and init code hash.
///
/// Address is left padded with 12 zeroes.
#[inline]
pub fn new_address(address: Address, salt: B256) -> Address {
    let mut buffer = [0; 85];
    buffer[0] = 0xff;
    // 1..13 are padded zeroes
    buffer[13..].copy_from_slice(address.as_ref());
    buffer[60..].copy_from_slice(salt.as_ref());
    Address::from_word(keccak256(buffer))
}
