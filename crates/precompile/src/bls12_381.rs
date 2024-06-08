use crate::PrecompileWithAddress;

mod g1;
pub mod g1_add;
pub mod g1_msm;
pub mod g1_mul;
mod g2;
pub mod g2_add;
pub mod g2_msm;
pub mod g2_mul;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
mod msm;
pub mod pairing;
mod utils;

/// Returns the BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    [
        g1_add::PRECOMPILE,
        g1_mul::PRECOMPILE,
        g1_msm::PRECOMPILE,
        g2_add::PRECOMPILE,
        g2_mul::PRECOMPILE,
        g2_msm::PRECOMPILE,
        pairing::PRECOMPILE,
        map_fp_to_g1::PRECOMPILE,
        map_fp2_to_g2::PRECOMPILE,
    ]
    .into_iter()
}

#[cfg(test)]
mod test {
    use super::g1_add;
    use super::g1_msm;
    use super::g1_mul;
    use super::g2_add;
    use super::g2_msm;
    use super::g2_mul;
    use super::map_fp2_to_g2;
    use super::map_fp_to_g1;
    use super::msm::msm_required_gas;
    use super::pairing;
    use eyre::Result;
    use revm_primitives::{hex::FromHex, Bytes, PrecompileResult};
    use rstest::rstest;
    use serde_derive::{Deserialize, Serialize};
    use std::{fs, path::Path};

    /// Test vector structure for BLS12-381 precompile tests.
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct TestVector {
        input: String,
        expected: Option<String>,
        name: String,
        gas: Option<u64>,
        expected_error: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestVectors(Vec<TestVector>);

    fn load_test_vectors<P: AsRef<Path>>(path: P) -> Result<TestVectors> {
        let file_contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file_contents)?)
    }

    #[rstest]
    #[case::fail_g1_add(g1_add::g1_add, "fail-add_G1_bls.json")]
    #[case::fail_g1_mul(g1_mul::g1_mul, "fail-mul_G1_bls.json")]
    #[case::fail_g1_msm(g1_msm::g1_msm, "fail-multiexp_G1_bls.json")]
    #[case::fail_g2_add(g2_add::g2_add, "fail-add_G2_bls.json")]
    #[case::fail_g2_mul(g2_mul::g2_mul, "fail-mul_G2_bls.json")]
    #[case::fail_g2_msm(g2_msm::g2_msm, "fail-multiexp_G2_bls.json")]
    #[case::fail_pairing(pairing::pairing, "fail-pairing_check_bls.json")]
    #[case::fail_map_fp_to_g1(map_fp_to_g1::map_fp_to_g1, "fail-map_fp_to_G1_bls.json")]
    #[case::fail_map_fp2_to_g2(map_fp2_to_g2::map_fp2_to_g2, "fail-map_fp2_to_G2_bls.json")]
    #[case::g1_add(g1_add::g1_add, "add_G1_bls.json")]
    #[case::g1_mul(g1_mul::g1_mul, "mul_G1_bls.json")]
    #[case::g1_msm(g1_msm::g1_msm, "multiexp_G1_bls.json")]
    #[case::g2_add(g2_add::g2_add, "add_G2_bls.json")]
    #[case::g2_mul(g2_mul::g2_mul, "mul_G2_bls.json")]
    #[case::g2_msm(g2_msm::g2_msm, "multiexp_G2_bls.json")]
    #[case::pairing(pairing::pairing, "pairing_check_bls.json")]
    #[case::map_fp_to_g1(map_fp_to_g1::map_fp_to_g1, "map_fp_to_G1_bls.json")]
    #[case::map_fp2_to_g2(map_fp2_to_g2::map_fp2_to_g2, "map_fp2_to_G2_bls.json")]
    fn test_bls(
        #[case] precompile: fn(input: &Bytes, gas_limit: u64) -> PrecompileResult,
        #[case] file_name: &str,
    ) {
        let test_vectors = load_test_vectors(format!("test-vectors/{file_name}"))
            .unwrap_or_else(|e| panic!("Failed to load test vectors from {file_name}: {e}"));

        for vector in test_vectors.0 {
            let test_name = format!("{file_name}/{}", vector.name);
            let input = Bytes::from_hex(vector.input.clone()).unwrap_or_else(|e| {
                panic!(
                    "could not deserialize input {} as hex in {test_name}: {e}",
                    &vector.input
                )
            });
            let target_gas: u64 = 30_000_000;
            let res = precompile(&input, target_gas);
            if let Some(expected_error) = vector.expected_error {
                assert!(res.is_err(), "expected error {expected_error} didn't happen in {test_name}, got result {res:?}");
            } else {
                let Some(gas) = vector.gas else {
                    panic!("gas is missing in {test_name}");
                };
                let outcome = res.unwrap_or_else(|e: revm_primitives::PrecompileErrors| {
                    panic!("precompile call failed for {test_name}: {e}")
                });
                assert_eq!(
                    gas, outcome.gas_used,
                    "expected gas: {}, actual gas: {} in {test_name}",
                    gas, outcome.gas_used
                );
                let Some(expected) = vector.expected else {
                    panic!("expected output is missing in {test_name}");
                };
                let expected_output = Bytes::from_hex(expected).unwrap();
                assert_eq!(
                    expected_output, outcome.bytes,
                    "expected output: {expected_output}, actual output: {:?} in {test_name}",
                    outcome.bytes
                );
            }
        }
    }

    #[rstest]
    #[case::g1_empty(0, g1_mul::BASE_GAS_FEE, 0)]
    #[case::g1_one_item(160, g1_mul::BASE_GAS_FEE, 14400)]
    #[case::g1_two_items(320, g1_mul::BASE_GAS_FEE, 21312)]
    #[case::g1_ten_items(1600, g1_mul::BASE_GAS_FEE, 50760)]
    #[case::g1_sixty_four_items(10240, g1_mul::BASE_GAS_FEE, 170496)]
    #[case::g1_one_hundred_twenty_eight_items(20480, g1_mul::BASE_GAS_FEE, 267264)]
    #[case::g1_one_hundred_twenty_nine_items(20640, g1_mul::BASE_GAS_FEE, 269352)]
    #[case::g1_two_hundred_fifty_six_items(40960, g1_mul::BASE_GAS_FEE, 534528)]
    fn test_g1_msm_required_gas(
        #[case] input_len: usize,
        #[case] multiplication_cost: u64,
        #[case] expected_output: u64,
    ) {
        let k = input_len / g1_mul::INPUT_LENGTH;

        let actual_output = msm_required_gas(k, multiplication_cost);

        assert_eq!(expected_output, actual_output);
    }
}
