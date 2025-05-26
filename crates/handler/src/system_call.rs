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
    /// Creates new transaction for system call.
    fn new_system_tx(data: Bytes, system_contract_address: Address) -> Self;
}

impl SystemCallTx for TxEnv {
    fn new_system_tx(data: Bytes, system_contract_address: Address) -> Self {
        TxEnv {
            caller: SYSTEM_ADDRESS,
            data,
            kind: TxKind::Call(system_contract_address),
            gas_limit: 30_000_000,
            ..Default::default()
        }
    }
}

/// API for executing the system calls. System calls dont deduct the caller or reward the
/// beneficiary. They are used before and after block execution to insert or obtain blockchain state.
///
/// It act similar to `transact` function and sets default Tx with data and system contract as a target.
pub trait SystemCallEvm: ExecuteEvm {
    /// System call is a special transaction call that is used to call a system contract.
    ///
    /// Transaction fields are reset and set in [`SystemCallTx`] and data and target are set to
    /// given values.
    ///
    /// Block values are taken into account and will determent how system call will be executed.
    fn transact_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self::Output;
}

/// Extension of the [`SystemCallEvm`] trait that adds a method that commits the state after execution.
pub trait SystemCallCommitEvm: SystemCallEvm + ExecuteCommitEvm {
    /// Transact the system call and commit to the state.
    fn transact_system_call_commit(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
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
        system_contract_address: Address,
        data: Bytes,
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
        system_contract_address: Address,
        data: Bytes,
    ) -> Self::CommitOutput {
        self.transact_system_call(system_contract_address, data)
            .map(|r| {
                self.db().commit(r.state);
                r.result
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{MainBuilder, MainContext};

    use super::*;
    use context::{
        result::{ExecutionResult, Output, SuccessReason},
        Context,
    };
    use database::InMemoryDB;
    use primitives::{b256, bytes, U256};
    use state::{AccountInfo, Bytecode};

    const HISTORY_STORAGE_ADDRESS: Address = address!("0x0000F90827F1C53a10cb7A02335B175320002935");
    static HISTORY_STORAGE_CODE: Bytes = bytes!("0x3373fffffffffffffffffffffffffffffffffffffffe14604657602036036042575f35600143038111604257611fff81430311604257611fff9006545f5260205ff35b5f5ffd5b5f35611fff60014303065500");

    #[test]
    fn test_system_call() {
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            HISTORY_STORAGE_ADDRESS,
            AccountInfo::default().with_code(Bytecode::new_legacy(HISTORY_STORAGE_CODE.clone())),
        );

        let block_hash =
            b256!("0x1111111111111111111111111111111111111111111111111111111111111111");

        let mut my_evm = Context::mainnet()
            .with_db(db)
            // block with number 1 will set storage at slot 0.
            .modify_block_chained(|b| b.number = 1)
            .build_mainnet();
        let res = my_evm
            .transact_system_call(HISTORY_STORAGE_ADDRESS, block_hash.0.into())
            .unwrap();

        let result = res.result;
        let state = res.state;
        assert_eq!(
            result,
            ExecutionResult::Success {
                reason: SuccessReason::Stop,
                gas_used: 22143,
                gas_refunded: 0,
                logs: vec![],
                output: Output::Call(Bytes::default())
            }
        );
        // only system contract is updated and present
        assert_eq!(state.len(), 1);
        assert_eq!(
            state[&HISTORY_STORAGE_ADDRESS]
                .storage
                .get(&U256::from(0))
                .map(|slot| slot.present_value)
                .unwrap_or_default(),
            U256::from_be_bytes(block_hash.0),
            "State is not updated {state:?}"
        );
    }
}
