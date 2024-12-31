use revm::primitives::{address, fixed_bytes, Address, FixedBytes};

pub const BURNTPIX_MAIN_ADDRESS: Address = address!("49206861766520746f6f206d7563682074696d65");
pub const BURNTPIX_ADDRESS_ONE: Address = address!("0a743ba7304efcc9e384ece9be7631e2470e401e");
pub const BURNTPIX_ADDRESS_TWO: Address = address!("c917e98213a05d271adc5d93d2fee6c1f1006f75");
pub const BURNTPIX_ADDRESS_THREE: Address = address!("f529c70db0800449ebd81fbc6e4221523a989f05");

pub const STORAGE_ZERO: FixedBytes<32> =
    fixed_bytes!("000000000000000000000000f529c70db0800449ebd81fbc6e4221523a989f05");
pub const STORAGE_ONE: FixedBytes<32> =
    fixed_bytes!("0000000000000000000000000a743ba7304efcc9e384ece9be7631e2470e401e");
pub const STORAGE_TWO: FixedBytes<32> =
    fixed_bytes!("000000000000000000000000c917e98213a05d271adc5d93d2fee6c1f1006f75");

pub const BURNTPIX_BYTECODE_ONE: &str = include_str!("bytecode_one.hex");
pub const BURNTPIX_BYTECODE_TWO: &str = include_str!("bytecode_two.hex");
pub const BURNTPIX_BYTECODE_THREE: &str = include_str!("bytecode_three.hex");
pub const BURNTPIX_BYTECODE_FOUR: &str = include_str!("bytecode_four.hex");
