use context_interface::CfgGetter;
use handler_interface::PrecompileProvider;
use interpreter::{Gas, InstructionResult, InterpreterResult};
use precompile::PrecompileErrors;
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{Address, Bytes};
use specification::hardfork::SpecId;
use std::boxed::Box;

pub struct EthPrecompileProvider<CTX, ERROR> {
    pub precompiles: &'static Precompiles,
    pub _phantom: core::marker::PhantomData<(CTX, ERROR)>,
}

impl<CTX, ERROR> Clone for EthPrecompileProvider<CTX, ERROR> {
    fn clone(&self) -> Self {
        Self {
            precompiles: self.precompiles,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR> EthPrecompileProvider<CTX, ERROR> {
    pub fn new(spec: SpecId) -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR> PrecompileProvider for EthPrecompileProvider<CTX, ERROR>
where
    CTX: CfgGetter,
    ERROR: From<PrecompileErrors>,
{
    type Context = CTX;
    type Error = ERROR;
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: SpecId) {
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec));
    }

    fn run(
        &mut self,
        _context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<InterpreterResult>, Self::Error> {
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
            Err(err @ PrecompileErrors::Fatal { .. }) => return Err(err.into()),
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
