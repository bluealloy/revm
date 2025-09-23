use crate::{
    inspect::{InspectCommitEvm, InspectEvm, InspectSystemCallEvm},
    Inspector, InspectorEvmTr, InspectorHandler, JournalExt,
};
use context::{ContextSetters, ContextTr, Evm, FrameStack, JournalTr};
use database_interface::DatabaseCommit;
use handler::{
    instructions::InstructionProvider, system_call::SystemCallTx, EthFrame, EvmTr, EvmTrError,
    Handler, MainnetHandler, PrecompileProvider,
};
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use primitives::{Address, Bytes};
use state::EvmState;

// Implementing InspectorHandler for MainnetHandler.
impl<EVM, ERROR> InspectorHandler for MainnetHandler<EVM, ERROR, EthFrame<EthInterpreter>>
where
    EVM: InspectorEvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Frame = EthFrame<EthInterpreter>,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM>,
{
    type IT = EthInterpreter;
}

// Implementing InspectEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextSetters + ContextTr<Journal: JournalTr<State = EvmState> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.inspector = inspector;
    }

    fn inspect_one_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_tx(tx);
        MainnetHandler::default().inspect_run(self)
    }
}

// Implementing InspectCommitEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectCommitEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextSetters
        + ContextTr<Journal: JournalTr<State = EvmState> + JournalExt, Db: DatabaseCommit>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
}

// Implementing InspectSystemCallEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectSystemCallEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextSetters
        + ContextTr<Journal: JournalTr<State = EvmState> + JournalExt, Tx: SystemCallTx>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn inspect_one_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        // Set system call transaction fields similar to transact_system_call_with_caller
        self.set_tx(CTX::Tx::new_system_tx_with_caller(
            caller,
            system_contract_address,
            data,
        ));
        // Use inspect_run_system_call instead of run_system_call for inspection
        MainnetHandler::default().inspect_run_system_call(self)
    }
}

// Implementing InspectorEvmTr for Evm
impl<CTX, INSP, I, P> InspectorEvmTr for Evm<CTX, INSP, I, P, EthFrame<EthInterpreter>>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &Self::Inspector,
        &Self::Instructions,
        &FrameStack<Self::Frame>,
    ) {
        let ctx = &self.ctx;
        let inspector = &self.inspector;
        let frame = &self.frame_stack;
        let instructions = &self.instruction;
        (ctx, inspector, instructions, frame)
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut FrameStack<Self::Frame>,
        &mut Self::Instructions,
    ) {
        let ctx = &mut self.ctx;
        let inspector = &mut self.inspector;
        let frame = &mut self.frame_stack;
        let instructions = &mut self.instruction;
        (ctx, inspector, frame, instructions)
    }
}
