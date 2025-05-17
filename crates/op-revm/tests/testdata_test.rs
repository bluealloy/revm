mod test_utils;

use revm::{
    primitives::Bytes,
    context_interface::result::{ExecutionResult, HaltReason, ResultAndState, SuccessReason, Output},
    state::EvmState,
};

use crate::test_utils::compare_or_save_testdata;

/// Tests the testdata comparison functionality with a simple transaction
#[test]
fn test_basic_testdata_comparison() {
    // Create a minimal result and state
    let result: ResultAndState<HaltReason> = ResultAndState {
        result: ExecutionResult::Success {
            reason: SuccessReason::Stop,
            gas_used: 100,
            gas_refunded: 0,
            logs: vec![],
            output: Output::Call(Bytes::new()),
        },
        state: EvmState::default(),
    };

    // Use our testdata comparison utility to save or compare the output
    // The file path is relative to the tests/testdata directory
    compare_or_save_testdata("basic_test.json", &result).unwrap();
}

/// Tests the testdata comparison functionality with a revert result
#[test]
fn test_revert_testdata_comparison() {
    // Create a result with a revert
    let result: ResultAndState<HaltReason> = ResultAndState {
        result: ExecutionResult::Revert {
            gas_used: 100,
            output: Bytes::new(),
        },
        state: EvmState::default(),
    };

    // Use our testdata comparison utility
    compare_or_save_testdata("revert_test.json", &result).unwrap();
}

/// Tests the testdata comparison functionality with a halt result
#[test]
fn test_halt_testdata_comparison() {
    // Create a result with a halt
    let result: ResultAndState<HaltReason> = ResultAndState {
        result: ExecutionResult::Halt {
            reason: HaltReason::OpcodeNotFound,
            gas_used: 100,
        },
        state: EvmState::default(),
    };

    // Use our testdata comparison utility
    compare_or_save_testdata("halt_test.json", &result).unwrap();
} 
