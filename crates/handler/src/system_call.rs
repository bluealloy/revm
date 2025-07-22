//! System call logic for external state transitions required by certain EIPs (notably [EIP-2935](https://eips.ethereum.org/EIPS/eip-2935) and [EIP-4788](https://eips.ethereum.org/EIPS/eip-4788)).
//!
//! These EIPs require the client to perform special system calls to update state (such as block hashes or beacon roots) at block boundaries, outside of normal EVM transaction execution. REVM provides the system call mechanism, but the actual state transitions must be performed by the client or test harness, not by the EVM itself.
//!
//! # Example: Using `transact_system_call` for pre/post block hooks
//!
//! The client should use [`SystemCallEvm::transact_system_call`] method to perform required state updates before or after block execution, as specified by the EIP:
//!
//! ```rust,ignore
//! // Example: update beacon root (EIP-4788) at the start of a block
//! let beacon_root: Bytes = ...; // obtained from consensus layer
//! let beacon_contract: Address = "0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02".parse().unwrap();
//! evm.transact_system_call(beacon_contract, beacon_root)?;
//!
//! // Example: update block hash (EIP-2935) at the end of a block
//! let block_hash: Bytes = ...; // new block hash
//! let history_contract: Address = "0x0000F90827F1C53a10cb7A02335B175320002935".parse().unwrap();
//! evm.transact_system_call(history_contract, block_hash)?;
//! ```
//!
//! See the book section on [External State Transitions](../../book/src/external_state_transitions.md) for more details.
use crate::{
    frame::EthFrame, instructions::InstructionProvider, ExecuteCommitEvm, ExecuteEvm, Handler,
    MainnetHandler, PrecompileProvider,
};
use context::{result::ExecResultAndState, ContextSetters, ContextTr, Evm, JournalTr, TxEnv};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use primitives::{address, Address, Bytes, TxKind};
use state::EvmState;

/// The system address used for system calls.
pub const SYSTEM_ADDRESS: Address = address!("0xfffffffffffffffffffffffffffffffffffffffe");

/// Creates the system transaction with default values and set data and tx call target to system contract address
/// that is going to be called.
///
/// The caller is set to be [`SYSTEM_ADDRESS`].
///
/// It is used inside [`SystemCallEvm`] and [`SystemCallCommitEvm`] traits to prepare EVM for system call execution.
pub trait SystemCallTx: Sized {
    /// Creates new transaction for system call.
    fn new_system_tx(system_contract_address: Address, data: Bytes) -> Self {
        Self::new_system_tx_with_caller(SYSTEM_ADDRESS, system_contract_address, data)
    }

    /// Creates a new system transaction with a custom caller address.
    fn new_system_tx_with_caller(
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self;
}

impl SystemCallTx for TxEnv {
    fn new_system_tx_with_caller(
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self {
        TxEnv::builder()
            .caller(caller)
            .data(data)
            .kind(TxKind::Call(system_contract_address))
            .gas_limit(30_000_000)
            .build()
            .unwrap()
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
    fn transact_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error>;

    /// Calls [`SystemCallEvm::transact_system_call_with_caller`] with [`SYSTEM_ADDRESS`] as a caller.
    fn transact_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.transact_system_call_with_caller(SYSTEM_ADDRESS, system_contract_address, data)
    }

    /// Transact the system call and finalize.
    ///
    /// Internally calls combo of `transact_system_call` and `finalize` functions.
    fn transact_system_call_finalize(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        self.transact_system_call_with_caller_finalize(
            SYSTEM_ADDRESS,
            system_contract_address,
            data,
        )
    }

    /// Calls [`SystemCallEvm::transact_system_call_with_caller`] and `finalize` functions.
    fn transact_system_call_with_caller_finalize(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let result =
            self.transact_system_call_with_caller(caller, system_contract_address, data)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(result, state))
    }
}

/// Extension of the [`SystemCallEvm`] trait that adds a method that commits the state after execution.
pub trait SystemCallCommitEvm: SystemCallEvm + ExecuteCommitEvm {
    /// Transact the system call and commit to the state.
    fn transact_system_call_commit(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.transact_system_call_with_caller_commit(SYSTEM_ADDRESS, system_contract_address, data)
    }

    /// Calls [`SystemCallCommitEvm::transact_system_call_commit`] with [`SYSTEM_ADDRESS`] as a caller.
    fn transact_system_call_with_caller_commit(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error>;
}

impl<CTX, INSP, INST, PRECOMPILES> SystemCallEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>, Tx: SystemCallTx> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        // set tx fields.
        self.set_tx(CTX::Tx::new_system_tx_with_caller(
            caller,
            system_contract_address,
            data,
        ));
        // create handler
        MainnetHandler::default().run_system_call(self)
    }
}

impl<CTX, INSP, INST, PRECOMPILES> SystemCallCommitEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit, Tx: SystemCallTx>
        + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call_with_caller_commit(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.transact_system_call_with_caller_finalize(caller, system_contract_address, data)
            .map(|output| {
                self.db_mut().commit(output.state);
                output.result
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{MainBuilder, MainContext};

    use super::*;
    use context::{
        result::{ExecutionResult, Output, SuccessReason},
        Context, Transaction,
    };
    use database::InMemoryDB;
    use primitives::{b256, bytes, StorageKey, U256};
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

        let mut evm = Context::mainnet()
            .with_db(db)
            // block with number 1 will set storage at slot 0.
            .modify_block_chained(|b| b.number = U256::ONE)
            .build_mainnet();
        let output = evm
            .transact_system_call_finalize(HISTORY_STORAGE_ADDRESS, block_hash.0.into())
            .unwrap();

        // system call gas limit is 30M
        assert_eq!(evm.ctx.tx().gas_limit(), 30_000_000);

        assert_eq!(
            output.result,
            ExecutionResult::Success {
                reason: SuccessReason::Stop,
                gas_used: 22143,
                gas_refunded: 0,
                logs: vec![],
                output: Output::Call(Bytes::default())
            }
        );
        // only system contract is updated and present
        assert_eq!(output.state.len(), 1);
        assert_eq!(
            output.state[&HISTORY_STORAGE_ADDRESS]
                .storage
                .get(&StorageKey::from(0))
                .map(|slot| slot.present_value)
                .unwrap_or_default(),
            U256::from_be_bytes(block_hash.0),
            "State is not updated {:?}",
            output.state
        );
    }
}
