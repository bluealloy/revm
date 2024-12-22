use context_interface::{Cfg, CfgGetter};
use handler_interface::PrecompileProvider;
use interpreter::{Gas, InstructionResult, InterpreterResult};
use precompile::PrecompileErrors;
use precompile::{PrecompileSpecId, Precompiles};
use primitives::{Address, Bytes};

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

impl<CTX, ERROR> PrecompileProvider for EthPrecompileProvider<CTX, ERROR>
where
    CTX: CfgGetter,
    ERROR: From<PrecompileErrors>,
{
    type Context = CTX;
    type Error = ERROR;

    fn new(context: &mut Self::Context) -> Self {
        let spec = context.cfg().spec().into();
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            _phantom: core::marker::PhantomData,
        }
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
                let success = result.gas.record_cost(output.gas_used);
                assert!(success, "Gas underflow is not possible");
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

    fn warm_addresses(&self) -> impl Iterator<Item = Address> {
        self.precompiles.addresses().cloned()
    }

    fn contains(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}
