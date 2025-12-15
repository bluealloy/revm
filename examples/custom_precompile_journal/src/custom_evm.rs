//! Custom EVM implementation with journal-accessing precompiles.

use crate::precompile_provider::CustomPrecompileProvider;
use revm::{
    context::{ContextError, ContextSetters, ContextTr, Evm, FrameStack},
    handler::{
        evm::FrameTr, instructions::EthInstructions, EthFrame, EvmTr, FrameInitOrResult,
        ItemOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Database, Inspector,
};

/// Custom EVM variant with journal-accessing precompiles.
///
/// This EVM extends the standard behavior by using a custom precompile provider
/// that includes journal access functionality. It follows the same pattern as MyEvm
/// but uses CustomPrecompileProvider instead of EthPrecompiles.
#[derive(Debug)]
pub struct CustomEvm<CTX, INSP>(
    pub  Evm<
        CTX,
        INSP,
        EthInstructions<EthInterpreter, CTX>,
        CustomPrecompileProvider,
        EthFrame<EthInterpreter>,
    >,
);

impl<CTX, INSP> CustomEvm<CTX, INSP>
where
    CTX: ContextTr<Cfg: revm::context::Cfg<Spec = SpecId>>,
{
    /// Creates a new instance of CustomEvm with the provided context and inspector.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context that manages state, environment, and journaling
    /// * `inspector` - The inspector for debugging and tracing execution
    ///
    /// # Returns
    ///
    /// A new CustomEvm instance configured with:
    /// - The provided context and inspector
    /// - Mainnet instruction set
    /// - Custom precompiles with journal access
    /// - A fresh frame stack for execution
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: CustomPrecompileProvider::new_with_spec(SpecId::CANCUN),
            frame_stack: FrameStack::new(),
        })
    }
}

impl<CTX, INSP> EvmTr for CustomEvm<CTX, INSP>
where
    CTX: ContextTr<Cfg: revm::context::Cfg<Spec = SpecId>>,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = CustomPrecompileProvider;
    type Frame = EthFrame<EthInterpreter>;

    fn all(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
    ) {
        self.0.all()
    }

    fn all_mut(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
    ) {
        self.0.all_mut()
    }

    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<
        ItemOrResult<&mut Self::Frame, <Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_init(frame_input)
    }

    fn frame_run(
        &mut self,
    ) -> Result<
        FrameInitOrResult<Self::Frame>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_run()
    }

    fn frame_return_result(
        &mut self,
        frame_result: <Self::Frame as FrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_return_result(frame_result)
    }
}

impl<CTX, INSP> InspectorEvmTr for CustomEvm<CTX, INSP>
where
    CTX: ContextSetters<Cfg: revm::context::Cfg<Spec = SpecId>, Journal: JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
        &Self::Inspector,
    ) {
        self.0.all_inspector()
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
        &mut Self::Inspector,
    ) {
        self.0.all_mut_inspector()
    }
}

#[cfg(test)]
mod tests {
    use crate::{custom_evm::CustomEvm, precompile_provider::CUSTOM_PRECOMPILE_ADDRESS};
    use revm::{
        context::{Context, ContextSetters, TxEnv},
        context_interface::{result::EVMError, ContextTr},
        database::InMemoryDB,
        handler::{Handler, MainnetHandler},
        inspector::{Inspector, JournalExt},
        interpreter::interpreter::EthInterpreter,
        primitives::{address, Log, TxKind, U256},
        state::AccountInfo,
        MainContext,
    };
    use std::vec::Vec;

    /// Custom inspector that captures logs
    #[derive(Debug, Default)]
    struct LogCapturingInspector {
        captured_logs: Vec<Log>,
    }

    impl LogCapturingInspector {
        fn new() -> Self {
            Self {
                captured_logs: Vec::new(),
            }
        }

        fn logs(&self) -> &[Log] {
            &self.captured_logs
        }
    }

    impl<CTX> Inspector<CTX, EthInterpreter> for LogCapturingInspector
    where
        CTX: ContextTr + ContextSetters<Journal: JournalExt>,
    {
        fn log(&mut self, _context: &mut CTX, log: Log) {
            // Capture logs as they're created
            self.captured_logs.push(log);
        }
    }

    #[test]
    fn test_custom_precompile_creates_log() {
        // Setup initial accounts
        let user_address = address!("0000000000000000000000000000000000000001");
        let mut db = InMemoryDB::default();

        // Give the user some ETH for gas
        let user_balance = U256::from(10).pow(U256::from(18)); // 1 ETH
        db.insert_account_info(
            user_address,
            AccountInfo {
                balance: user_balance,
                nonce: 0,
                code_hash: revm::primitives::KECCAK_EMPTY,
                code: None,
                account_id: None,
            },
        );

        // Give the precompile some initial balance for transfers
        db.insert_account_info(
            CUSTOM_PRECOMPILE_ADDRESS,
            AccountInfo {
                balance: U256::from(1000), // 1000 wei
                nonce: 0,
                code_hash: revm::primitives::KECCAK_EMPTY,
                code: None,
                account_id: None,
            },
        );

        // Create custom EVM with log capturing inspector
        let context = Context::mainnet().with_db(db);
        let inspector = LogCapturingInspector::new();
        let mut evm = CustomEvm::new(context, inspector);

        // Write value 42 to storage (this should create a log)
        let storage_value = U256::from(42);
        evm.0.ctx.set_tx(
            TxEnv::builder()
                .caller(user_address)
                .kind(TxKind::Call(CUSTOM_PRECOMPILE_ADDRESS))
                .data(storage_value.to_be_bytes_vec().into())
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let result: Result<
            _,
            EVMError<core::convert::Infallible, revm::context::result::InvalidTransaction>,
        > = MainnetHandler::default().run(&mut evm);

        // Verify transaction succeeded
        assert!(
            result.is_ok(),
            "Transaction should succeed, got: {result:?}"
        );

        match result.unwrap() {
            revm::context::result::ExecutionResult::Success { logs, .. } => {
                // Transaction succeeded, now check logs from execution result
                // Note: Inspector might not be called for precompile logs,
                // so we check the execution result logs instead

                // Also check inspector logs (though they may be empty)
                let inspector_logs = evm.0.inspector.logs();

                // Combine both sources - use execution result logs if inspector logs are empty
                let all_logs = if inspector_logs.is_empty() {
                    &logs
                } else {
                    inspector_logs
                };

                // Verify that at least one log was captured
                assert!(
                    !all_logs.is_empty(),
                    "Should have captured at least one log (either from inspector or execution result)"
                );

                // Find the log from our custom precompile
                let precompile_log = all_logs
                    .iter()
                    .find(|log| log.address == CUSTOM_PRECOMPILE_ADDRESS);

                assert!(
                    precompile_log.is_some(),
                    "Should have a log from the custom precompile. Found {} total logs",
                    all_logs.len()
                );

                let log = precompile_log.unwrap();

                // Verify log structure
                assert_eq!(log.address, CUSTOM_PRECOMPILE_ADDRESS);
                assert_eq!(log.data.topics().len(), 2, "Should have 2 topics");

                // Topic 1 should be the caller address (left-padded to 32 bytes)
                let topic1 = log.data.topics()[1];
                let mut expected_caller_bytes = [0u8; 32];
                expected_caller_bytes[12..32].copy_from_slice(user_address.as_slice());
                let expected_caller_topic = revm::primitives::B256::from(expected_caller_bytes);
                assert_eq!(
                    topic1, expected_caller_topic,
                    "Second topic should be caller address"
                );

                // Data should contain the value that was written (42)
                let log_data_bytes = &log.data.data;
                let logged_value = U256::from_be_slice(log_data_bytes);
                assert_eq!(
                    logged_value,
                    U256::from(42),
                    "Log data should contain the written value (42)"
                );

                println!("✅ Test passed! Log was successfully created and captured");
                println!("   Log address: {}", log.address);
                println!("   Number of topics: {}", log.data.topics().len());
                println!("   Logged value: {logged_value}");
                println!(
                    "   Inspector logs: {}, Execution result logs: {}",
                    inspector_logs.len(),
                    logs.len()
                );
            }
            revm::context::result::ExecutionResult::Revert { .. } => {
                panic!("Transaction reverted unexpectedly");
            }
            revm::context::result::ExecutionResult::Halt { reason, .. } => {
                panic!("Transaction halted unexpectedly: {reason:?}");
            }
        }
    }

    #[test]
    fn test_read_operation_does_not_create_log() {
        // Setup initial accounts
        let user_address = address!("0000000000000000000000000000000000000001");
        let mut db = InMemoryDB::default();

        // Give the user some ETH for gas
        let user_balance = U256::from(10).pow(U256::from(18)); // 1 ETH
        db.insert_account_info(
            user_address,
            AccountInfo {
                balance: user_balance,
                nonce: 0,
                code_hash: revm::primitives::KECCAK_EMPTY,
                code: None,
                account_id: None,
            },
        );

        // Create custom EVM with log capturing inspector
        let context = Context::mainnet().with_db(db);
        let inspector = LogCapturingInspector::new();
        let mut evm = CustomEvm::new(context, inspector);

        // Read from storage (empty input - should not create a log)
        evm.0.ctx.set_tx(
            TxEnv::builder()
                .caller(user_address)
                .kind(TxKind::Call(CUSTOM_PRECOMPILE_ADDRESS))
                .data(revm::primitives::Bytes::new()) // Empty data for read operation
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let result: Result<
            _,
            EVMError<core::convert::Infallible, revm::context::result::InvalidTransaction>,
        > = MainnetHandler::default().run(&mut evm);

        // Verify transaction succeeded
        assert!(
            result.is_ok(),
            "Transaction should succeed, got: {result:?}"
        );

        match result.unwrap() {
            revm::context::result::ExecutionResult::Success { .. } => {
                // Transaction succeeded, check that no logs were created
                let logs = evm.0.inspector.logs();

                // Verify that no logs from the precompile were captured
                let precompile_log = logs
                    .iter()
                    .find(|log| log.address == CUSTOM_PRECOMPILE_ADDRESS);

                assert!(
                    precompile_log.is_none(),
                    "Read operation should not create any logs"
                );

                println!("✅ Test passed! Read operation correctly did not create any logs");
            }
            revm::context::result::ExecutionResult::Revert { .. } => {
                panic!("Transaction reverted unexpectedly");
            }
            revm::context::result::ExecutionResult::Halt { reason, .. } => {
                panic!("Transaction halted unexpectedly: {reason:?}");
            }
        }
    }
}
