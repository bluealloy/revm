use crate::{
    instructions::InstructionProvider, EthFrame, ExecuteCommitEvm, ExecuteEvm, Handler,
    MainnetHandler, PrecompileProvider,
};
use context::{ContextSetters, ContextTr, Evm, JournalOutput, JournalTr, TxEnv};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use primitives::{address, Address, Bytes, TxKind};

pub const SYSTEM_ADDRESS: Address = address!("0xfffffffffffffffffffffffffffffffffffffffe");

/// Creates the system transaction with default values and set data and tx call target to system contract address
/// that is going to be called.
///
/// The caller is set to be [`SYSTEM_ADDRESS`].
///
/// It is used inside [`SystemCallEvm`] and [`SystemCallCommitEvm`] traits to prepare EVM for system call execution.
pub trait SystemCallTx {
    /// Created new tranaction for system call.
    fn new_system_tx(data: Bytes, system_contract_address: Address) -> Self;
}

impl SystemCallTx for TxEnv {
    fn new_system_tx(data: Bytes, system_contract_address: Address) -> Self {
        let mut tx = TxEnv::default();
        tx.data = data;
        tx.kind = TxKind::Call(system_contract_address);
        tx.gas_limit = 30_000_000;
        tx.chain_id = None;
        tx
    }
}

/// API for executing the system calls. System calls dont deduct the caller or reward the
/// beneficiary. They are used before and after block execution to insert or obtain blockchain state.
///
/// It act similari to `transact` function and sets default Tx with data and system contract as a target.
pub trait SystemCallEvm: ExecuteEvm {
    /// System call is a special transaction call that is used to call a system contract.
    ///
    /// Transaction fields are reset and set in [`SystemCallTx`] and data and target are set to
    /// given values.
    fn transact_system_call(
        &mut self,
        data: Bytes,
        system_contract_address: Address,
    ) -> Self::Output;
}

/// Extension of the [`SystemCallEvm`] trait that adds a method that commits the state after execution.
pub trait SystemCallCommitEvm: SystemCallEvm + ExecuteCommitEvm {
    /// Transact the system call and commit to the state.
    fn transact_system_call_commit(
        &mut self,
        data: Bytes,
        system_contract_address: Address,
    ) -> Self::CommitOutput;
}

impl<CTX, INSP, INST, PRECOMPILES> SystemCallEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>, Tx: SystemCallTx>
        + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call(
        &mut self,
        data: Bytes,
        system_contract_address: Address,
    ) -> Self::Output {
        // set tx fields.
        self.set_tx(CTX::Tx::new_system_tx(data, system_contract_address));
        // create handler
        let mut handler = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        handler.run_system_call(self)
    }
}

impl<CTX, INSP, INST, PRECOMPILES> SystemCallCommitEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<
            Journal: JournalTr<FinalOutput = JournalOutput>,
            Db: DatabaseCommit,
            Tx: SystemCallTx,
        > + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call_commit(
        &mut self,
        data: Bytes,
        system_contract_address: Address,
    ) -> Self::CommitOutput {
        self.transact_system_call(data, system_contract_address)
            .map(|r| {
                self.db().commit(r.state);
                r.result
            })
    }
}
