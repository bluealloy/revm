use crate::{cfg::CfgExt, context::GwynethContextTr};
use revm::{
    context::{ContextTr, Transaction},
    precompile::{u64_to_address, PrecompileError, PrecompileOutput, PrecompileResult},
    primitives::{Address, Bytes, FixedBytes, B256},
};

/// The address of the xcall precompile
pub const XCALL_ADDRESS: Address = u64_to_address(1234);

const XCALL_INVALID_INPUT_LENGTH: &str = "XCallInvalidInputLength";
const XCALL_INVALID_VERSION: &str = "XCallInvalidVersion";
const XCALL_INVALID_ORIGIN: &str = "XCallInvalidOrigin";

/// The options for the xcall precompile
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XCallOptions {
    /// The target chain id
    pub chain_id: u64,
    /// If the call needs to persist state changes or not
    pub sandbox: bool,
    /// Mocked `tx.origin`
    pub tx_origin: Address,
    /// Mocked `msg.sender`
    pub msg_sender: Address,
    /// The block hash to execute against (None will execute against the latest known blockhash)
    pub block_hash: Option<B256>,
    /// The data necessary to execute the call
    pub proof: Vec<u8>,
}

/// Run the xcall precompile.
pub fn run_xcall<CTX: GwynethContextTr>(
    input: &[u8],
    _gas_limit: u64,
    ctx: &CTX,
    caller: Address,
) -> PrecompileResult {
    println!("  xcalloptions_run: {}, {:?}", input.len(), input);

    // Verify input length.
    if input.len() < 83 {
        return Err(PrecompileError::other(XCALL_INVALID_INPUT_LENGTH));
    }

    // Read the input data
    let version = u16::from_be_bytes(input[0..2].try_into().unwrap());
    let chain_id = u64::from_be_bytes(input[2..10].try_into().unwrap());
    let sandbox = input[10] != 0;
    let tx_origin: Address = (&input[11..31], chain_id).try_into().unwrap();
    let msg_sender: Address = (&input[31..51], caller.chain_id()).try_into().unwrap();
    let block_hash: Option<FixedBytes<32>> = Some(input[51..83].try_into().unwrap());
    let proof = &input[83..];

    // Check the version
    if version != 1 {
        return Err(PrecompileError::other(XCALL_INVALID_VERSION));
    }

    if !sandbox && !ctx.cfg().allow_mocking() {
        // env.tx.caller is the Signer of the transaction
        // caller is the address of the contract that is calling the precompile
        if tx_origin != ctx.tx().caller() || msg_sender != caller {
            println!(
                "  tx_origin: {:?}, tx.caller: {:?}, msg_sender: {:?}, caller: {:?}",
                tx_origin,
                ctx.tx().caller(),
                msg_sender,
                caller
            );
            return Err(PrecompileError::other(XCALL_INVALID_ORIGIN));
        }
    }

    // Set the call options
    *ctx.chain().xcall_options = Some(XCallOptions {
        chain_id,
        sandbox,
        tx_origin,
        msg_sender,
        block_hash,
        proof: proof.to_vec(),
    });
    println!("  CallOptions: {:?}", xcall_options);

    Ok(PrecompileOutput::new(
        0,
        Bytes::from_static(&[0x6c, 0x54, 0x13, 0x30]),
    ))
}
