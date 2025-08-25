//! Custom precompile provider implementation.

use revm::{
    context::Cfg,
    context_interface::{ContextTr, JournalTr, LocalContextTr, Transaction},
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{Gas, InputsImpl, InstructionResult, InterpreterResult},
    precompile::{PrecompileError, PrecompileOutput, PrecompileResult},
    primitives::{address, hardfork::SpecId, Address, Bytes, U256},
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
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        // Check if this is our custom precompile
        if *address == CUSTOM_PRECOMPILE_ADDRESS {
            return Ok(Some(run_custom_precompile(
                context, inputs, is_static, gas_limit,
            )?));
        }

        // Otherwise, delegate to standard Ethereum precompiles
        self.inner
            .run(context, address, inputs, is_static, gas_limit)
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
    inputs: &InputsImpl,
    is_static: bool,
    gas_limit: u64,
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
        handle_read_storage(context, gas_limit)
    } else if input_bytes.len() == 32 {
        if is_static {
            return Err("Cannot modify state in static context".to_string());
        }
        // Write storage operation
        handle_write_storage(context, &input_bytes, gas_limit)
    } else {
        Err(PrecompileError::InputLength)
    };

    match result {
        Ok(output) => {
            let mut interpreter_result = InterpreterResult {
                result: if output.reverted {
                    InstructionResult::Revert
                } else {
                    InstructionResult::Return
                },
                gas: Gas::new(gas_limit),
                output: output.bytes,
            };
            let underflow = interpreter_result.gas.record_cost(output.gas_used);
            if !underflow {
                interpreter_result.result = InstructionResult::PrecompileOOG;
            }
            Ok(interpreter_result)
        }
        Err(e) => Ok(InterpreterResult {
            result: if e.is_oog() {
                InstructionResult::PrecompileOOG
            } else {
                InstructionResult::PrecompileError(e)
            },
            gas: Gas::new(gas_limit),
            output: Bytes::new(),
        }),
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
        .map_err(|_| PrecompileError::StorageOperationFailed)?
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
        .map_err(|_| PrecompileError::StorageOperationFailed)?;

    // Get the caller address
    let caller = context.tx().caller();

    // Transfer 1 wei from the precompile to the caller as a reward
    // First, ensure the precompile has balance
    context
        .journal_mut()
        .balance_incr(CUSTOM_PRECOMPILE_ADDRESS, U256::from(1))
        .map_err(|_| PrecompileError::BalanceOperationFailed)?;

    // Then transfer to caller
    let transfer_result = context
        .journal_mut()
        .transfer(CUSTOM_PRECOMPILE_ADDRESS, caller, U256::from(1))
        .map_err(|_| PrecompileError::TransferFailed)?;

    if let Some(_error) = transfer_result {
        return Err(PrecompileError::TransferFailed);
    }

    // Return success with empty output
    Ok(PrecompileOutput::new(BASE_GAS + SSTORE_GAS, Bytes::new()))
}
