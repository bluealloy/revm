//! EOF related constants and functions.
use crate::{keccak256, Address, B256};

/// TXCREATE transaction type.
pub const INITCODE_TX_TYPE: u8 = 0x06;
/// Maximum number of initcode in TXCREATE transactions.
pub const MAX_INITCODE_COUNT: usize = 256;

/// Calculated new EOF address from address and salt.
///
/// Buffer that is hashed is 65 bytes long. First bytes is magic number 0xFF,
/// than comes 12 zeros, than 20 byte of address and in the end 32 bytes of salt.
///
///
/// | 0xFF | zero padding (12 bytes) | Address (20 bytes) | salt (32 bytes).
#[inline]
pub fn new_eof_address(address: Address, salt: B256) -> Address {
    let mut buffer = [0; 65];
    buffer[0] = 0xff;
    // 1..13 are padded zeroes
    buffer[13..].copy_from_slice(address.as_ref());
    buffer[33..].copy_from_slice(salt.as_ref());
    Address::from_word(keccak256(buffer))
}
