use crate::{gas_query, Return, Precompile, PrecompileOutput, PrecompileResult};

use alloc::{borrow::Cow, vec::Vec};
use primitive_types::{H160 as Address, U256};

pub mod add {
    use super::*;
    const ADDRESS: Address = crate::make_address(0, 6);

    pub const ISTANBUL: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_add(input, 150, target_gas)
        }),
    );

    pub const BYZANTIUM: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_add(input, 500, target_gas)
        }),
    );
}

pub mod mul {
    use super::*;
    const ADDRESS: Address = crate::make_address(0, 7);
    pub const ISTANBUL: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_mul(input, 6_000, target_gas)
        }),
    );

    pub const BYZANTIUM: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_mul(input, 40_000, target_gas)
        }),
    );
}

pub mod pair {
    use super::*;
    const ADDRESS: Address = crate::make_address(0, 8);

    const ISTANBUL_PAIR_PER_POINT: u64 = 34_000;
    const ISTANBUL_PAIR_BASE: u64 = 45_000;
    pub const ISTANBUL: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_pair(
                input,
                ISTANBUL_PAIR_PER_POINT,
                ISTANBUL_PAIR_BASE,
                target_gas,
            )
        }),
    );

    const BYZANTIUM_PAIR_PER_POINT: u64 = 80_000;
    const BYZANTIUM_PAIR_BASE: u64 = 100_000;
    pub const BYZANTIUM: (Address, Precompile) = (
        ADDRESS,
        Precompile::Standard(|input: &[u8], target_gas: u64| -> PrecompileResult {
            super::run_pair(
                input,
                BYZANTIUM_PAIR_PER_POINT,
                BYZANTIUM_PAIR_BASE,
                target_gas,
            )
        }),
    );
}

/// Input length for the add operation.
const ADD_INPUT_LEN: usize = 128;

/// Input length for the multiplication operation.
const MUL_INPUT_LEN: usize = 128;

/// Pair element length.
const PAIR_ELEMENT_LEN: usize = 192;

/// Reads the `x` and `y` points from an input at a given position.
fn read_point(input: &[u8], pos: usize) -> Result<bn::G1, Return> {
    use bn::{AffineG1, Fq, Group, G1};

    let mut px_buf = [0u8; 32];
    px_buf.copy_from_slice(&input[pos..(pos + 32)]);
    let px = Fq::from_slice(&px_buf)
        .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_X")))?;

    let mut py_buf = [0u8; 32];
    py_buf.copy_from_slice(&input[(pos + 32)..(pos + 64)]);
    let py = Fq::from_slice(&py_buf)
        .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_Y")))?;

    Ok(if px == Fq::zero() && py == bn::Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(px, py)
            .map_err(|_| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_POINT")))?
            .into()
    })
}

fn run_add(input: &[u8], cost: u64, target_gas: u64) -> PrecompileResult {
    let cost = gas_query(cost, target_gas)?;

    use bn::AffineG1;

    let mut input = input.to_vec();
    input.resize(ADD_INPUT_LEN, 0);

    let p1 = read_point(&input, 0)?;
    let p2 = read_point(&input, 64)?;

    let mut output = [0u8; 64];
    if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
        sum.x()
            .into_u256()
            .to_big_endian(&mut output[..32])
            .unwrap();
        sum.y()
            .into_u256()
            .to_big_endian(&mut output[32..])
            .unwrap();
    }

    Ok(PrecompileOutput::without_logs(cost, output.to_vec()))
}

fn run_mul(input: &[u8], cost: u64, target_gas: u64) -> PrecompileResult {
    let cost = gas_query(cost, target_gas)?;
    use bn::AffineG1;

    let mut input = input.to_vec();
    input.resize(MUL_INPUT_LEN, 0);

    let p = read_point(&input, 0)?;

    let mut fr_buf = [0u8; 32];
    fr_buf.copy_from_slice(&input[64..96]);
    let fr = bn::Fr::from_slice(&fr_buf[..])
        .map_err(|_| Return::Other(Cow::Borrowed("Invalid field element")))?;

    let mut out = [0u8; 64];
    if let Some(mul) = AffineG1::from_jacobian(p * fr) {
        mul.x().to_big_endian(&mut out[..32]).unwrap();
        mul.y().to_big_endian(&mut out[32..]).unwrap();
    }

    Ok(PrecompileOutput::without_logs(cost, out.to_vec()))
}

fn run_pair(
    input: &[u8],
    pair_per_point_cost: u64,
    pair_base_cost: u64,
    target_gas: u64,
) -> PrecompileResult {
    let cost = pair_per_point_cost * input.len() as u64 / PAIR_ELEMENT_LEN as u64 + pair_base_cost;
    let cost = gas_query(cost, target_gas)?;

    use bn::{AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

    if input.len() % PAIR_ELEMENT_LEN != 0 {
        return Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_LEN")));
    }

    let output = if input.is_empty() {
        U256::one()
    } else {
        let elements = input.len() / PAIR_ELEMENT_LEN;
        let mut vals = Vec::with_capacity(elements);

        for idx in 0..elements {
            let mut buf = [0u8; 32];

            buf.copy_from_slice(&input[(idx * PAIR_ELEMENT_LEN)..(idx * PAIR_ELEMENT_LEN + 32)]);
            let ax = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_AX")))?;
            buf.copy_from_slice(
                &input[(idx * PAIR_ELEMENT_LEN + 32)..(idx * PAIR_ELEMENT_LEN + 64)],
            );
            let ay = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_AY")))?;
            buf.copy_from_slice(
                &input[(idx * PAIR_ELEMENT_LEN + 64)..(idx * PAIR_ELEMENT_LEN + 96)],
            );
            let bay = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_B_AY")))?;
            buf.copy_from_slice(
                &input[(idx * PAIR_ELEMENT_LEN + 96)..(idx * PAIR_ELEMENT_LEN + 128)],
            );
            let bax = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_B_AX")))?;
            buf.copy_from_slice(
                &input[(idx * PAIR_ELEMENT_LEN + 128)..(idx * PAIR_ELEMENT_LEN + 160)],
            );
            let bby = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_B_BY")))?;
            buf.copy_from_slice(
                &input[(idx * PAIR_ELEMENT_LEN + 160)..(idx * PAIR_ELEMENT_LEN + 192)],
            );
            let bbx = Fq::from_slice(&buf)
                .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_B_BX")))?;

            let a = {
                if ax.is_zero() && ay.is_zero() {
                    G1::zero()
                } else {
                    G1::from(
                        AffineG1::new(ax, ay)
                            .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_A")))?,
                    )
                }
            };
            let b = {
                let ba = Fq2::new(bax, bay);
                let bb = Fq2::new(bbx, bby);

                if ba.is_zero() && bb.is_zero() {
                    G2::zero()
                } else {
                    G2::from(
                        AffineG2::new(ba, bb)
                            .map_err(|_e| Return::Other(Cow::Borrowed("ERR_BN128_INVALID_B")))?,
                    )
                }
            };
            vals.push((a, b))
        }

        let mul = vals
            .into_iter()
            .fold(Gt::one(), |s, (a, b)| s * bn::pairing(a, b));

        if mul == Gt::one() {
            U256::one()
        } else {
            U256::zero()
        }
    };

    let mut buf = [0u8; 32];
    output.to_big_endian(&mut buf);

    Ok(PrecompileOutput::without_logs(cost, buf.to_vec()))
}

/*
#[cfg(test)]
mod tests {
    use crate::test_utils::new_context;

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

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // zero sum test
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

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let res = Bn128Add::<Byzantium>::run(&input, 499, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // no input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_POINT")))
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

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0200000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let res = Bn128Mul::<Byzantium>::run(&input, 39_999, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // zero multiplication test
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

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // no input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            0f00000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_POINT")))
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

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
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
        let res = Bn128Pair::<Byzantium>::run(&input, 259_999, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // no input test
        let input = [0u8; 0];
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
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

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_A")))
        ));

        // invalid input length
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            111111111111111111111111111111\
        ",
        )
        .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_LEN",)))
        ));
    }
}
*/
