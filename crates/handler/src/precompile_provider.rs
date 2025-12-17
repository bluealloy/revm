use auto_impl::auto_impl;
use context::{Cfg, LocalContextTr, SetSpecTr};
use context_interface::{ContextTr, JournalTr};
use interpreter::{CallInput, CallInputs, Gas, InstructionResult, InterpreterResult};
use precompile::{PrecompileError, PrecompileSpecId, Precompiles};
use primitives::{hardfork::SpecId, Address, Bytes};
use std::{
    boxed::Box,
    string::{String, ToString},
};

/// Provider for precompiled contracts in the EVM.
#[auto_impl(&mut, Box)]
pub trait PrecompileProvider<CTX: ContextTr> {
    /// The output type returned by precompile execution.
    type Output;

    /// Sets the spec id and returns true if the spec id was changed. Initial call to set_spec will always return true.
    ///
    /// Returns `true` if precompile addresses should be injected into the journal.
    #[deprecated(
        note = "We are moving away from runtime setting off spec to setting spec in initialization. Check EvmTrSetSpec trait for more information."
    )]
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool;

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
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
    /// Spec override function.
    pub spec_override_fn: Option<fn(spec: SpecId) -> &'static Precompiles>,
}

impl EthPrecompiles {
    /// Create a new precompile provider with the given spec.
    pub fn new(spec: SpecId) -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            spec,
            spec_override_fn: None,
        }
    }

    /// Returns addresses of the precompiles.
    pub fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        Box::new(self.precompiles.addresses().cloned())
    }

    /// Returns whether the address is a precompile.
    pub fn contains(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}

impl<SPEC: Into<SpecId> + Clone> SetSpecTr<SPEC> for EthPrecompiles {
    fn set_spec(&mut self, spec: SPEC) {
        let spec = spec.into();
        if spec == self.spec {
            return;
        }
        self.precompiles = self
            .spec_override_fn
            .map(|override_fn| override_fn(spec))
            .unwrap_or_else(|| Precompiles::new(PrecompileSpecId::from_spec_id(spec)));
        self.spec = spec;
    }
}

impl Clone for EthPrecompiles {
    fn clone(&self) -> Self {
        Self {
            precompiles: self.precompiles,
            spec: self.spec,
            spec_override_fn: self.spec_override_fn,
        }
    }
}

impl Default for EthPrecompiles {
    fn default() -> Self {
        let spec = SpecId::default();
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            spec,
            spec_override_fn: None,
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
        self.precompiles = self
            .spec_override_fn
            .map(|override_fn| override_fn(spec))
            .unwrap_or_else(|| Precompiles::new(PrecompileSpecId::from_spec_id(spec)));
        self.spec = spec;
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<InterpreterResult>, String> {
        let Some(precompile) = self.precompiles.get(&inputs.bytecode_address) else {
            return Ok(None);
        };

        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas: Gas::new(inputs.gas_limit),
            output: Bytes::new(),
        };

        let exec_result = {
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
            precompile.execute(input_bytes, inputs.gas_limit)
        };

        match exec_result {
            Ok(output) => {
                result.gas.record_refund(output.gas_refunded);
                let underflow = result.gas.record_cost(output.gas_used);
                assert!(underflow, "Gas underflow is not possible");
                result.result = if output.reverted {
                    InstructionResult::Revert
                } else {
                    InstructionResult::Return
                };
                result.output = output.bytes;
            }
            Err(PrecompileError::Fatal(e)) => return Err(e),
            Err(e) => {
                result.result = if e.is_oog() {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
                // If this is a top-level precompile call (depth == 1), persist the error message
                // into the local context so it can be returned as output in the final result.
                // Only do this for non-OOG errors (OOG is a distinct halt reason without output).
                if !e.is_oog() && context.journal().depth() == 1 {
                    context
                        .local_mut()
                        .set_precompile_error_context(e.to_string());
                }
            }
        }
        Ok(Some(result))
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        Self::warm_addresses(self)
    }

    fn contains(&self, address: &Address) -> bool {
        Self::contains(self, address)
    }
}
