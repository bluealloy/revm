//! BN128 precompiles added in [`EIP-1962`](https://eips.ethereum.org/EIPS/eip-1962)
use crate::{
    utilities::{bool_to_bytes32, right_pad},
    Address, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
};
use std::vec::Vec;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        mod substrate;
        use substrate::{
            encode_g1_point, g1_point_add, g1_point_mul, pairing_check, read_g1_point, read_g2_point,
            read_scalar,
        };
    } else {
        mod arkworks;
        use arkworks::{
            encode_g1_point, g1_point_add, g1_point_mul, pairing_check, read_g1_point, read_g2_point,
            read_scalar,
        };
    }
}

/// Bn128 add precompile
pub mod add {
    use super::*;

    /// Bn128 add precompile address
    pub const ADDRESS: Address = crate::u64_to_address(6);

    /// Bn128 add precompile with ISTANBUL gas rules
    pub const ISTANBUL_ADD_GAS_COST: u64 = 150;

    /// Bn128 add precompile with ISTANBUL gas rules
    pub const ISTANBUL: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_add(input, ISTANBUL_ADD_GAS_COST, gas_limit)
        });

    /// Bn128 add precompile with BYZANTIUM gas rules
    pub const BYZANTIUM_ADD_GAS_COST: u64 = 500;

    /// Bn128 add precompile with BYZANTIUM gas rules
    pub const BYZANTIUM: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_add(input, BYZANTIUM_ADD_GAS_COST, gas_limit)
        });
}

/// Bn128 mul precompile
pub mod mul {
    use super::*;

    /// Bn128 mul precompile address
    pub const ADDRESS: Address = crate::u64_to_address(7);

    /// Bn128 mul precompile with ISTANBUL gas rules
    pub const ISTANBUL_MUL_GAS_COST: u64 = 6_000;

    /// Bn128 mul precompile with ISTANBUL gas rules
    pub const ISTANBUL: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_mul(input, ISTANBUL_MUL_GAS_COST, gas_limit)
        });

    /// Bn128 mul precompile with BYZANTIUM gas rules
    pub const BYZANTIUM_MUL_GAS_COST: u64 = 40_000;

    /// Bn128 mul precompile with BYZANTIUM gas rules
    pub const BYZANTIUM: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_mul(input, BYZANTIUM_MUL_GAS_COST, gas_limit)
        });
}

/// Bn128 pair precompile
pub mod pair {
    use super::*;

    /// Bn128 pair precompile address
    pub const ADDRESS: Address = crate::u64_to_address(8);

    /// Bn128 pair precompile with ISTANBUL gas rules
    pub const ISTANBUL_PAIR_PER_POINT: u64 = 34_000;

    /// Bn128 pair precompile with ISTANBUL gas rules
    pub const ISTANBUL_PAIR_BASE: u64 = 45_000;

    /// Bn128 pair precompile with ISTANBUL gas rules
    pub const ISTANBUL: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_pair(
                input,
                ISTANBUL_PAIR_PER_POINT,
                ISTANBUL_PAIR_BASE,
                gas_limit,
            )
        });

    /// Bn128 pair precompile with BYZANTIUM gas rules
    pub const BYZANTIUM_PAIR_PER_POINT: u64 = 80_000;

    /// Bn128 pair precompile with BYZANTIUM gas rules
    pub const BYZANTIUM_PAIR_BASE: u64 = 100_000;

    /// Bn128 pair precompile with BYZANTIUM gas rules
    pub const BYZANTIUM: PrecompileWithAddress =
        PrecompileWithAddress(ADDRESS, |input, gas_limit| {
            run_pair(
                input,
                BYZANTIUM_PAIR_PER_POINT,
                BYZANTIUM_PAIR_BASE,
                gas_limit,
            )
        });
}

/// FQ_LEN specifies the number of bytes needed to represent an
/// Fq element. This is an element in the base field of BN254.
///
/// Note: The base field is used to define G1 and G2 elements.
const FQ_LEN: usize = 32;

/// SCALAR_LEN specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BN254.
const SCALAR_LEN: usize = 32;

/// FQ2_LEN specifies the number of bytes needed to represent an
/// Fq^2 element.
///
/// Note: This is the quadratic extension of Fq, and by definition
/// means we need 2 Fq elements.
const FQ2_LEN: usize = 2 * FQ_LEN;

/// G1_LEN specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fq elements.
const G1_LEN: usize = 2 * FQ_LEN;
/// G2_LEN specifies the number of bytes needed to represent a G2 element.
///
/// Note: A G2 element contains 2 Fq^2 elements.
const G2_LEN: usize = 2 * FQ2_LEN;

/// Input length for the add operation.
/// `ADD` takes two uncompressed G1 points (64 bytes each).
pub const ADD_INPUT_LEN: usize = 2 * G1_LEN;

/// Input length for the multiplication operation.
/// `MUL` takes an uncompressed G1 point (64 bytes) and scalar (32 bytes).
pub const MUL_INPUT_LEN: usize = G1_LEN + SCALAR_LEN;

/// Pair element length.
/// `PAIR` elements are composed of an uncompressed G1 point (64 bytes) and an uncompressed G2 point
/// (128 bytes).
pub const PAIR_ELEMENT_LEN: usize = G1_LEN + G2_LEN;

/// Run the Bn128 add precompile
pub fn run_add(input: &[u8], gas_cost: u64, gas_limit: u64) -> PrecompileResult {
    if gas_cost > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let input = right_pad::<ADD_INPUT_LEN>(input);

    let p1 = read_g1_point(&input[..G1_LEN])?;
    let p2 = read_g1_point(&input[G1_LEN..])?;
    let result = g1_point_add(p1, p2);

    let output = encode_g1_point(result);

    Ok(PrecompileOutput::new(gas_cost, output.into()))
}

/// Run the Bn128 mul precompile
pub fn run_mul(input: &[u8], gas_cost: u64, gas_limit: u64) -> PrecompileResult {
    if gas_cost > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let input = right_pad::<MUL_INPUT_LEN>(input);

    let p = read_g1_point(&input[..G1_LEN])?;

    let scalar = read_scalar(&input[G1_LEN..G1_LEN + SCALAR_LEN]);
    let result = g1_point_mul(p, scalar);

    let output = encode_g1_point(result);

    Ok(PrecompileOutput::new(gas_cost, output.into()))
}

/// Run the Bn128 pair precompile
pub fn run_pair(
    input: &[u8],
    pair_per_point_cost: u64,
    pair_base_cost: u64,
    gas_limit: u64,
) -> PrecompileResult {
    let gas_used = (input.len() / PAIR_ELEMENT_LEN) as u64 * pair_per_point_cost + pair_base_cost;
    if gas_used > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() % PAIR_ELEMENT_LEN != 0 {
        return Err(PrecompileError::Bn128PairLength);
    }

    let elements = input.len() / PAIR_ELEMENT_LEN;

    let mut points = Vec::with_capacity(elements);

    for idx in 0..elements {
        // Offset to the start of the pairing element at index `idx` in the byte slice
        let start = idx * PAIR_ELEMENT_LEN;
        let g1_start = start;
        // Offset to the start of the G2 element in the pairing element
        // This is where G1 ends.
        let g2_start = start + G1_LEN;

        let encoded_g1_element = &input[g1_start..g2_start];
        let encoded_g2_element = &input[g2_start..g2_start + G2_LEN];

        // If either the G1 or G2 element is the encoded representation
        // of the point at infinity, then these two points are no-ops
        // in the pairing computation.
        //
        // Note: we do not skip the validation of these two elements even if
        // one of them is the point at infinity because we could have G1 be
        // the point at infinity and G2 be an invalid element or vice versa.
        // In that case, the precompile should error because one of the elements
        // was invalid.
        let g1_is_zero = encoded_g1_element.iter().all(|i| *i == 0);
        let g2_is_zero = encoded_g2_element.iter().all(|i| *i == 0);

        // Get G1 and G2 points from the input
        let a = read_g1_point(encoded_g1_element)?;
        let b = read_g2_point(encoded_g2_element)?;

        if !g1_is_zero && !g2_is_zero {
            points.push((a, b));
        }
    }

    let success = pairing_check(&points);

    Ok(PrecompileOutput::new(gas_used, bool_to_bytes32(success)))
}

#[cfg(test)]
mod tests {
    use crate::{
        bn128::{
            add::BYZANTIUM_ADD_GAS_COST,
            mul::BYZANTIUM_MUL_GAS_COST,
            pair::{BYZANTIUM_PAIR_BASE, BYZANTIUM_PAIR_PER_POINT},
        },
        PrecompileError,
    };
    use primitives::hex;

    use super::*;

    #[test]
    fn test_alt_bn128_add() {
        let input = hex::decode(
            "\
             18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9\
             063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266\
             07c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed\
             06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703\
            301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915",
        )
        .unwrap();

        let outcome = run_add(&input, BYZANTIUM_ADD_GAS_COST, 500).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Zero sum test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let outcome = run_add(&input, BYZANTIUM_ADD_GAS_COST, 500).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = run_add(&input, BYZANTIUM_ADD_GAS_COST, 499);

        assert!(matches!(res, Err(PrecompileError::OutOfGas)));

        // No input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let outcome = run_add(&input, BYZANTIUM_ADD_GAS_COST, 500).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let res = run_add(&input, BYZANTIUM_ADD_GAS_COST, 500);
        assert!(matches!(
            res,
            Err(PrecompileError::Bn128AffineGFailedToCreate)
        ));
    }

    #[test]
    fn test_alt_bn128_mul() {
        let input = hex::decode(
            "\
            2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7\
            21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204\
            00000000000000000000000000000000000000000000000011138ce750fa15c2",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c\
            031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc",
        )
        .unwrap();

        let outcome = run_mul(&input, BYZANTIUM_MUL_GAS_COST, 40_000).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0200000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = run_mul(&input, BYZANTIUM_MUL_GAS_COST, 39_999);
        assert!(matches!(res, Err(PrecompileError::OutOfGas)));

        // Zero multiplication test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0200000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let outcome = run_mul(&input, BYZANTIUM_MUL_GAS_COST, 40_000).unwrap();
        assert_eq!(outcome.bytes, expected);

        // No input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let outcome = run_mul(&input, BYZANTIUM_MUL_GAS_COST, 40_000).unwrap();
        assert_eq!(outcome.bytes, expected);

        // Point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            0f00000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = run_mul(&input, BYZANTIUM_MUL_GAS_COST, 40_000);
        assert!(matches!(
            res,
            Err(PrecompileError::Bn128AffineGFailedToCreate)
        ));
    }

    #[test]
    fn test_alt_bn128_pair() {
        let input = hex::decode(
            "\
            1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
            3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
            209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
            04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
            2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
            120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
            111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
            2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
            198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
            1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
            090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
            12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        )
        .unwrap();
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let outcome = run_pair(
            &input,
            BYZANTIUM_PAIR_PER_POINT,
            BYZANTIUM_PAIR_BASE,
            260_000,
        )
        .unwrap();
        assert_eq!(outcome.bytes, expected);

        // Out of gas test
        let input = hex::decode(
            "\
            1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
            3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
            209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
            04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
            2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
            120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
            111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
            2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
            198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
            1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
            090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
            12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        )
        .unwrap();

        let res = run_pair(
            &input,
            BYZANTIUM_PAIR_PER_POINT,
            BYZANTIUM_PAIR_BASE,
            259_999,
        );
        assert!(matches!(res, Err(PrecompileError::OutOfGas)));

        // No input test
        let input = [0u8; 0];
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let outcome = run_pair(
            &input,
            BYZANTIUM_PAIR_PER_POINT,
            BYZANTIUM_PAIR_BASE,
            260_000,
        )
        .unwrap();
        assert_eq!(outcome.bytes, expected);

        // Point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let res = run_pair(
            &input,
            BYZANTIUM_PAIR_PER_POINT,
            BYZANTIUM_PAIR_BASE,
            260_000,
        );
        assert!(matches!(
            res,
            Err(PrecompileError::Bn128AffineGFailedToCreate)
        ));

        // Invalid input length
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            111111111111111111111111111111\
        ",
        )
        .unwrap();

        let res = run_pair(
            &input,
            BYZANTIUM_PAIR_PER_POINT,
            BYZANTIUM_PAIR_BASE,
            260_000,
        );
        assert!(matches!(res, Err(PrecompileError::Bn128PairLength)));
    }
}
