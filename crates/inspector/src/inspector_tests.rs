#[cfg(test)]
mod tests {
    use crate::{
        InspectCommitEvm, InspectEvm, InspectSystemCallEvm, InspectorEvent, TestInspector,
    };
    use context::{CfgEnv, Context, TxEnv};
    use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
    use handler::{ExecuteEvm, MainBuilder, MainContext};
    use primitives::{
        address,
        eip7708::{ETH_TRANSFER_LOG_ADDRESS, ETH_TRANSFER_LOG_TOPIC},
        hardfork::SpecId,
        Address, Bytes, TxKind, B256, U256,
    };
    use state::{bytecode::opcode, AccountInfo, Bytecode};

    #[test]
    fn test_push_opcodes_and_stack_operations() {
        // PUSH1 0x42, PUSH2 0x1234, ADD, PUSH1 0x00, MSTORE, STOP
        let code = Bytes::from(vec![
            opcode::PUSH1,
            0x42,
            opcode::PUSH2,
            0x12,
            0x34,
            opcode::ADD,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE,
            opcode::STOP,
        ]);

        let bytecode = Bytecode::new_raw(code);
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();
        let step_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Step(record) = e {
                    Some(record)
                } else {
                    None
                }
            })
            .collect();

        // Verify PUSH1 0x42
        let push1_event = &step_events[0];
        assert_eq!(push1_event.opcode_name, "PUSH1");
        assert_eq!(push1_event.before.stack_len, 0);
        assert_eq!(push1_event.after.as_ref().unwrap().stack_len, 1);

        // Verify PUSH2 0x1234
        let push2_event = &step_events[1];
        assert_eq!(push2_event.opcode_name, "PUSH2");
        assert_eq!(push2_event.before.stack_len, 1);
        assert_eq!(push2_event.after.as_ref().unwrap().stack_len, 2);

        // Verify ADD
        let add_event = &step_events[2];
        assert_eq!(add_event.opcode_name, "ADD");
        assert_eq!(add_event.before.stack_len, 2);
        assert_eq!(add_event.after.as_ref().unwrap().stack_len, 1);

        // Verify all opcodes were tracked
        assert!(inspector.get_step_count() >= 5); // PUSH1, PUSH2, ADD, PUSH1, MSTORE, STOP
    }

    #[test]
    fn test_jump_and_jumpi_control_flow() {
        // PUSH1 0x08, JUMP, INVALID, JUMPDEST, PUSH1 0x01, PUSH1 0x0F, JUMPI, INVALID, JUMPDEST, STOP
        let code = Bytes::from(vec![
            opcode::PUSH1,
            0x08,
            opcode::JUMP,
            opcode::INVALID,
            opcode::INVALID,
            opcode::INVALID,
            opcode::INVALID,
            opcode::INVALID,
            opcode::JUMPDEST, // offset 0x08
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x0F,
            opcode::JUMPI,
            opcode::INVALID,
            opcode::JUMPDEST, // offset 0x0F
            opcode::STOP,
        ]);

        let bytecode = Bytecode::new_raw(code);
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();
        let step_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Step(record) = e {
                    Some(record)
                } else {
                    None
                }
            })
            .collect();

        // Find JUMP instruction
        let jump_event = step_events
            .iter()
            .find(|e| e.opcode_name == "JUMP")
            .unwrap();
        assert_eq!(jump_event.before.pc, 2); // After PUSH1 0x08
        assert_eq!(jump_event.after.as_ref().unwrap().pc, 8); // Jumped to JUMPDEST

        // Find JUMPI instruction
        let jumpi_event = step_events
            .iter()
            .find(|e| e.opcode_name == "JUMPI")
            .unwrap();
        assert!(jumpi_event.before.stack_len >= 2); // Has condition and destination
                                                    // JUMPI should have jumped since condition is 1 (true)
        assert_eq!(jumpi_event.after.as_ref().unwrap().pc, 0x0F);
    }

    #[test]
    fn test_call_operations() {
        // For CALL tests, we need a more complex setup with multiple contracts
        // Deploy a simple contract that returns a value
        let callee_code = Bytes::from(vec![
            opcode::PUSH1,
            0x42, // Push return value
            opcode::PUSH1,
            0x00, // Push memory offset
            opcode::MSTORE,
            opcode::PUSH1,
            0x20, // Push return size
            opcode::PUSH1,
            0x00, // Push return offset
            opcode::RETURN,
        ]);

        // Caller contract that calls the callee
        let caller_code = Bytes::from(vec![
            // Setup CALL parameters
            opcode::PUSH1,
            0x20, // retSize
            opcode::PUSH1,
            0x00, // retOffset
            opcode::PUSH1,
            0x00, // argsSize
            opcode::PUSH1,
            0x00, // argsOffset
            opcode::PUSH1,
            0x00, // value
            opcode::PUSH20,
            // address: 20 bytes to match callee_address exactly
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x01,
            opcode::PUSH2,
            0xFF,
            0xFF, // gas
            opcode::CALL,
            opcode::STOP,
        ]);

        // Create a custom database with two contracts
        let mut db = database::InMemoryDB::default();

        // Add caller contract at BENCH_TARGET
        db.insert_account_info(
            BENCH_TARGET,
            AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                code_hash: primitives::keccak256(&caller_code),
                code: Some(Bytecode::new_raw(caller_code)),
                ..Default::default()
            },
        );

        // Add callee contract at a specific address
        let callee_address = Address::new([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ]);
        db.insert_account_info(
            callee_address,
            AccountInfo {
                balance: U256::ZERO,
                nonce: 0,
                code_hash: primitives::keccak256(&callee_code),
                code: Some(Bytecode::new_raw(callee_code)),
                ..Default::default()
            },
        );

        let ctx = Context::mainnet().with_db(db);
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();

        // Find CALL events
        let call_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Call { inputs, outcome } = e {
                    Some((inputs, outcome))
                } else {
                    None
                }
            })
            .collect();

        assert!(!call_events.is_empty(), "Should have recorded CALL events");
        let (call_inputs, call_outcome) = &call_events[0];
        // The test setup might be using BENCH_CALLER as the default target
        // Just verify that a call was made and completed successfully
        assert_eq!(call_inputs.target_address, BENCH_TARGET);
        assert!(call_outcome.is_some(), "Call should have completed");
    }

    #[test]
    fn test_create_opcodes() {
        // CREATE test: deploy a contract that creates another contract
        let init_code = vec![
            opcode::PUSH1,
            0x42, // Push constructor value
            opcode::PUSH1,
            0x00, // Push memory offset
            opcode::MSTORE,
            opcode::PUSH1,
            0x20, // Push return size
            opcode::PUSH1,
            0x00, // Push return offset
            opcode::RETURN,
        ];

        let create_code = vec![
            // First, store init code in memory using CODECOPY
            opcode::PUSH1,
            init_code.len() as u8, // size
            opcode::PUSH1,
            0x20, // code offset (after CREATE params)
            opcode::PUSH1,
            0x00, // memory offset
            opcode::CODECOPY,
            // CREATE parameters
            opcode::PUSH1,
            init_code.len() as u8, // size
            opcode::PUSH1,
            0x00, // offset
            opcode::PUSH1,
            0x00, // value
            opcode::CREATE,
            opcode::STOP,
        ];

        let mut full_code = create_code;
        full_code.extend_from_slice(&init_code);

        let bytecode = Bytecode::new_raw(Bytes::from(full_code));
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();

        // Find CREATE events
        let create_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Create { inputs, outcome } = e {
                    Some((inputs, outcome))
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !create_events.is_empty(),
            "Should have recorded CREATE events"
        );
        let (_create_inputs, create_outcome) = &create_events[0];
        assert!(create_outcome.is_some(), "CREATE should have completed");
    }

    #[test]
    fn test_log_operations() {
        // Simple LOG0 test - no topics
        let code = vec![
            // Store some data in memory for the log
            opcode::PUSH1,
            0x42,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE,
            // LOG0 parameters
            opcode::PUSH1,
            0x20, // size
            opcode::PUSH1,
            0x00, // offset
            opcode::LOG0,
            opcode::STOP,
        ];

        let bytecode = Bytecode::new_raw(Bytes::from(code));
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();

        // Find LOG events
        let log_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Log(log) = e {
                    Some(log)
                } else {
                    None
                }
            })
            .collect();

        // Remove debug code - test should work now

        assert_eq!(log_events.len(), 1, "Should have recorded one LOG event");
        let log = &log_events[0];
        assert_eq!(log.topics().len(), 0, "LOG0 should have 0 topics");
    }

    #[test]
    fn test_eip7708_tx_value_transfer_log_is_inspected() {
        let recipient = address!("4000000000000000000000000000000000000000");
        let value = U256::from(1_000_000_000_000_000u128);

        let ctx = Context::mainnet()
            .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new()));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        let result = evm
            .inspect_one_tx(
                TxEnv::builder_for_bench()
                    .to(recipient)
                    .value(value)
                    .gas_limit(300_000)
                    .gas_price(0)
                    .build_fill(),
            )
            .unwrap();
        assert!(result.is_success());

        let events = evm.inspector.get_events();
        let transfer_log = events.iter().find_map(|event| {
            let InspectorEvent::Log(log) = event else {
                return None;
            };
            (log.address == ETH_TRANSFER_LOG_ADDRESS
                && log.data.topics().len() == 3
                && log.data.topics()[0] == ETH_TRANSFER_LOG_TOPIC
                && log.data.topics()[1] == B256::left_padding_from(BENCH_CALLER.as_slice())
                && log.data.topics()[2] == B256::left_padding_from(recipient.as_slice()))
            .then_some(log)
        });

        assert!(
            transfer_log.is_some(),
            "expected inspector to receive EIP-7708 tx value transfer log"
        );
    }

    #[test]
    fn test_eip7708_selfdestruct_transfer_log_is_inspected() {
        let code = Bytes::from(vec![opcode::CALLER, opcode::SELFDESTRUCT, opcode::STOP]);
        let ctx = Context::mainnet()
            .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(code)));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        let result = evm
            .inspect_one_tx(
                TxEnv::builder_for_bench()
                    .gas_limit(100_000)
                    .gas_price(0)
                    .build_fill(),
            )
            .unwrap();
        assert!(result.is_success());

        let events = evm.inspector.get_events();
        let transfer_log = events.iter().find_map(|event| {
            let InspectorEvent::Log(log) = event else {
                return None;
            };
            (log.address == ETH_TRANSFER_LOG_ADDRESS
                && log.data.topics().len() == 3
                && log.data.topics()[0] == ETH_TRANSFER_LOG_TOPIC
                && log.data.topics()[1] == B256::left_padding_from(BENCH_TARGET.as_slice())
                && log.data.topics()[2] == B256::left_padding_from(BENCH_CALLER.as_slice()))
            .then_some(log)
        });

        assert!(
            transfer_log.is_some(),
            "expected inspector to receive EIP-7708 selfdestruct transfer log"
        );
    }

    #[test]
    fn test_selfdestruct() {
        // SELFDESTRUCT test
        let beneficiary = address!("3000000000000000000000000000000000000000");
        let mut code = vec![opcode::PUSH20];
        code.extend_from_slice(beneficiary.as_ref());
        code.push(opcode::SELFDESTRUCT);

        let bytecode = Bytecode::new_raw(Bytes::from(code));
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();

        // Find SELFDESTRUCT events
        let selfdestruct_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Selfdestruct {
                    address,
                    beneficiary,
                    value,
                } = e
                {
                    Some((address, beneficiary, value))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            selfdestruct_events.len(),
            1,
            "Should have recorded SELFDESTRUCT event"
        );
        let (_address, event_beneficiary, _value) = selfdestruct_events[0];
        assert_eq!(*event_beneficiary, beneficiary);
    }

    #[test]
    fn cancun_selfdestruct_to_self_does_not_reuse_prior_journal_entry() {
        let code = Bytes::from(vec![opcode::ADDRESS, opcode::SELFDESTRUCT]);
        let ctx = Context::mainnet()
            .with_cfg(CfgEnv::new_with_spec(SpecId::CANCUN))
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(code)));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        let result = evm
            .inspect_one_tx(
                TxEnv::builder()
                    .caller(BENCH_CALLER)
                    .kind(TxKind::Call(BENCH_TARGET))
                    .value(U256::from(459))
                    .gas_limit(100_000)
                    .gas_price(0)
                    .build()
                    .unwrap(),
            )
            .unwrap();
        assert!(result.is_success());

        assert!(
            !evm.inspector
                .get_events()
                .iter()
                .any(|event| matches!(event, InspectorEvent::Selfdestruct { .. })),
            "Cancun selfdestruct-to-self must not reuse the transaction value-transfer journal entry"
        );
    }

    #[test]
    fn test_comprehensive_inspector_integration() {
        // Complex contract with multiple operations:
        // 1. PUSH and arithmetic
        // 2. Memory operations
        // 3. Conditional jump
        // 4. LOG0

        let code = vec![
            // Stack operations
            opcode::PUSH1,
            0x10,
            opcode::PUSH1,
            0x20,
            opcode::ADD,
            opcode::DUP1,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE,
            // Conditional jump
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x00,
            opcode::MLOAD,
            opcode::GT,
            opcode::PUSH1,
            0x17, // Jump destination (adjusted)
            opcode::JUMPI,
            // This should be skipped
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::REVERT,
            // Jump destination
            opcode::JUMPDEST, // offset 0x14
            // LOG0
            opcode::PUSH1,
            0x20,
            opcode::PUSH1,
            0x00,
            opcode::LOG0,
            opcode::STOP,
        ];

        let bytecode = Bytecode::new_raw(Bytes::from(code));
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Run transaction
        let _ = evm.inspect_one_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .build()
                .unwrap(),
        );

        let inspector = &evm.inspector;
        let events = inspector.get_events();

        // Verify we captured various event types
        let step_count = events
            .iter()
            .filter(|e| matches!(e, InspectorEvent::Step(_)))
            .count();
        let log_count = events
            .iter()
            .filter(|e| matches!(e, InspectorEvent::Log(_)))
            .count();

        assert!(step_count > 10, "Should have multiple step events");
        assert_eq!(log_count, 1, "Should have one log event");

        // Verify stack operations were tracked
        let step_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let InspectorEvent::Step(record) = e {
                    Some(record)
                } else {
                    None
                }
            })
            .collect();

        // Find ADD operation
        let add_event = step_events.iter().find(|e| e.opcode_name == "ADD").unwrap();
        assert_eq!(add_event.before.stack_len, 2);
        assert_eq!(add_event.after.as_ref().unwrap().stack_len, 1);

        // Verify memory was written
        let mstore_event = step_events
            .iter()
            .find(|e| e.opcode_name == "MSTORE")
            .unwrap();
        assert!(mstore_event.after.as_ref().unwrap().memory_size > 0);

        // Verify conditional jump worked correctly
        let jumpi_event = step_events
            .iter()
            .find(|e| e.opcode_name == "JUMPI")
            .unwrap();
        assert_eq!(
            jumpi_event.after.as_ref().unwrap().pc,
            0x17,
            "Should have jumped to JUMPDEST"
        );
    }

    #[test]
    fn test_system_call_inspection_basic() {
        // PUSH1 0x42, SSTORE, STOP
        let code = Bytes::from(vec![
            opcode::PUSH1,
            0x42,
            opcode::PUSH1,
            0x00,
            opcode::SSTORE,
            opcode::STOP,
        ]);

        let bytecode = Bytecode::new_raw(code);
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        let result = evm
            .inspect_system_call(BENCH_TARGET, Bytes::default())
            .unwrap();

        assert!(result.result.is_success());
        assert!(evm.inspector.get_step_count() > 0);
        assert!(!result.state.is_empty());
    }

    #[test]
    fn test_system_call_inspection_api_variants() {
        let code = vec![
            opcode::CALLER,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE,
            opcode::PUSH1,
            0x20,
            opcode::PUSH1,
            0x00,
            opcode::RETURN,
        ];

        let bytecode = Bytecode::new_raw(Bytes::from(code));
        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode));
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Test inspect_one_system_call
        let result = evm
            .inspect_one_system_call(BENCH_TARGET, Bytes::default())
            .unwrap();
        assert!(result.is_success());

        // Test inspect_one_system_call_with_caller
        let custom_caller = address!("0x1234567890123456789012345678901234567890");
        let result = evm
            .inspect_one_system_call_with_caller(custom_caller, BENCH_TARGET, Bytes::default())
            .unwrap();
        assert!(result.is_success());

        // Test inspect_one_system_call_with_inspector
        let result = evm
            .inspect_one_system_call_with_inspector(
                BENCH_TARGET,
                Bytes::default(),
                TestInspector::new(),
            )
            .unwrap();
        assert!(result.is_success());

        assert!(evm.inspector.get_step_count() > 0);
    }

    /// Regression test for inspector vs non-inspector divergence in
    /// `run_system_call`'s `ResultGas` construction.
    ///
    /// The non-inspect path uses `build_result_gas`, which subtracts the gas
    /// reservoir from `limit - remaining`. The inspect path used to construct
    /// `ResultGas` inline via `gas.total_gas_spent()` (which is just
    /// `limit - remaining`), so when a system call left `reservoir > 0` the
    /// two paths reported different `total_gas_spent`.
    ///
    /// To force `reservoir > 0` in the top-level system frame we run a
    /// scenario where the system contract calls a child that consumes state
    /// gas (SSTORE on a fresh slot under AMSTERDAM) and then REVERT. On
    /// revert the parent's reservoir is set to `child.state_gas_spent +
    /// child.reservoir` (see `frame::handle_reservoir_remaining_gas`), which
    /// is non-zero and triggers the divergence.
    #[test]
    fn test_system_call_gas_consistency_with_reservoir() {
        use database::{CacheDB, EmptyDB};
        use handler::SystemCallEvm;
        use primitives::hardfork::SpecId;

        let child_addr = address!("0x000000000000000000000000000000000000c0de");

        // PUSH1 0x42 PUSH1 0x00 SSTORE PUSH0 PUSH0 REVERT
        // Charges sstore_set_state_gas (32 * 1174 = 37568 under AMSTERDAM) and
        // then reverts so the parent inherits that state gas as reservoir.
        let child_code = Bytes::from(vec![
            opcode::PUSH1,
            0x42,
            opcode::PUSH1,
            0x00,
            opcode::SSTORE,
            opcode::PUSH0,
            opcode::PUSH0,
            opcode::REVERT,
        ]);

        // System contract: CALL child, ignore return value, STOP.
        let system_code = Bytes::from(vec![
            opcode::PUSH1,
            0x00, // retSize
            opcode::PUSH1,
            0x00, // retOffset
            opcode::PUSH1,
            0x00, // argsSize
            opcode::PUSH1,
            0x00, // argsOffset
            opcode::PUSH1,
            0x00, // value
            opcode::PUSH20,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xc0,
            0xde, // child_addr
            opcode::PUSH4,
            0x00,
            0xFF,
            0xFF,
            0xFF, // gas to forward
            opcode::CALL,
            opcode::POP,
            opcode::STOP,
        ]);

        let make_db = || {
            let mut db = CacheDB::<EmptyDB>::default();
            db.insert_account_info(
                BENCH_TARGET,
                AccountInfo {
                    balance: U256::ZERO,
                    nonce: 0,
                    code_hash: primitives::keccak256(&system_code),
                    code: Some(Bytecode::new_raw(system_code.clone())),
                    ..Default::default()
                },
            );
            db.insert_account_info(
                child_addr,
                AccountInfo {
                    balance: U256::ZERO,
                    nonce: 0,
                    code_hash: primitives::keccak256(&child_code),
                    code: Some(Bytecode::new_raw(child_code.clone())),
                    ..Default::default()
                },
            );
            db
        };

        let make_ctx = || {
            Context::mainnet()
                .modify_cfg_chained(|c| c.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM))
                .with_db(make_db())
        };

        // Non-inspect path.
        let mut evm_plain = make_ctx().build_mainnet();
        let result_plain = evm_plain
            .system_call_one(BENCH_TARGET, Bytes::default())
            .expect("non-inspect system call must succeed");

        // Inspect path.
        let mut evm_inspect = make_ctx().build_mainnet_with_inspector(TestInspector::new());
        let result_inspect = evm_inspect
            .inspect_one_system_call(BENCH_TARGET, Bytes::default())
            .expect("inspect system call must succeed");

        assert!(result_plain.is_success(), "non-inspect must succeed");
        assert!(result_inspect.is_success(), "inspect must succeed");

        let plain_total = result_plain.gas().total_gas_spent();
        let inspect_total = result_inspect.gas().total_gas_spent();

        // Both paths must report identical gas — observability must not depend
        // on whether an inspector is attached.
        assert_eq!(
            plain_total, inspect_total,
            "system_call total_gas_spent must match between inspect and non-inspect paths",
        );

        // Sanity check: state_gas_spent should also agree.
        assert_eq!(
            result_plain.gas().state_gas_spent_final(),
            result_inspect.gas().state_gas_spent_final(),
            "system_call state_gas_spent must match between inspect and non-inspect paths",
        );
    }

    /// Regression test for https://github.com/bluealloy/revm/issues/3779
    ///
    /// `inspect_tx` used to short-circuit on `inspect_one_tx` returning `Err`
    /// and skip `finalize()`, leaving the journal (EIP-2929 warm set, touched
    /// accounts) in a dirty state that leaked into the next transaction. The
    /// handler's error path calls `discard_tx`, which reverts account *values*
    /// but keeps the account *entries* in the state map, so a subsequent
    /// `finalize()` drains those leftovers — making the leak observable.
    ///
    /// Here a nonce mismatch triggers an `InvalidTransaction` *after* the
    /// caller and coinbase have been loaded and warmed in `load_accounts`.
    /// After `inspect_tx` returns `Err`, the journal must already be cleared,
    /// i.e. the drained state must be empty.
    #[test]
    fn test_inspect_tx_finalizes_journal_on_error() {
        use database::{CacheDB, EmptyDB};

        // Caller account exists in the DB with nonce = 1.
        let mut db = CacheDB::<EmptyDB>::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 1,
                ..Default::default()
            },
        );
        let ctx = Context::mainnet().with_db(db);
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        // Send a tx with nonce = 0 -> InvalidTransaction::NonceTooLow, raised
        // after load_accounts warmed the caller/coinbase.
        let result = evm.inspect_tx(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .nonce(0)
                .build()
                .unwrap(),
        );
        assert!(
            result.is_err(),
            "tx with stale nonce must error so the error path is exercised"
        );

        // `inspect_tx` must have finalized the journal on error, so draining
        // it now yields an empty state. Before the fix this contained the
        // warmed caller/coinbase accounts and was non-empty.
        let leftover = evm.finalize();
        assert!(
            leftover.is_empty(),
            "journal must be cleared after a failed inspect_tx, but found {} leftover accounts",
            leftover.len(),
        );
    }

    /// Companion to [`Self::test_inspect_tx_finalizes_journal_on_error`] for the
    /// `inspect_tx_commit` path: on error the journal must be finalized (not
    /// committed), so a subsequent drain yields an empty state.
    #[test]
    fn test_inspect_tx_commit_finalizes_journal_on_error() {
        use database::{CacheDB, EmptyDB};

        let mut db = CacheDB::<EmptyDB>::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 1,
                ..Default::default()
            },
        );
        let ctx = Context::mainnet().with_db(db);
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());

        let result = evm.inspect_tx_commit(
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(BENCH_TARGET))
                .gas_limit(100_000)
                .nonce(0)
                .build()
                .unwrap(),
        );
        assert!(
            result.is_err(),
            "tx with stale nonce must error so the error path is exercised"
        );

        let leftover = evm.finalize();
        assert!(
            leftover.is_empty(),
            "journal must be cleared after a failed inspect_tx_commit, but found {} leftover accounts",
            leftover.len(),
        );
    }

    fn transfer_logs(events: &[InspectorEvent]) -> Vec<(Address, Address, U256)> {
        events
            .iter()
            .filter_map(|event| {
                let InspectorEvent::Log(log) = event else {
                    return None;
                };
                if log.address != ETH_TRANSFER_LOG_ADDRESS
                    || log.data.topics().first() != Some(&ETH_TRANSFER_LOG_TOPIC)
                {
                    return None;
                }
                Some((
                    Address::from_word(log.data.topics()[1]),
                    Address::from_word(log.data.topics()[2]),
                    U256::from_be_slice(log.data.data.as_ref()),
                ))
            })
            .collect()
    }

    const CALLEE: Address = address!("5000000000000000000000000000000000000000");

    fn run_transfer_log_probe(
        code: Vec<u8>,
        callee_code: Option<Bytecode>,
    ) -> Vec<(Address, Address, U256)> {
        let mut db = database::CacheDB::<database::EmptyDB>::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                balance: U256::from(1_000_000_000u64),
                ..Default::default()
            },
        );
        db.insert_account_info(
            BENCH_TARGET,
            AccountInfo {
                balance: U256::from(1_000u64),
                code_hash: Bytecode::new_legacy(Bytes::from(code.clone())).hash_slow(),
                code: Some(Bytecode::new_legacy(Bytes::from(code))),
                ..Default::default()
            },
        );
        if let Some(callee_code) = callee_code {
            db.insert_account_info(
                CALLEE,
                AccountInfo {
                    code_hash: callee_code.hash_slow(),
                    code: Some(callee_code),
                    ..Default::default()
                },
            );
        }

        let ctx = Context::mainnet()
            .with_cfg(CfgEnv::new_with_spec(SpecId::AMSTERDAM))
            .with_db(db);
        let mut evm = ctx.build_mainnet_with_inspector(TestInspector::new());
        let result = evm
            .inspect_one_tx(
                TxEnv::builder()
                    .caller(BENCH_CALLER)
                    .kind(TxKind::Call(BENCH_TARGET))
                    .gas_limit(1_000_000)
                    .gas_price(0)
                    .build()
                    .unwrap(),
            )
            .unwrap();
        assert!(result.is_success(), "tx failed: {result:?}");
        transfer_logs(&evm.inspector.get_events())
    }

    /// CALL with value into a code-bearing contract (new-frame path).
    #[test]
    fn test_eip7708_nested_call_transfer_log_is_inspected() {
        let mut code = vec![
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x07,
            opcode::PUSH20,
        ];
        code.extend_from_slice(CALLEE.as_slice());
        code.extend_from_slice(&[opcode::PUSH2, 0xFF, 0xFF, opcode::CALL, opcode::STOP]);
        let logs = run_transfer_log_probe(
            code,
            Some(Bytecode::new_legacy(Bytes::from(vec![opcode::STOP]))),
        );
        assert_eq!(
            logs,
            vec![(BENCH_TARGET, CALLEE, U256::from(7))],
            "nested CALL transfer log missing"
        );
    }

    /// CALL with value to an account with no code (immediate-result path).
    #[test]
    fn test_eip7708_call_to_eoa_transfer_log_is_inspected() {
        let mut code = vec![
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x07,
            opcode::PUSH20,
        ];
        code.extend_from_slice(CALLEE.as_slice());
        code.extend_from_slice(&[opcode::PUSH2, 0xFF, 0xFF, opcode::CALL, opcode::STOP]);
        let logs = run_transfer_log_probe(code, None);
        assert_eq!(
            logs,
            vec![(BENCH_TARGET, CALLEE, U256::from(7))],
            "EOA CALL transfer log missing"
        );
    }

    /// CALL with value to a precompile (precompile path).
    #[test]
    fn test_eip7708_call_to_precompile_transfer_log_is_inspected() {
        let identity = address!("0000000000000000000000000000000000000004");
        let mut code = vec![
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x07,
            opcode::PUSH20,
        ];
        code.extend_from_slice(identity.as_slice());
        code.extend_from_slice(&[opcode::PUSH2, 0xFF, 0xFF, opcode::CALL, opcode::STOP]);
        let logs = run_transfer_log_probe(code, None);
        assert_eq!(
            logs,
            vec![(BENCH_TARGET, identity, U256::from(7))],
            "precompile CALL transfer log missing"
        );
    }

    /// CREATE with value (new-frame path via create_account_checkpoint).
    #[test]
    fn test_eip7708_create_transfer_log_is_inspected() {
        // store a single STOP byte at memory[0], then CREATE(value=7, offset=0, size=1)
        let code = vec![
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x00,
            opcode::MSTORE8,
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x00,
            opcode::PUSH1,
            0x07,
            opcode::CREATE,
            opcode::STOP,
        ];
        let logs = run_transfer_log_probe(code, None);
        assert_eq!(logs.len(), 1, "CREATE transfer log missing: {logs:?}");
        assert_eq!(logs[0].0, BENCH_TARGET);
        assert_eq!(logs[0].2, U256::from(7));
    }

    /// Inspector answers to frames, built with `CallOutcome::new_answer` and
    /// `CreateOutcome::new_answer`, under EIP-8037.
    mod answered_frames {
        use crate::{InspectEvm, Inspector};
        use context::{result::ExecutionResult, CfgEnv, Context, ContextTr, JournalTr, TxEnv};
        use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
        use handler::{FrameResult, MainBuilder, MainContext};
        use interpreter::{
            interpreter_types::Jumps, CallInputs, CallOutcome, CreateInputs, CreateOutcome,
            FrameInput, Gas, InstructionResult, Interpreter, InterpreterResult, InterpreterTypes,
        };
        use primitives::{
            address, eip2780, eip8037, eip8038, hardfork::SpecId, Address, Bytes, TxKind, U256,
        };
        use state::{bytecode::opcode, Bytecode, EvmState};

        /// Absent account whose calls the [`AnsweringInspector`] answers. A
        /// value call to it is charged the new-account state gas upfront.
        const ANSWERED: Address = address!("a000000000000000000000000000000000000001");

        /// Regular gas cap of the transactions. The gas limit above it is the
        /// state gas reservoir the transaction starts with.
        const REGULAR_GAS_CAP: u64 = 1_000_000;

        /// State gas reservoir the transactions start with, unless a test
        /// says otherwise.
        const RESERVOIR: u64 = 5_000_000;

        /// State gas charged upfront for a value call into an empty account,
        /// and for a creation at an empty address.
        const NEW_ACCOUNT_STATE_GAS: u64 = eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;

        /// Gas the `CALL` in [`call_to_answered`] forwards.
        const CALL_GAS: u64 = 100_000;

        /// Stipend a value call receives on top of the gas it forwards.
        const CALL_STIPEND: u64 = 2_300;

        /// How the [`AnsweringInspector`] builds its answers.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Build {
            /// With [`CallOutcome::new_answer`] and [`CreateOutcome::new_answer`].
            NewAnswer,
            /// With `Gas::new(gas_limit)` and the plain `new` constructors.
            GasNew,
        }

        /// Answers every call to [`ANSWERED`] and every creation without
        /// running them, and records the gas before each instruction.
        ///
        /// A successful answer to a call with value moves the value itself, as
        /// a mocked call does.
        struct AnsweringInspector {
            build: Build,
            result: InstructionResult,
            /// Address a creation answer reports.
            created_address: Option<Address>,
            /// Answers from `frame_start` instead of `call` and `create`.
            from_frame_start: bool,
            /// `(opcode, remaining, reservoir)` before each instruction.
            steps: Vec<(u8, u64, u64)>,
            /// Gas limit of each answered frame.
            answered_gas_limits: Vec<u64>,
            /// For each answer, whether it equals the one built with `Gas::new`.
            equals_gas_new: Vec<bool>,
        }

        impl AnsweringInspector {
            fn new(build: Build, result: InstructionResult) -> Self {
                Self {
                    build,
                    result,
                    created_address: None,
                    from_frame_start: false,
                    steps: Vec::new(),
                    answered_gas_limits: Vec::new(),
                    equals_gas_new: Vec::new(),
                }
            }

            fn answer_call<CTX: ContextTr>(
                &mut self,
                context: &mut CTX,
                inputs: &CallInputs,
            ) -> Option<CallOutcome> {
                if inputs.target_address != ANSWERED {
                    return None;
                }
                self.answered_gas_limits.push(inputs.gas_limit);
                if self.result.is_ok() {
                    if let Some(value) = inputs.transfer_value() {
                        let error = context.journal_mut().transfer_loaded(
                            inputs.caller,
                            inputs.target_address,
                            value,
                        );
                        assert_eq!(error, None);
                    }
                }
                let answer = CallOutcome::new_answer(inputs, self.result, Bytes::new());
                let with_gas_new = CallOutcome::new(
                    InterpreterResult::new(self.result, Bytes::new(), Gas::new(inputs.gas_limit)),
                    inputs.return_memory_offset.clone(),
                );
                self.equals_gas_new.push(answer == with_gas_new);
                Some(match self.build {
                    Build::NewAnswer => answer,
                    Build::GasNew => with_gas_new,
                })
            }

            fn answer_create(&mut self, inputs: &CreateInputs) -> CreateOutcome {
                self.answered_gas_limits.push(inputs.gas_limit());
                let answer = CreateOutcome::new_answer(
                    inputs,
                    self.result,
                    Bytes::new(),
                    self.created_address,
                );
                let with_gas_new = CreateOutcome::new(
                    InterpreterResult::new(self.result, Bytes::new(), Gas::new(inputs.gas_limit())),
                    self.created_address,
                );
                self.equals_gas_new.push(answer == with_gas_new);
                match self.build {
                    Build::NewAnswer => answer,
                    Build::GasNew => with_gas_new,
                }
            }

            /// `(remaining, reservoir)` before the first `opcode` and before
            /// the instruction that follows it.
            fn gas_around(&self, opcode: u8) -> ((u64, u64), (u64, u64)) {
                let i = self
                    .steps
                    .iter()
                    .position(|(op, ..)| *op == opcode)
                    .expect("opcode was executed");
                let (_, remaining, reservoir) = self.steps[i];
                let (_, next_remaining, next_reservoir) = self.steps[i + 1];
                ((remaining, reservoir), (next_remaining, next_reservoir))
            }
        }

        impl<CTX: ContextTr, INTR: InterpreterTypes> Inspector<CTX, INTR> for AnsweringInspector {
            fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
                self.steps.push((
                    interp.bytecode.opcode(),
                    interp.gas.remaining(),
                    interp.gas.reservoir(),
                ));
            }

            fn frame_start(
                &mut self,
                context: &mut CTX,
                frame_input: &mut FrameInput,
            ) -> Option<FrameResult> {
                if !self.from_frame_start {
                    return None;
                }
                match frame_input {
                    FrameInput::Call(inputs) => {
                        self.answer_call(context, inputs).map(FrameResult::Call)
                    }
                    FrameInput::Create(inputs) => {
                        Some(FrameResult::Create(self.answer_create(inputs)))
                    }
                    FrameInput::Empty => None,
                }
            }

            fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
                if self.from_frame_start {
                    return None;
                }
                self.answer_call(context, inputs)
            }

            fn create(
                &mut self,
                _context: &mut CTX,
                inputs: &mut CreateInputs,
            ) -> Option<CreateOutcome> {
                if self.from_frame_start {
                    return None;
                }
                Some(self.answer_create(inputs))
            }
        }

        /// Runs a transaction of `kind` with `value` under `spec`, with `code`
        /// at [`BENCH_TARGET`]. Under Amsterdam the transaction starts with a
        /// state gas reservoir of `reservoir`.
        fn run(
            spec: SpecId,
            code: Vec<u8>,
            kind: TxKind,
            value: U256,
            reservoir: u64,
            inspector: AnsweringInspector,
        ) -> (ExecutionResult, EvmState, AnsweringInspector) {
            let mut cfg = CfgEnv::new_with_spec(spec);
            cfg.tx_gas_limit_cap = Some(REGULAR_GAS_CAP);
            let ctx = Context::mainnet()
                .with_cfg(cfg)
                .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(code.into())));
            let mut evm = ctx.build_mainnet_with_inspector(inspector);
            let output = evm
                .inspect_tx(
                    TxEnv::builder()
                        .caller(BENCH_CALLER)
                        .kind(kind)
                        .value(value)
                        .gas_limit(REGULAR_GAS_CAP + reservoir)
                        .gas_price(0)
                        .build()
                        .unwrap(),
                )
                .unwrap();
            (output.result, output.state, evm.inspector)
        }

        /// Runs an Amsterdam transaction to [`BENCH_TARGET`] running `code`.
        fn run_code(
            code: Vec<u8>,
            reservoir: u64,
            inspector: AnsweringInspector,
        ) -> (ExecutionResult, EvmState, AnsweringInspector) {
            run(
                SpecId::AMSTERDAM,
                code,
                TxKind::Call(BENCH_TARGET),
                U256::ZERO,
                reservoir,
                inspector,
            )
        }

        /// `CALL(CALL_GAS, ANSWERED, value, 0, 0, 0, 0); POP; STOP`
        fn call_to_answered(value: u8) -> Vec<u8> {
            let mut code = vec![
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                value,
                opcode::PUSH20,
            ];
            code.extend_from_slice(ANSWERED.as_slice());
            code.push(opcode::PUSH3);
            code.extend_from_slice(&CALL_GAS.to_be_bytes()[5..]);
            code.extend_from_slice(&[opcode::CALL, opcode::POP, opcode::STOP]);
            code
        }

        /// `CREATE(0, 0, 0); POP; STOP`
        fn create_code() -> Vec<u8> {
            vec![
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                0x00,
                opcode::PUSH1,
                0x00,
                opcode::CREATE,
                opcode::POP,
                opcode::STOP,
            ]
        }

        /// Regular gas the `CALL` of [`call_to_answered`] costs its caller when
        /// the call returns all its gas: a cold account access, plus for a
        /// value call the account write of the transfer (the stipend part of
        /// the transfer cost comes back with the call's gas).
        fn call_cost(value: u8) -> u64 {
            let transfer = if value == 0 {
                0
            } else {
                eip8038::ACCOUNT_WRITE
            };
            eip8038::COLD_ACCOUNT_ACCESS + transfer
        }

        /// Regular gas of the transaction running [`call_to_answered`] when the
        /// call returns all its gas: the intrinsic gas of a call without value
        /// to an existing account, 23 for the pushes and the `POP`, and the
        /// `CALL`.
        fn call_tx_regular_gas(value: u8) -> u64 {
            eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + 23 + call_cost(value)
        }

        /// An answer built with the constructor keeps the caller's reservoir,
        /// whatever its result. An answer built with `Gas::new` loses it.
        #[test]
        fn test_answer_keeps_caller_reservoir() {
            let regular = call_tx_regular_gas(0);
            for (result, consumed) in [
                (InstructionResult::Stop, 0),
                (InstructionResult::Revert, 0),
                (InstructionResult::OutOfGas, CALL_GAS),
            ] {
                let (outcome, _, inspector) = run_code(
                    call_to_answered(0),
                    RESERVOIR,
                    AnsweringInspector::new(Build::NewAnswer, result),
                );
                let ((_, before), (_, after)) = inspector.gas_around(opcode::CALL);
                assert_eq!(inspector.answered_gas_limits, [CALL_GAS], "{result:?}");
                assert_eq!(before, RESERVOIR, "{result:?}");
                assert_eq!(after, RESERVOIR, "{result:?}");
                assert!(outcome.is_success(), "{result:?}: {outcome:?}");
                assert_eq!(
                    outcome.gas().total_gas_spent(),
                    regular + consumed,
                    "{result:?}"
                );
                assert_eq!(outcome.gas().state_gas_spent_final(), 0, "{result:?}");
            }

            let (outcome, _, inspector) = run_code(
                call_to_answered(0),
                RESERVOIR,
                AnsweringInspector::new(Build::GasNew, InstructionResult::Stop),
            );
            let ((_, before), (_, after)) = inspector.gas_around(opcode::CALL);
            assert_eq!(before, RESERVOIR);
            assert_eq!(after, 0, "the caller adopts the answer's empty reservoir");
            assert_eq!(outcome.gas().total_gas_spent(), regular + RESERVOIR);
        }

        /// A value call into an absent account is charged the new-account
        /// state gas upfront. A failed answer gets the charge refunded. A
        /// successful one, which moved the value itself, keeps it.
        #[test]
        fn test_answer_to_value_call_into_absent_account() {
            let regular = call_tx_regular_gas(1);
            for (result, charged, consumed) in [
                (InstructionResult::Stop, NEW_ACCOUNT_STATE_GAS, 0),
                (InstructionResult::Revert, 0, 0),
                (InstructionResult::OutOfGas, 0, CALL_GAS + CALL_STIPEND),
            ] {
                let (outcome, state, inspector) = run_code(
                    call_to_answered(1),
                    RESERVOIR,
                    AnsweringInspector::new(Build::NewAnswer, result),
                );
                let ((_, before), (_, after)) = inspector.gas_around(opcode::CALL);
                assert_eq!(
                    inspector.answered_gas_limits,
                    [CALL_GAS + CALL_STIPEND],
                    "{result:?}"
                );
                assert_eq!(before, RESERVOIR, "{result:?}");
                assert_eq!(after, RESERVOIR - charged, "{result:?}");
                assert!(outcome.is_success(), "{result:?}: {outcome:?}");
                assert_eq!(
                    outcome.gas().total_gas_spent(),
                    regular + charged + consumed,
                    "{result:?}"
                );
                assert_eq!(outcome.gas().state_gas_spent_final(), charged, "{result:?}");
                // Only the successful answer moved the value.
                let balance = state.get(&ANSWERED).map(|account| account.info.balance);
                assert_eq!(
                    balance.is_some_and(|balance| balance == U256::from(1)),
                    result.is_ok(),
                    "{result:?}"
                );
            }
        }

        /// A creation at an absent address is charged the account-creation
        /// state gas upfront. A failed answer, or one without an address, gets
        /// the charge refunded. A successful one with an address keeps it.
        #[test]
        fn test_answer_to_create() {
            // Intrinsic gas of a call without value to an existing account,
            // the creation's access cost, and 11 for the pushes and the `POP`.
            let regular =
                eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + eip8038::CREATE_ACCESS + 11;
            let created = Some(BENCH_TARGET.create(1));
            for (result, address, charged, halts) in [
                (
                    InstructionResult::Return,
                    created,
                    NEW_ACCOUNT_STATE_GAS,
                    false,
                ),
                (InstructionResult::Return, None, 0, false),
                (InstructionResult::Revert, created, 0, false),
                (InstructionResult::Revert, None, 0, false),
                (InstructionResult::OutOfGas, None, 0, true),
            ] {
                let case = format!("{result:?} {address:?}");
                let (outcome, _, inspector) = run_code(
                    create_code(),
                    RESERVOIR,
                    AnsweringInspector {
                        created_address: address,
                        ..AnsweringInspector::new(Build::NewAnswer, result)
                    },
                );
                let ((remaining, before), (_, after)) = inspector.gas_around(opcode::CREATE);
                // All but one 64th of the remaining gas, after the access cost.
                let gas_limit = remaining - eip8038::CREATE_ACCESS;
                let gas_limit = gas_limit - gas_limit / 64;
                assert_eq!(inspector.answered_gas_limits, [gas_limit], "{case}");
                let consumed = if halts { gas_limit } else { 0 };
                assert_eq!(before, RESERVOIR, "{case}");
                assert_eq!(after, RESERVOIR - charged, "{case}");
                assert!(outcome.is_success(), "{case}: {outcome:?}");
                assert_eq!(
                    outcome.gas().total_gas_spent(),
                    regular + charged + consumed,
                    "{case}"
                );
                assert_eq!(outcome.gas().state_gas_spent_final(), charged, "{case}");
            }
        }

        /// The transaction's own frame, answered from `frame_start`, settles
        /// the same way: the sender is not billed for the reservoir, and the
        /// upfront state gas of a value transfer into an absent account, or of
        /// a creation at an absent address, is refunded unless the answer
        /// succeeds (with an address, for a creation).
        #[test]
        fn test_answer_to_transaction_frame() {
            // Intrinsic gas of a value transfer to another account.
            let transfer_intrinsic =
                eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + eip2780::TX_VALUE_COST;
            // Intrinsic gas of a creation with empty init code.
            let create_intrinsic = eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS;
            let created = Some(BENCH_CALLER.create(0));
            for (kind, address, result, intrinsic, charged) in [
                (
                    TxKind::Call(ANSWERED),
                    None,
                    InstructionResult::Stop,
                    transfer_intrinsic,
                    NEW_ACCOUNT_STATE_GAS,
                ),
                (
                    TxKind::Call(ANSWERED),
                    None,
                    InstructionResult::Revert,
                    transfer_intrinsic,
                    0,
                ),
                (
                    TxKind::Call(ANSWERED),
                    None,
                    InstructionResult::OutOfGas,
                    transfer_intrinsic,
                    0,
                ),
                (
                    TxKind::Create,
                    created,
                    InstructionResult::Return,
                    create_intrinsic,
                    NEW_ACCOUNT_STATE_GAS,
                ),
                (
                    TxKind::Create,
                    None,
                    InstructionResult::Return,
                    create_intrinsic,
                    0,
                ),
                (
                    TxKind::Create,
                    created,
                    InstructionResult::Revert,
                    create_intrinsic,
                    0,
                ),
            ] {
                let case = format!("{kind:?} {result:?} {address:?}");
                let (outcome, _, inspector) = run(
                    SpecId::AMSTERDAM,
                    Vec::new(),
                    kind,
                    U256::from(1),
                    RESERVOIR,
                    AnsweringInspector {
                        created_address: address,
                        from_frame_start: true,
                        ..AnsweringInspector::new(Build::NewAnswer, result)
                    },
                );
                assert_eq!(inspector.answered_gas_limits.len(), 1, "{case}");
                // A halt consumes all regular gas.
                let total = if result.is_halt() {
                    REGULAR_GAS_CAP
                } else {
                    intrinsic + charged
                };
                assert_eq!(outcome.is_success(), result.is_ok(), "{case}: {outcome:?}");
                assert_eq!(outcome.gas().total_gas_spent(), total, "{case}");
                assert_eq!(outcome.gas().state_gas_spent_final(), charged, "{case}");
            }
        }

        /// When the caller's reservoir does not cover the upfront charge, the
        /// rest spills into its regular gas and the call inherits no
        /// reservoir. A failed answer gets the spilled part back as regular
        /// gas and the rest as reservoir. A successful one keeps both.
        #[test]
        fn test_answer_with_upfront_charge_spilling_into_regular_gas() {
            let regular = call_tx_regular_gas(1);
            for reservoir in [0, NEW_ACCOUNT_STATE_GAS / 2] {
                let spilled = NEW_ACCOUNT_STATE_GAS - reservoir;
                for (result, charged) in [
                    (InstructionResult::Stop, NEW_ACCOUNT_STATE_GAS),
                    (InstructionResult::Revert, 0),
                ] {
                    let case = format!("reservoir {reservoir} {result:?}");
                    let (outcome, _, inspector) = run_code(
                        call_to_answered(1),
                        reservoir,
                        AnsweringInspector::new(Build::NewAnswer, result),
                    );
                    let ((remaining_before, before), (remaining_after, after)) =
                        inspector.gas_around(opcode::CALL);
                    let kept_spill = if result.is_ok() { spilled } else { 0 };
                    assert_eq!(before, reservoir, "{case}");
                    assert_eq!(after, if result.is_ok() { 0 } else { reservoir }, "{case}");
                    assert_eq!(
                        remaining_before - remaining_after,
                        call_cost(1) + kept_spill,
                        "{case}"
                    );
                    assert!(outcome.is_success(), "{case}: {outcome:?}");
                    assert_eq!(outcome.gas().total_gas_spent(), regular + charged, "{case}");
                    assert_eq!(outcome.gas().state_gas_spent_final(), charged, "{case}");
                }
            }
        }

        /// Before Amsterdam a frame inherits no reservoir and is charged no
        /// state gas upfront, so an answer built with the constructor equals
        /// one built with `Gas::new`.
        #[test]
        fn test_answer_before_amsterdam_equals_gas_new() {
            // A value call into an absent account, then a creation.
            let mut code = call_to_answered(1);
            code.pop();
            code.extend_from_slice(&create_code());
            for result in [InstructionResult::Stop, InstructionResult::Revert] {
                let (outcome, _, inspector) = run(
                    SpecId::OSAKA,
                    code.clone(),
                    TxKind::Call(BENCH_TARGET),
                    U256::ZERO,
                    0,
                    AnsweringInspector {
                        created_address: Some(BENCH_TARGET.create(1)),
                        ..AnsweringInspector::new(Build::NewAnswer, result)
                    },
                );
                assert!(outcome.is_success(), "{result:?}: {outcome:?}");
                assert_eq!(inspector.equals_gas_new, [true, true], "{result:?}");
            }
        }
    }
}
