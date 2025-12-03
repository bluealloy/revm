//! CountInspector - Inspector that counts all opcodes that were called.
use crate::inspector::Inspector;
use interpreter::{interpreter_types::Jumps, InterpreterTypes};
use primitives::Log;

/// Inspector that counts all opcodes that were called during execution.
#[derive(Clone, Debug)]
pub struct CountInspector {
    /// Fixed array keyed by opcode value to count executions.
    opcode_counts: [u64; 256],
    /// Count of initialize_interp calls.
    initialize_interp_count: u64,
    /// Count of step calls.
    step_count: u64,
    /// Count of step_end calls.
    step_end_count: u64,
    /// Count of log calls.
    log_count: u64,
    /// Count of call calls.
    call_count: u64,
    /// Count of call_end calls.
    call_end_count: u64,
    /// Count of create calls.
    create_count: u64,
    /// Count of create_end calls.
    create_end_count: u64,
    /// Count of selfdestruct calls.
    selfdestruct_count: u64,
}

impl Default for CountInspector {
    fn default() -> Self {
        Self {
            opcode_counts: [0; 256],
            initialize_interp_count: 0,
            step_count: 0,
            step_end_count: 0,
            log_count: 0,
            call_count: 0,
            call_end_count: 0,
            create_count: 0,
            create_end_count: 0,
            selfdestruct_count: 0,
        }
    }
}

impl CountInspector {
    /// Create a new CountInspector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the count for a specific opcode.
    pub fn get_count(&self, opcode: u8) -> u64 {
        self.opcode_counts[opcode as usize]
    }

    /// Get a reference to all opcode counts.
    pub fn opcode_counts(&self) -> &[u64; 256] {
        &self.opcode_counts
    }

    /// Get the total number of opcodes executed.
    pub fn total_opcodes(&self) -> u64 {
        self.opcode_counts.iter().copied().sum()
    }

    /// Get the number of unique opcodes executed.
    pub fn unique_opcodes(&self) -> usize {
        self.opcode_counts
            .iter()
            .filter(|&&count| count > 0)
            .count()
    }

    /// Clear all counts.
    pub fn clear(&mut self) {
        self.opcode_counts = [0; 256];
        self.initialize_interp_count = 0;
        self.step_count = 0;
        self.step_end_count = 0;
        self.log_count = 0;
        self.call_count = 0;
        self.call_end_count = 0;
        self.create_count = 0;
        self.create_end_count = 0;
        self.selfdestruct_count = 0;
    }

    /// Get the count of initialize_interp calls.
    pub fn initialize_interp_count(&self) -> u64 {
        self.initialize_interp_count
    }

    /// Get the count of step calls.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Get the count of step_end calls.
    pub fn step_end_count(&self) -> u64 {
        self.step_end_count
    }

    /// Get the count of log calls.
    pub fn log_count(&self) -> u64 {
        self.log_count
    }

    /// Get the count of call calls.
    pub fn call_count(&self) -> u64 {
        self.call_count
    }

    /// Get the count of call_end calls.
    pub fn call_end_count(&self) -> u64 {
        self.call_end_count
    }

    /// Get the count of create calls.
    pub fn create_count(&self) -> u64 {
        self.create_count
    }

    /// Get the count of create_end calls.
    pub fn create_end_count(&self) -> u64 {
        self.create_end_count
    }

    /// Get the count of selfdestruct calls.
    pub fn selfdestruct_count(&self) -> u64 {
        self.selfdestruct_count
    }
}

impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for CountInspector {
    fn initialize_interp(
        &mut self,
        _interp: &mut interpreter::Interpreter<INTR>,
        _context: &mut CTX,
    ) {
        self.initialize_interp_count += 1;
    }

    fn step(&mut self, interp: &mut interpreter::Interpreter<INTR>, _context: &mut CTX) {
        self.step_count += 1;
        let opcode = interp.bytecode.opcode();
        self.opcode_counts[opcode as usize] += 1;
    }

    fn step_end(&mut self, _interp: &mut interpreter::Interpreter<INTR>, _context: &mut CTX) {
        self.step_end_count += 1;
    }

    fn log(&mut self, _context: &mut CTX, _log: Log) {
        self.log_count += 1;
    }

    fn call(
        &mut self,
        _context: &mut CTX,
        _inputs: &mut interpreter::CallInputs,
    ) -> Option<interpreter::CallOutcome> {
        self.call_count += 1;
        None
    }

    fn call_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &interpreter::CallInputs,
        _outcome: &mut interpreter::CallOutcome,
    ) {
        self.call_end_count += 1;
    }

    fn create(
        &mut self,
        _context: &mut CTX,
        _inputs: &mut interpreter::CreateInputs,
    ) -> Option<interpreter::CreateOutcome> {
        self.create_count += 1;
        None
    }

    fn create_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &interpreter::CreateInputs,
        _outcome: &mut interpreter::CreateOutcome,
    ) {
        self.create_end_count += 1;
    }

    fn selfdestruct(
        &mut self,
        _contract: primitives::Address,
        _target: primitives::Address,
        _value: primitives::U256,
    ) {
        self.selfdestruct_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InspectEvm;
    use context::Context;
    use database::BenchmarkDB;
    use handler::{MainBuilder, MainContext};
    use primitives::{Bytes, TxKind};
    use state::bytecode::{opcode, Bytecode};

    #[test]
    fn test_count_inspector() {
        // Create simple bytecode that just adds two numbers and stops
        let contract_data: Bytes = Bytes::from(vec![
            opcode::PUSH1,
            0x10, // 0: PUSH1 16
            opcode::PUSH1,
            0x20,         // 2: PUSH1 32
            opcode::ADD,  // 4: ADD
            opcode::DUP1, // 5: DUP1 (duplicate the result)
            opcode::PUSH1,
            0x00,           // 6: PUSH1 0
            opcode::MSTORE, // 8: MSTORE (store result in memory)
            opcode::STOP,   // 9: STOP
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode.clone()));
        let mut count_inspector = CountInspector::new();

        let mut evm = ctx.build_mainnet_with_inspector(&mut count_inspector);

        // Execute the contract
        evm.inspect_one_tx(
            context::TxEnv::builder()
                .kind(TxKind::Call(database::BENCH_TARGET))
                .gas_limit(30000)
                .build()
                .unwrap(),
        )
        .unwrap();

        // Check opcode counts
        assert_eq!(count_inspector.get_count(opcode::PUSH1), 3);
        assert_eq!(count_inspector.get_count(opcode::ADD), 1);
        assert_eq!(count_inspector.get_count(opcode::DUP1), 1);
        assert_eq!(count_inspector.get_count(opcode::MSTORE), 1);
        assert_eq!(count_inspector.get_count(opcode::STOP), 1);

        // Check totals
        assert_eq!(count_inspector.total_opcodes(), 7);
        assert_eq!(count_inspector.unique_opcodes(), 5);

        // Check inspector function counts
        assert_eq!(count_inspector.initialize_interp_count(), 1);
        assert_eq!(count_inspector.step_count(), 7); // Each opcode triggers a step
        assert_eq!(count_inspector.step_end_count(), 7); // Each opcode triggers a step_end
        assert_eq!(count_inspector.log_count(), 0); // No LOG opcodes
        assert_eq!(count_inspector.call_count(), 1); // The transaction itself is a call
        assert_eq!(count_inspector.call_end_count(), 1);
        assert_eq!(count_inspector.create_count(), 0); // No CREATE opcodes
        assert_eq!(count_inspector.create_end_count(), 0);
        assert_eq!(count_inspector.selfdestruct_count(), 0); // No SELFDESTRUCT opcodes
    }

    #[test]
    fn test_count_inspector_clear() {
        let mut inspector = CountInspector::new();

        // Add some counts manually for testing
        inspector.opcode_counts[opcode::PUSH1 as usize] += 5;
        inspector.opcode_counts[opcode::ADD as usize] += 3;
        inspector.initialize_interp_count = 2;
        inspector.step_count = 10;
        inspector.step_end_count = 10;
        inspector.log_count = 1;
        inspector.call_count = 3;
        inspector.call_end_count = 3;
        inspector.create_count = 1;
        inspector.create_end_count = 1;
        inspector.selfdestruct_count = 1;

        assert_eq!(inspector.total_opcodes(), 8);
        assert_eq!(inspector.unique_opcodes(), 2);
        assert_eq!(inspector.initialize_interp_count(), 2);
        assert_eq!(inspector.step_count(), 10);

        // Clear and verify
        inspector.clear();
        assert_eq!(inspector.total_opcodes(), 0);
        assert_eq!(inspector.unique_opcodes(), 0);
        assert_eq!(inspector.initialize_interp_count(), 0);
        assert_eq!(inspector.step_count(), 0);
        assert_eq!(inspector.step_end_count(), 0);
        assert_eq!(inspector.log_count(), 0);
        assert_eq!(inspector.call_count(), 0);
        assert_eq!(inspector.call_end_count(), 0);
        assert_eq!(inspector.create_count(), 0);
        assert_eq!(inspector.create_end_count(), 0);
        assert_eq!(inspector.selfdestruct_count(), 0);
        assert!(inspector.opcode_counts().iter().all(|&count| count == 0));
    }

    #[test]
    fn test_count_inspector_with_logs() {
        // Create bytecode that emits a log
        let contract_data: Bytes = Bytes::from(vec![
            opcode::PUSH1,
            0x20, // 0: PUSH1 32 (length)
            opcode::PUSH1,
            0x00,         // 2: PUSH1 0 (offset)
            opcode::LOG0, // 4: LOG0 - emit log with no topics
            opcode::STOP, // 5: STOP
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(bytecode.clone()));
        let mut count_inspector = CountInspector::new();

        let mut evm = ctx.build_mainnet_with_inspector(&mut count_inspector);

        // Execute the contract
        evm.inspect_one_tx(
            context::TxEnv::builder()
                .kind(TxKind::Call(database::BENCH_TARGET))
                .gas_limit(30000)
                .build()
                .unwrap(),
        )
        .unwrap();

        // Check that log was counted
        assert_eq!(count_inspector.log_count(), 1);
        assert_eq!(count_inspector.step_count(), 4); // 2 PUSH1 + LOG0 + STOP
    }
}
