use auto_impl::auto_impl;
use context::Cfg;
use context_interface::ContextTr;
use interpreter::{Gas, InstructionResult, InterpreterResult};
use precompile::PrecompileError;
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{hardfork::SpecId, Address, Bytes};
use std::boxed::Box;
use std::string::String;

#[auto_impl(&mut, Box)]
pub trait PrecompileProvider<CTX: ContextTr> {
    type Output;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec);

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        bytes: &Bytes,
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
    pub precompiles: &'static Precompiles,
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
        }
    }
}

impl Default for EthPrecompiles {
    fn default() -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(SpecId::default())),
        }
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for EthPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) {
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec.into()));
    }

    fn run(
        &mut self,
        _context: &mut CTX,
        address: &Address,
        bytes: &Bytes,
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

        match (*precompile)(bytes, gas_limit) {
            Ok(output) => {
                let underflow = result.gas.record_cost(output.gas_used);
                assert!(underflow, "Gas underflow is not possible");
                result.result = InstructionResult::Return;
                result.output = output.bytes;
            }
            Err(PrecompileError::Fatal(e)) => return Err(e),
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
