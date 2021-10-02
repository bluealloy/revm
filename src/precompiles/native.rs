// use crate::parameters::PromiseCreateArgs;
// use crate::prelude::Address;
// #[cfg(not(feature = "contract"))]
// use crate::prelude::Vec;
// use evm::{Context, ExitError};
// #[cfg(feature = "contract")]
// use {
//     crate::parameters::WithdrawCallArgs,
//     crate::prelude::{is_valid_account_id, Cow, String, ToString, TryInto, U256},
//     crate::storage::{bytes_to_key, KeyPrefix},
//     crate::types::AccountId,
//     borsh::BorshSerialize,
// };

// use super::{Precompile, PrecompileResult};

// const ERR_TARGET_TOKEN_NOT_FOUND: &str = "Target token not found";

// use crate::precompiles::PrecompileOutput;
// use crate::state::AuroraStackState;

// trait ReturnPromise {
//     fn promise(&self, state: &mut AuroraStackState) -> PromiseCreateArgs;
// }

// mod costs {
//     use crate::types::Gas;

//     // TODO(#51): Determine the correct amount of gas
//     pub(super) const EXIT_TO_NEAR_GAS: Gas = 0;

//     // TODO(#51): Determine the correct amount of gas
//     pub(super) const EXIT_TO_ETHEREUM_GAS: Gas = 0;

//     // TODO(#51): Determine the correct amount of gas
//     pub(super) const FT_TRANSFER_GAS: Gas = 100_000_000_000_000;

//     // TODO(#51): Determine the correct amount of gas
//     pub(super) const WITHDRAWAL_GAS: Gas = 100_000_000_000_000;
// }

// pub struct ExitToNear; //TransferEthToNear

// impl ExitToNear {
//     /// Exit to NEAR precompile address
//     ///
//     /// Address: `0xe9217bc70b7ed1f598ddd3199e80b093fa71124f`
//     /// This address is computed as: `&keccak("exitToNear")[12..]`
//     pub(crate) const ADDRESS: Address =
//         super::make_address(0xe9217bc7, 0x0b7ed1f598ddd3199e80b093fa71124f);
// }

// #[cfg(feature = "contract")]
// fn get_nep141_from_erc20(erc20_token: &[u8]) -> AccountId {
//     AccountId::from_utf8(
//         crate::sdk::read_storage(bytes_to_key(KeyPrefix::Erc20Nep141Map, erc20_token).as_slice())
//             .expect(ERR_TARGET_TOKEN_NOT_FOUND),
//     )
//     .unwrap()
// }

// impl Precompile for ExitToNear {
//     fn required_gas(_input: &[u8]) -> Result<u64, ExitError> {
//         Ok(costs::EXIT_TO_NEAR_GAS)
//     }

//     #[cfg(not(feature = "contract"))]
//     fn run(
//         input: &[u8],
//         target_gas: u64,
//         _context: &CallContext,
//         _is_static: bool,
//     ) -> PrecompileResult {
//         if Self::required_gas(input)? > target_gas {
//             return Err(ExitError::OutOfGas);
//         }

//         Ok(PrecompileOutput {
//             output: Vec::new(),
//             cost: 0,
//             logs: Vec::new(),
//             promise: None,
//         })
//     }

//     #[cfg(feature = "contract")]
//     fn run(input: &[u8], target_gas: u64, context: &CallContext, is_static: bool) -> PrecompileResult {
//         if Self::required_gas(input)? > target_gas {
//             return Err(ExitError::OutOfGas);
//         }

//         // It's not allowed to call exit precompiles in static mode
//         if is_static {
//             return Err(ExitError::Other(Cow::from("ERR_INVALID_IN_STATIC")));
//         }

//         // First byte of the input is a flag, selecting the behavior to be triggered:
//         //      0x0 -> Eth transfer
//         //      0x1 -> Erc20 transfer
//         let mut input = input;
//         let flag = input[0];
//         input = &input[1..];

//         let (nep141_address, args) = match flag {
//             0x0 => {
//                 // ETH transfer
//                 //
//                 // Input slice format:
//                 //      recipient_account_id (bytes) - the NEAR recipient account which will receive NEP-141 ETH tokens

//                 if is_valid_account_id(input) {
//                     (
//                         String::from_utf8(crate::sdk::current_account_id()).unwrap(),
//                         // There is no way to inject json, given the encoding of both arguments
//                         // as decimal and valid account id respectively.
//                         crate::prelude::format!(
//                             r#"{{"receiver_id": "{}", "amount": "{}", "memo": null}}"#,
//                             String::from_utf8(input.to_vec()).unwrap(),
//                             context.apparent_value.as_u128()
//                         ),
//                     )
//                 } else {
//                     return Err(ExitError::Other(Cow::from(
//                         "ERR_INVALID_RECEIVER_ACCOUNT_ID",
//                     )));
//                 }
//             }
//             0x1 => {
//                 // ERC20 transfer
//                 //
//                 // This precompile branch is expected to be called from the ERC20 burn function\
//                 //
//                 // Input slice format:
//                 //      amount (U256 big-endian bytes) - the amount that was burned
//                 //      recipient_account_id (bytes) - the NEAR recipient account which will receive NEP-141 tokens

//                 if context.apparent_value != U256::from(0) {
//                     return Err(ExitError::Other(Cow::from(
//                         "ERR_ETH_ATTACHED_FOR_ERC20_EXIT",
//                     )));
//                 }

//                 let nep141_address = get_nep141_from_erc20(context.caller.as_bytes());

//                 let amount = U256::from_big_endian(&input[..32]).as_u128();
//                 input = &input[32..];

//                 if is_valid_account_id(input) {
//                     let receiver_account_id: AccountId = String::from_utf8(input.to_vec()).unwrap();
//                     (
//                         nep141_address,
//                         // There is no way to inject json, given the encoding of both arguments
//                         // as decimal and valid account id respectively.
//                         crate::prelude::format!(
//                             r#"{{"receiver_id": "{}", "amount": "{}", "memo": null}}"#,
//                             receiver_account_id,
//                             amount
//                         ),
//                     )
//                 } else {
//                     return Err(ExitError::Other(Cow::from(
//                         "ERR_INVALID_RECEIVER_ACCOUNT_ID",
//                     )));
//                 }
//             }
//             _ => return Err(ExitError::Other(Cow::from("ERR_INVALID_FLAG"))),
//         };

//         let promise = PromiseCreateArgs {
//             target_account_id: nep141_address,
//             method: "ft_transfer".to_string(),
//             args: args.as_bytes().to_vec(),
//             attached_balance: 1,
//             attached_gas: costs::FT_TRANSFER_GAS,
//         };

//         Ok(PrecompileOutput {
//             promise: Some(promise),
//             ..Default::default()
//         })
//     }
// }

// pub struct ExitToEthereum;

// impl ExitToEthereum {
//     /// Exit to Ethereum precompile address
//     ///
//     /// Address: `0xb0bd02f6a392af548bdf1cfaee5dfa0eefcc8eab`
//     /// This address is computed as: `&keccak("exitToEthereum")[12..]`
//     pub(crate) const ADDRESS: Address =
//         super::make_address(0xb0bd02f6, 0xa392af548bdf1cfaee5dfa0eefcc8eab);
// }

// impl Precompile for ExitToEthereum {
//     fn required_gas(_input: &[u8]) -> Result<u64, ExitError> {
//         Ok(costs::EXIT_TO_ETHEREUM_GAS)
//     }

//     #[cfg(not(feature = "contract"))]
//     fn run(
//         input: &[u8],
//         target_gas: u64,
//         _context: &CallContext,
//         _is_static: bool,
//     ) -> PrecompileResult {
//         if Self::required_gas(input)? > target_gas {
//             return Err(ExitError::OutOfGas);
//         }

//         Ok(PrecompileOutput {
//             output: Vec::new(),
//             cost: 0,
//             logs: Vec::new(),
//             promise: None,
//         })
//     }

//     #[cfg(feature = "contract")]
//     fn run(input: &[u8], target_gas: u64, context: &CallContext, is_static: bool) -> PrecompileResult {
//         if Self::required_gas(input)? > target_gas {
//             return Err(ExitError::OutOfGas);
//         }

//         // It's not allowed to call exit precompiles in static mode
//         if is_static {
//             return Err(ExitError::Other(Cow::from("ERR_INVALID_IN_STATIC")));
//         }

//         // First byte of the input is a flag, selecting the behavior to be triggered:
//         //      0x0 -> Eth transfer
//         //      0x1 -> Erc20 transfer
//         let mut input = input;
//         let flag = input[0];
//         input = &input[1..];

//         let (nep141_address, serialized_args) = match flag {
//             0x0 => {
//                 // ETH transfer
//                 //
//                 // Input slice format:
//                 //      eth_recipient (20 bytes) - the address of recipient which will receive ETH on Ethereum
//                 (
//                     String::from_utf8(crate::sdk::current_account_id()).unwrap(),
//                     // There is no way to inject json, given the encoding of both arguments
//                     // as decimal and hexadecimal respectively.
//                     WithdrawCallArgs {
//                         recipient_address: input.try_into().map_err(|_| {
//                             ExitError::Other(Cow::from("ERR_INVALID_RECIPIENT_ADDRESS"))
//                         })?,
//                         amount: context.apparent_value.as_u128(),
//                     }
//                     .try_to_vec()
//                     .map_err(|_| ExitError::Other(Cow::from("ERR_INVALID_AMOUNT")))?,
//                 )
//             }
//             0x1 => {
//                 // ERC-20 transfer
//                 //
//                 // This precompile branch is expected to be called from the ERC20 withdraw function
//                 // (or burn function with some flag provided that this is expected to be withdrawn)
//                 //
//                 // Input slice format:
//                 //      amount (U256 big-endian bytes) - the amount that was burned
//                 //      eth_recipient (20 bytes) - the address of recipient which will receive ETH on Ethereum

//                 if context.apparent_value != U256::from(0) {
//                     return Err(ExitError::Other(Cow::from(
//                         "ERR_ETH_ATTACHED_FOR_ERC20_EXIT",
//                     )));
//                 }

//                 let nep141_address = get_nep141_from_erc20(context.caller.as_bytes());

//                 let amount = U256::from_big_endian(&input[..32]).as_u128();
//                 input = &input[32..];

//                 if input.len() == 20 {
//                     // Parse ethereum address in hex
//                     let eth_recipient: String = hex::encode(input.to_vec());

//                     (
//                         nep141_address,
//                         // There is no way to inject json, given the encoding of both arguments
//                         // as decimal and hexadecimal respectively.
//                         crate::prelude::format!(
//                             r#"{{"amount": "{}", "recipient": "{}"}}"#,
//                             amount,
//                             eth_recipient
//                         )
//                         .as_bytes()
//                         .to_vec(),
//                     )
//                 } else {
//                     return Err(ExitError::Other(Cow::from("ERR_INVALID_RECIPIENT_ADDRESS")));
//                 }
//             }
//             _ => {
//                 return Err(ExitError::Other(Cow::from(
//                     "ERR_INVALID_RECEIVER_ACCOUNT_ID",
//                 )));
//             }
//         };

//         let promise = PromiseCreateArgs {
//             target_account_id: nep141_address,
//             method: "withdraw".to_string(),
//             args: serialized_args,
//             attached_balance: 1,
//             attached_gas: costs::WITHDRAWAL_GAS,
//         };

//         Ok(PrecompileOutput {
//             promise: Some(promise),
//             ..Default::default()
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::{ExitToEthereum, ExitToNear};
//     use crate::types::near_account_to_evm_address;

//     #[test]
//     fn test_precompile_id() {
//         assert_eq!(
//             ExitToEthereum::ADDRESS,
//             near_account_to_evm_address("exitToEthereum".as_bytes())
//         );
//         assert_eq!(
//             ExitToNear::ADDRESS,
//             near_account_to_evm_address("exitToNear".as_bytes())
//         );
//     }
// }
