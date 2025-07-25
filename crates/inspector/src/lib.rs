//! Inspector is a crate that provides a set of traits that allow inspecting the EVM execution.
//!
//! It is used to implement tracers that can be used to inspect the EVM execution.
//! Implementing inspection is optional and it does not effect the core execution.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

mod count_inspector;
#[cfg(feature = "tracer")]
mod eip3155;
mod either;
mod gas;
/// Handler implementations for inspector integration.
pub mod handler;
mod inspect;
mod inspector;
mod mainnet_inspect;
mod noop;
mod traits;

#[cfg(test)]
mod inspector_tests;

/// Inspector implementations.
pub mod inspectors {
    #[cfg(feature = "tracer")]
    pub use super::eip3155::TracerEip3155;
    pub use super::gas::GasInspector;
}

pub use count_inspector::CountInspector;
pub use handler::{inspect_instructions, InspectorHandler};
pub use inspect::{InspectCommitEvm, InspectEvm};
pub use inspector::*;
pub use noop::NoOpInspector;
pub use traits::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ::handler::{MainBuilder, MainContext};
    use context::{BlockEnv, CfgEnv, Context, Journal, TxEnv};
    use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
    use interpreter::{interpreter::EthInterpreter, InstructionResult, InterpreterTypes};
    use primitives::TxKind;
    use state::{bytecode::opcode, Bytecode};

    struct HaltInspector;
    impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for HaltInspector {
        fn step(&mut self, interp: &mut interpreter::Interpreter<INTR>, _context: &mut CTX) {
            interp.halt(InstructionResult::Stop);
        }
    }

    #[test]
    fn test_step_halt() {
        let bytecode = [opcode::INVALID];
        let r = run(&bytecode, HaltInspector);
        dbg!(&r);
        assert!(r.is_success());
    }

    fn run(
        bytecode: &[u8],
        inspector: impl Inspector<
            Context<BlockEnv, TxEnv, CfgEnv, BenchmarkDB, Journal<BenchmarkDB>, ()>,
            EthInterpreter,
        >,
    ) -> context::result::ExecutionResult {
        let bytecode = Bytecode::new_raw(bytecode.to_vec().into());
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(inspector);
        evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(21100)
                .build()
                .unwrap(),
        )
        .unwrap()
    }
}
