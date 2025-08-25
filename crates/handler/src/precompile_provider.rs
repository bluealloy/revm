use auto_impl::auto_impl;
use context::{Cfg, LocalContextTr};
use context_interface::ContextTr;
use interpreter::{CallInput, Gas, InputsImpl, InstructionResult, InterpreterResult};
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{hardfork::SpecId, Address, Bytes};
use std::boxed::Box;
use std::string::String;

/// Provider for precompiled contracts in the EVM.
#[auto_impl(&mut, Box)]
pub trait PrecompileProvider<CTX: ContextTr> {
    /// The output type returned by precompile execution.
    type Output;

    /// Sets the spec id and returns true if the spec id was changed. Initial call to set_spec will always return true.
    ///
    /// Returned booling will determine if precompile addresses should be injected into the journal.
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool;

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String>;

    /// Get the warm addresses.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>>;

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool;
}

/// The [`PrecompileProvider`] for ethereum precompiles.
#[derive(Debug)]
pub struct EthPrecompiles {
    /// Contains precompiles for the current spec.
    pub precompiles: &'static Precompiles,
    /// Current spec. None means that spec was not set yet.
    pub spec: SpecId,
}

impl EthPrecompiles {
    /// Returns addresses of the precompiles.
    pub fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        Box::new(self.precompiles.addresses().cloned())
    }

    /// Returns whether the address is a precompile.
    pub fn contains(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}

impl Clone for EthPrecompiles {
    fn clone(&self) -> Self {
        Self {
            precompiles: self.precompiles,
            spec: self.spec,
        }
    }
}

impl Default for EthPrecompiles {
    fn default() -> Self {
        let spec = SpecId::default();
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            spec,
        }
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for EthPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        let spec = spec.into();
        // generate new precompiles only on new spec
        if spec == self.spec {
            return false;
        }
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec));
        self.spec = spec;
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        _is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, String> {
        let Some(precompile) = self.precompiles.get(address) else {
            return Ok(None);
        };

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas: Gas::new(gas_limit),
            output: Bytes::new(),
        };

        let r;
        let input_bytes = match &inputs.input {
            CallInput::SharedBuffer(range) => {
                if let Some(slice) = context.local().shared_memory_buffer_slice(range.clone()) {
                    r = slice;
                    r.as_ref()
                } else {
                    &[]
                }
            }
            CallInput::Bytes(bytes) => bytes.0.iter().as_slice(),
        };

        match precompile.execute(input_bytes, gas_limit) {
            Ok(output) => {
                let underflow = result.gas.record_cost(output.gas_used);
                assert!(underflow, "Gas underflow is not possible");
                result.result = if output.reverted {
                    InstructionResult::Revert
                } else {
                    InstructionResult::Return
                };
                result.output = output.bytes;
            }
            Err(e) => {
                result.result = if e.is_oog() {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
        }
        Ok(Some(result))
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        self.warm_addresses()
    }

    fn contains(&self, address: &Address) -> bool {
        self.contains(address)
    }
}
