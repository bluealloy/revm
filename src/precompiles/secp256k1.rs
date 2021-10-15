use crate::precompiles::{Precompile, PrecompileOutput, PrecompileResult};
use crate::{CallContext, ExitError};
use core::cmp::min;
use primitive_types::{H160 as Address, H256};
use sha3::{Digest};
mod costs {
    pub(super) const ECRECOVER_BASE: u64 = 3_000;
}
mod consts {
    pub(super) const INPUT_LEN: usize = 128;
}

//use libsecp256k1::ThirtyTwoByteHash;
use parity_crypto::{
    publickey::{public_to_address, recover, Error as ParityCryptoError, Signature},
};

pub(super) struct ECRecover;

impl ECRecover {
    pub(super) const ADDRESS: Address = super::make_address(0, 1);

    // return padded address as H256
    fn secp256k1_ecdsa_recover(
        sig: &[u8; 65],
        msg: &[u8; 32],
    ) -> Result<Address, ParityCryptoError> {
        let rs = Signature::from_electrum(&sig[..]);
        if rs == Signature::default() {
            return Err(ParityCryptoError::InvalidSignature);
        }
        let msg = H256::from_slice(msg);
        let address = public_to_address(&recover(&rs, &msg)?);
        Ok(address)
    }
}

/// Error verifying ECDSA signature
pub enum EcdsaVerifyError {
    /// Incorrect value of R or S
    BadRS,
    /// Incorrect value of V
    BadV,
    /// Invalid signature
    BadSignature,
}

impl Precompile for ECRecover {
    fn run(
        i: &[u8],
        target_gas: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        let cost = costs::ECRECOVER_BASE;
        if cost > target_gas {
            return Err(ExitError::OutOfGas);
        }
        let mut input = [0u8; 128];
        input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

        let mut msg = [0u8; 32];
        let mut sig = [0u8; 65];

        msg[0..32].copy_from_slice(&input[0..32]);
        sig[0..32].copy_from_slice(&input[64..96]);
        sig[32..64].copy_from_slice(&input[96..128]);

        // TODO do this correctly: return if there is junk in V.
        if input[32..63] != [0u8; 31] || !matches!(input[63], 27 | 28) {
            return Ok(PrecompileOutput::without_logs(cost, Vec::new()));
        }

        // TODO hm it will fail for chainId that are more then one byte;
        sig[64] = input[63];

        let out = match Self::secp256k1_ecdsa_recover(&sig, &msg) {
            Ok(out) => H256::from(out).as_bytes().to_vec(),
            Err(_) => Vec::new(),
        };

        Ok(PrecompileOutput::without_logs(cost, out))
    }
}

/*

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::new_context;

    fn ecverify(hash: H256, signature: &[u8], signer: Address) -> bool {
        matches!(ecrecover(hash, signature), Ok(s) if s == signer)
    }

    #[test]
    fn test_ecverify() {
        let hash = H256::from_slice(
            &hex::decode("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap(),
        );
        let signature =
            &hex::decode("b9f0bb08640d3c1c00761cdd0121209268f6fd3816bc98b9e6f3cc77bf82b69812ac7a61788a0fdc0e19180f14c945a8e1088a27d92a74dce81c0981fb6447441b")
                .unwrap();
        let signer =
            Address::from_slice(&hex::decode("1563915e194D8CfBA1943570603F7606A3115508").unwrap());
        assert!(ecverify(hash, &signature, signer));
    }

    #[test]
    fn test_ecrecover() {
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
        let expected =
            hex::decode("000000000000000000000000c08b5542d177ac6686946920409741463a15dddb")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();

        let res = ECRecover::run(&input, 2_999, &new_context(), false);
        assert!(matches!(res, Err(ExitError::OutOfGas)));

        // bad inputs
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001a650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001b").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // Why is this test returning an address???
        // let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        // let expected = hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        //
        // let res = ecrecover_raw(&input, Some(500)).unwrap().output;
        // assert_eq!(res, expected);
    }
}
*/
