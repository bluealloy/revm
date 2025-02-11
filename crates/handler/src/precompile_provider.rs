use auto_impl::auto_impl;
use context::Cfg;
use context_interface::ContextTrait;
use interpreter::{Gas, InstructionResult, InterpreterResult};
use precompile::PrecompileErrors;
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{Address, Bytes};
use specification::hardfork::SpecId;
use std::boxed::Box;

#[auto_impl(&mut, Box)]
pub trait PrecompileProvider {
    type Context: ContextTrait;
    type Output;

    fn set_spec(&mut self, spec: <<Self::Context as ContextTrait>::Cfg as Cfg>::Spec);

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, PrecompileErrors>;

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

impl<CTX> Default for EthPrecompiles<CTX>
where
    CTX: ContextTrait,
{
    fn default() -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(SpecId::LATEST)),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX> PrecompileProvider for EthPrecompiles<CTX>
where
    CTX: ContextTrait,
{
    type Context = CTX;
    type Output = InterpreterResult;
    fn set_spec(&mut self, spec: <<Self::Context as ContextTrait>::Cfg as Cfg>::Spec) {
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec.into()));
    }

    fn run(
        &mut self,
        _context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, PrecompileErrors> {
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
            Err(PrecompileErrors::Error(e)) => {
                result.result = if e.is_oog() {
                    InstructionResult::PrecompileOOG
                } else {
                    InstructionResult::PrecompileError
                };
            }
            Err(err @ PrecompileErrors::Fatal { .. }) => return Err(err),
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
