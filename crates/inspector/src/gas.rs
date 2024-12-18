//! GasIspector. Helper Inspector to calculate gas for others.
use revm::interpreter::{CallOutcome, CreateOutcome, Gas};

/// Helper that keeps track of gas.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct GasInspector {
    gas_remaining: u64,
    last_gas_cost: u64,
}

impl Default for GasInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl GasInspector {
    pub fn gas_remaining(&self) -> u64 {
        self.gas_remaining
    }

    pub fn last_gas_cost(&self) -> u64 {
        self.last_gas_cost
    }

    pub fn new() -> Self {
        Self {
            gas_remaining: 0,
            last_gas_cost: 0,
        }
    }

    #[inline]
    pub fn initialize_interp(&mut self, gas: &Gas) {
        self.gas_remaining = gas.limit();
    }

    #[inline]
    pub fn step(&mut self, gas: &Gas) {
        self.gas_remaining = gas.remaining();
    }

    #[inline]
    pub fn step_end(&mut self, gas: &mut Gas) {
        let remaining = gas.remaining();
        self.last_gas_cost = self.gas_remaining.saturating_sub(remaining);
        self.gas_remaining = remaining;
    }

    #[inline]
    pub fn call_end(&mut self, outcome: &mut CallOutcome) {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
    }

    #[inline]
    pub fn create_end(&mut self, outcome: &mut CreateOutcome) {
        if outcome.result.result.is_error() {
            outcome.result.gas.spend_all();
            self.gas_remaining = 0;
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::inspector_handle_register;
//     use database::BenchmarkDB;
//     use revm::{
//         bytecode::{opcode, Bytecode},
//         context_interface::EvmWiring as PrimitiveEvmWiring,
//         context_interface::{DefaultEthereumWiring, EthereumWiring},
//         interpreter::Interpreter,
//         primitives::{address, Bytes, Log, TxKind},
//         Evm, EvmWiring,
//     };

//     type TestEvmWiring = DefaultEthereumWiring;

//     #[derive(Default, Debug)]
//     struct StackInspector {
//         pc: usize,
//         gas_inspector: GasInspector,
//         gas_remaining_steps: Vec<(usize, u64)>,
//     }

//     impl<EvmWiringT: EvmWiring> Inspector<EvmWiringT> for StackInspector {
//         fn initialize_interp(
//             &mut self,
//             interp: &mut Interpreter,
//             context: &mut EvmContext<EvmWiringT>,
//         ) {
//             self.gas_inspector.initialize_interp(interp, context);
//         }

//         fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
//             self.pc = interp.program_counter();
//             self.gas_inspector.step(interp, context);
//         }

//         fn log(
//             &mut self,
//             interp: &mut Interpreter,
//             context: &mut EvmContext<EvmWiringT>,
//             log: &Log,
//         ) {
//             self.gas_inspector.log(interp, context, log);
//         }

//         fn step_end(&mut self, interp: &mut Interpreter, context: &mut EvmContext<EvmWiringT>) {
//             self.gas_inspector.step_end(interp, context);
//             self.gas_remaining_steps
//                 .push((self.pc, self.gas_inspector.gas_remaining()));
//         }

//         fn call(
//             &mut self,
//             context: &mut EvmContext<EvmWiringT>,
//             call: &mut CallInputs,
//         ) -> Option<CallOutcome> {
//             self.gas_inspector.call(context, call)
//         }

//         fn call_end(
//             &mut self,
//             context: &mut EvmContext<EvmWiringT>,
//             inputs: &CallInputs,
//             outcome: CallOutcome,
//         ) -> CallOutcome {
//             self.gas_inspector.call_end(context, inputs, outcome)
//         }

//         fn create(
//             &mut self,
//             context: &mut EvmContext<EvmWiringT>,
//             call: &mut CreateInputs,
//         ) -> Option<CreateOutcome> {
//             self.gas_inspector.create(context, call);
//             None
//         }

//         fn create_end(
//             &mut self,
//             context: &mut EvmContext<EvmWiringT>,
//             inputs: &CreateInputs,
//             outcome: CreateOutcome,
//         ) -> CreateOutcome {
//             self.gas_inspector.create_end(context, inputs, outcome)
//         }
//     }

//     #[test]
//     fn test_gas_inspector() {
//         let contract_data: Bytes = Bytes::from(vec![
//             opcode::PUSH1,
//             0x1,
//             opcode::PUSH1,
//             0xb,
//             opcode::JUMPI,
//             opcode::PUSH1,
//             0x1,
//             opcode::PUSH1,
//             0x1,
//             opcode::PUSH1,
//             0x1,
//             opcode::JUMPDEST,
//             opcode::STOP,
//         ]);
//         let bytecode = Bytecode::new_raw(contract_data);

//         let mut evm = Evm::<EthereumWiring<BenchmarkDB, StackInspector>>::builder()
//             .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
//             .with_default_ext_context()
//             .modify_tx_env(|tx| {
//                 *tx = <TestEvmWiring as PrimitiveEvmWiring>::Transaction::default();

//                 tx.caller = address!("1000000000000000000000000000000000000000");
//                 tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
//                 tx.gas_limit = 21100;
//             })
//             .append_handler_register(inspector_handle_register)
//             .build();

//         // run evm.
//         evm.transact().unwrap();

//         let inspector = evm.into_context().external;

//         // starting from 100gas
//         let steps = vec![
//             // push1 -3
//             (0, 97),
//             // push1 -3
//             (2, 94),
//             // jumpi -10
//             (4, 84),
//             // jumpdest 1
//             (11, 83),
//             // stop 0
//             (12, 83),
//         ];

//         assert_eq!(inspector.gas_remaining_steps, steps);
//     }
// }
