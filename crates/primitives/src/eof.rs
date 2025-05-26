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
    buffer[13..33].copy_from_slice(address.as_ref());
    buffer[33..].copy_from_slice(salt.as_ref());
    Address::from_word(keccak256(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, b256};

    #[test]
    fn test_new_eof_address() {
        let address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        let salt = b256!("0x0000000000000000000000000000000000000000000000000000000000000000");
        let eof_address = new_eof_address(address, salt);
        assert_eq!(
            eof_address,
            address!("0x02b6826e9392ee6bf6479e413c570846ab0107ec")
        );
    }
}
