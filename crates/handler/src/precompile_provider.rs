use auto_impl::auto_impl;
use context::Cfg;
use context_interface::ContextTr;
use interpreter::{Gas, InstructionResult, InterpreterResult};
use precompile::PrecompileError;
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{hardfork::SpecId, Address, Bytes};
use std::boxed::Box;

#[auto_impl(&mut, Box)]
pub trait PrecompileProvider {
    type Context: ContextTr;
    type Output;

    fn set_spec(&mut self, spec: <<Self::Context as ContextTr>::Cfg as Cfg>::Spec);

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, PrecompileError>;

    /// Get the warm addresses.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address> + '_>;

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool;
}

pub struct EthPrecompiles<CTX> {
    pub precompiles: &'static Precompiles,
    pub _phantom: core::marker::PhantomData<CTX>,
}

impl<CTX> Clone for EthPrecompiles<CTX> {
    fn clone(&self) -> Self {
        Self {
            precompiles: self.precompiles,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX> Default for EthPrecompiles<CTX> {
    fn default() -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(SpecId::LATEST)),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX> PrecompileProvider for EthPrecompiles<CTX>
where
    CTX: ContextTr,
{
    type Context = CTX;
    type Output = InterpreterResult;
    fn set_spec(&mut self, spec: <<Self::Context as ContextTr>::Cfg as Cfg>::Spec) {
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec.into()));
    }

    fn run(
        &mut self,
        _context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, PrecompileError> {
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
            Err(e) => {
                if let PrecompileError::Fatal(_) = e {
                    return Err(e);
                }
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
        Box::new(self.precompiles.addresses().cloned())
    }

    fn contains(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}
