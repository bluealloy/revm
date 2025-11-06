//! Custom precompile provider implementation.

use revm::{
    context::Cfg,
    context_interface::{ContextTr, JournalTr, LocalContextTr, Transaction},
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{CallInputs, Gas, InstructionResult, InterpreterResult},
    precompile::{PrecompileError, PrecompileOutput, PrecompileResult},
    primitives::{address, hardfork::SpecId, Address, Bytes, Log, B256, U256},
};
use std::boxed::Box;
use std::string::String;

// Define our custom precompile address
pub const CUSTOM_PRECOMPILE_ADDRESS: Address = address!("0000000000000000000000000000000000000100");

// Custom storage key for our example
const STORAGE_KEY: U256 = U256::ZERO;

/// Custom precompile provider that includes journal access functionality
#[derive(Debug, Clone)]
pub struct CustomPrecompileProvider {
    inner: EthPrecompiles,
    spec: SpecId,
}

impl CustomPrecompileProvider {
    pub fn new_with_spec(spec: SpecId) -> Self {
        Self {
            inner: EthPrecompiles::default(),
            spec,
        }
    }
}

impl<CTX> PrecompileProvider<CTX> for CustomPrecompileProvider
where
    CTX: ContextTr<Cfg: Cfg<Spec = SpecId>>,
{
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        if spec == self.spec {
            return false;
        }
        self.spec = spec;
        // Create a new inner provider with the new spec
        self.inner = EthPrecompiles::default();
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<Self::Output>, String> {
        // Check if this is our custom precompile
        if inputs.bytecode_address == CUSTOM_PRECOMPILE_ADDRESS {
            return Ok(Some(run_custom_precompile(context, inputs)?));
        }

        // Otherwise, delegate to standard Ethereum precompiles
        self.inner.run(context, inputs)
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        // Include our custom precompile address along with standard ones
        let mut addresses = vec![CUSTOM_PRECOMPILE_ADDRESS];
        addresses.extend(self.inner.warm_addresses());
        Box::new(addresses.into_iter())
    }

    fn contains(&self, address: &Address) -> bool {
        *address == CUSTOM_PRECOMPILE_ADDRESS || self.inner.contains(address)
    }
}

/// Runs our custom precompile
fn run_custom_precompile<CTX: ContextTr>(
    context: &mut CTX,
    inputs: &CallInputs,
) -> Result<InterpreterResult, String> {
    let input_bytes = match &inputs.input {
        revm::interpreter::CallInput::SharedBuffer(range) => {
            if let Some(slice) = context.local().shared_memory_buffer_slice(range.clone()) {
                slice.to_vec()
            } else {
                vec![]
            }
        }
        revm::interpreter::CallInput::Bytes(bytes) => bytes.0.to_vec(),
    };

    // For this example, we'll implement a simple precompile that:
    // - If called with empty data: reads a storage value
    // - If called with 32 bytes: writes that value to storage and transfers 1 wei to the caller

    let result = if input_bytes.is_empty() {
        // Read storage operation
        handle_read_storage(context, inputs.gas_limit)
    } else if input_bytes.len() == 32 {
        if inputs.is_static {
            return Err("Cannot modify state in static context".to_string());
        }
        // Write storage operation
        handle_write_storage(context, &input_bytes, inputs.gas_limit)
    } else {
        Err(PrecompileError::Other("Invalid input length".to_string()))
    };

    match result {
        Ok(output) => {
            let mut interpreter_result = InterpreterResult {
                result: if output.reverted {
                    InstructionResult::Revert
                } else {
                    InstructionResult::Return
                },
                gas: Gas::new(inputs.gas_limit),
                output: output.bytes,
            };
            let underflow = interpreter_result.gas.record_cost(output.gas_used);
            if !underflow {
                interpreter_result.result = InstructionResult::PrecompileOOG;
            }
            Ok(interpreter_result)
        }
        Err(e) => {
            // If this is a top-level precompile call and error is non-OOG, record the message
            if !e.is_oog() && context.journal().depth() == 1 {
                context
                    .local_mut()
                    .set_precompile_error_context(e.to_string());
            }
            Ok(InterpreterResult {
                result: if e.is_oog() {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                },
                gas: Gas::new(inputs.gas_limit),
                output: Bytes::new(),
            })
        }
    }
}

/// Handles reading from storage
fn handle_read_storage<CTX: ContextTr>(context: &mut CTX, gas_limit: u64) -> PrecompileResult {
    // Base gas cost for reading storage
    const BASE_GAS: u64 = 2_100;

    if gas_limit < BASE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    // Read from storage using the journal
    let value = context
        .journal_mut()
        .sload(CUSTOM_PRECOMPILE_ADDRESS, STORAGE_KEY)
        .map_err(|e| PrecompileError::Other(format!("Storage read failed: {e:?}")))?
        .data;

    // Return the value as output
    Ok(PrecompileOutput::new(
        BASE_GAS,
        value.to_be_bytes_vec().into(),
    ))
}

/// Handles writing to storage and transferring balance
fn handle_write_storage<CTX: ContextTr>(
    context: &mut CTX,
    input: &[u8],
    gas_limit: u64,
) -> PrecompileResult {
    // Base gas cost for the operation
    const BASE_GAS: u64 = 21_000;
    const SSTORE_GAS: u64 = 20_000;

    if gas_limit < BASE_GAS + SSTORE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse the input as a U256 value
    let value = U256::from_be_slice(input);

    // Store the value in the precompile's storage
    context
        .journal_mut()
        .sstore(CUSTOM_PRECOMPILE_ADDRESS, STORAGE_KEY, value)
        .map_err(|e| PrecompileError::Other(format!("Storage write failed: {e:?}")))?;

    // Get the caller address
    let caller = context.tx().caller();

    // Transfer 1 wei from the precompile to the caller as a reward
    // First, ensure the precompile has balance
    context
        .journal_mut()
        .balance_incr(CUSTOM_PRECOMPILE_ADDRESS, U256::from(1))
        .map_err(|e| PrecompileError::Other(format!("Balance increment failed: {e:?}")))?;

    // Then transfer to caller
    let transfer_result = context
        .journal_mut()
        .transfer(CUSTOM_PRECOMPILE_ADDRESS, caller, U256::from(1))
        .map_err(|e| PrecompileError::Other(format!("Transfer failed: {e:?}")))?;

    if let Some(error) = transfer_result {
        return Err(PrecompileError::Other(format!("Transfer error: {error:?}")));
    }

    // Create a log to record the storage write operation
    // Topic 0: keccak256("StorageWritten(address,uint256)")
    let topic0 = B256::from_slice(&[
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
        0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
        0xde, 0xf0,
    ]);
    // Topic 1: caller address (indexed) - left-padded to 32 bytes
    let mut topic1_bytes = [0u8; 32];
    topic1_bytes[12..32].copy_from_slice(caller.as_slice());
    let topic1 = B256::from(topic1_bytes);
    // Data: the value that was written
    let log_data = value.to_be_bytes_vec();

    let log = Log::new(
        CUSTOM_PRECOMPILE_ADDRESS,
        vec![topic0, topic1],
        log_data.into(),
    )
    .expect("Failed to create log");

    context.journal_mut().log(log);

    // Return success with empty output
    Ok(PrecompileOutput::new(BASE_GAS + SSTORE_GAS, Bytes::new()))
}
