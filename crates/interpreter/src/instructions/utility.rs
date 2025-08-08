use primitives::{Address, B256, U256};

/// Trait for converting types into U256 values.
pub trait IntoU256 {
    /// Converts the implementing type into a U256 value.
    fn into_u256(self) -> U256;
}

impl IntoU256 for Address {
    fn into_u256(self) -> U256 {
        self.into_word().into_u256()
    }
}

impl IntoU256 for B256 {
    fn into_u256(self) -> U256 {
        U256::from_be_bytes(self.0)
    }
}

/// Trait for converting types into Address values. It ingnores excess bytes.
pub trait IntoAddress {
    /// Converts the implementing type into an Address value.
    fn into_address(self) -> Address;
}

impl IntoAddress for U256 {
    fn into_address(self) -> Address {
        Address::from_word(B256::from(self.to_be_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use primitives::address;

    use super::*;

    #[test]
    fn test_into_u256() {
        let addr = address!("0x0000000000000000000000000000000000000001");
        let u256 = addr.into_u256();
        assert_eq!(u256, U256::from(0x01));
        assert_eq!(u256.into_address(), addr);
    }
}
