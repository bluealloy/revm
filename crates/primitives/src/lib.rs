//! # revm-primitives
//!
//! EVM primitive types.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

mod constants;
pub use constants::*;

pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, keccak256, ruint, uint, Address,
    Bytes, FixedBytes, Log, LogData, TxKind, B256, I256, U256,
};

pub use alloy_primitives::map::{self, hash_map, hash_set, HashMap, HashSet};

/// The Keccak-256 hash of the empty string `""`.
pub const KECCAK_EMPTY: B256 =
    b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");

macro_rules! mmm {
    ( [ $($x:ident),* ] $(,$ret:expr)? ) => {
        let [$( $x ),*] = test();
        return $($ret)?;
        // #[derive(Debug, Clone, serde::Serialize)]
        // #[serde(rename_all = "camelCase")]
        // pub struct $name
        // {
        //     $(pub $field_name : $field_type,) *
        // }
    };
}

pub fn test<const N: usize>() -> [u8; N] {
    let mut arr = [0u8; N];
    arr
}

pub fn mm() -> bool {
    //mmm!([a, b, c], false);
    mmm!([a, b], false);
    //return false
}
