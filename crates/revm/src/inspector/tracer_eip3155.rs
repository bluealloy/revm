//! Inspector that support tracing of EIP-3155 https://eips.ethereum.org/EIPS/eip-3155

use crate::inspectors::GasInspector;
use crate::interpreter::{CallInputs, CreateInputs, Gas, InstructionResult};
use crate::primitives;
use crate::primitives::{db::Database, hex, Bytes, B160};
use crate::{evm_impl::EVMData, Inspector};
use ethers_core::types::{H160, H256};
use ethers_core::utils::rlp::{self, RlpStream};
use plain_hasher::PlainHasher;
use revm_interpreter::primitives::{Account, SpecId, B256, U256};
use revm_interpreter::{opcode, Interpreter, Memory, Stack};
use serde_json::json;
use sha3::{Digest, Keccak256};
use std::io::Write;
use triehash::sec_trie_root;

pub struct TracerEip3155 {
    output: Box<dyn Write>,
    gas_inspector: GasInspector,

    #[allow(dead_code)]
    trace_mem: bool,
    #[allow(dead_code)]
    trace_return_data: bool,

    stack: Stack,
    pc: usize,
    opcode: u8,
    gas: u64,
    mem_size: usize,
    #[allow(dead_code)]
    memory: Option<Memory>,
    skip: bool,
}

impl TracerEip3155 {
    pub fn new(output: Box<dyn Write>, trace_mem: bool, trace_return_data: bool) -> Self {
        Self {
            output,
            gas_inspector: GasInspector::default(),
            trace_mem,
            trace_return_data,
            stack: Stack::new(),
            pc: 0,
            opcode: 0,
            gas: 0,
            mem_size: 0,
            memory: None,
            skip: false,
        }
    }
}

impl<DB: Database> Inspector<DB> for TracerEip3155 {
    fn initialize_interp(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        self.gas_inspector.initialize_interp(interp, data);
        InstructionResult::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(&mut self, interp: &mut Interpreter, data: &mut EVMData<'_, DB>) -> InstructionResult {
        self.gas_inspector.step(interp, data);
        self.stack = interp.stack.clone();
        self.pc = interp.program_counter();
        self.opcode = interp.current_opcode();
        self.mem_size = interp.memory.len();
        self.gas = self.gas_inspector.gas_remaining();
        //
        InstructionResult::Continue
    }

    fn step_end(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        eval: InstructionResult,
    ) -> InstructionResult {
        self.gas_inspector.step_end(interp, data, eval);
        if self.skip {
            self.skip = false;
            return InstructionResult::Continue;
        };

        self.print_log_line(data.journaled_state.depth());
        InstructionResult::Continue
    }

    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        _inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        self.print_log_line(data.journaled_state.depth());
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CallInputs,
        remaining_gas: Gas,
        ret: InstructionResult,
        out: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        self.gas_inspector
            .call_end(data, inputs, remaining_gas, ret, out.clone());
        // self.log_step(interp, data, is_static, eval);
        let is_legacy = !SpecId::enabled(data.env.cfg.spec_id, primitives::SpecId::SPURIOUS_DRAGON);

        self.skip = true;
        if data.journaled_state.depth() == 0 {
            let state_root = state_merkle_trie_root(
                data.journaled_state
                    .state
                    .iter()
                    .filter(|(_address, acc)| account_is_part_of_trie(acc, is_legacy))
                    .map(|(k, v)| (*k, v.clone())),
            );
            let log_line = json!({
                "stateRoot": format!("0x{state_root:x}"),
                "output": format!("{out:?}"),
                "gasUser": format!("0x{:x}", self.gas_inspector.gas_remaining()),
                //time
                //fork
            });

            writeln!(
                self.output,
                "{:?}",
                serde_json::to_string(&log_line).unwrap()
            )
            .expect("If output fails we can ignore the logging");
        }
        (ret, remaining_gas, out)
    }

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<B160>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<B160>, Gas, Bytes) {
        self.gas_inspector
            .create_end(data, inputs, ret, address, remaining_gas, out.clone());
        (ret, address, remaining_gas, out)
    }
}

impl TracerEip3155 {
    fn print_log_line(&mut self, depth: u64) {
        let short_stack: Vec<String> = self.stack.data().iter().map(|&b| short_hex(b)).collect();
        let log_line = json!({
            "pc": self.pc,
            "op": self.opcode,
            "gas": format!("0x{:x}", self.gas),
            "gasCost": format!("0x{:x}", self.gas_inspector.last_gas_cost()),
            //memory?
            "memSize": self.mem_size,
            "stack": short_stack,
            "depth": depth,
            //returnData
            //refund
            "opName": opcode::OPCODE_JUMPMAP[self.opcode as usize],
            //error
            //storage
            //returnStack
        });

        writeln!(self.output, "{}", serde_json::to_string(&log_line).unwrap())
            .expect("If output fails we can ignore the logging");
    }
}

fn short_hex(b: U256) -> String {
    let s = hex::encode(b.to_be_bytes_vec())
        .trim_start_matches('0')
        .to_string();
    if s.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{s}")
    }
}

// (&B160, &revm_interpreter::revm_primitives::Account)
pub fn state_merkle_trie_root(accounts: impl Iterator<Item = (B160, Account)>) -> B256 {
    let vec = accounts
        .map(|(address, info)| {
            let acc_root = trie_account_rlp(&info);
            (H160::from(address.0), acc_root)
        })
        .collect();

    trie_root(vec)
}

/// Returns the RLP for this account.
pub fn trie_account_rlp(acc: &Account) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&acc.info.nonce);
    stream.append(&acc.info.balance);
    stream.append(&{
        sec_trie_root::<KeccakHasher, _, _, _>(
            acc.storage
                .iter()
                .filter(|(_k, &ref v)| v.original_value != U256::ZERO)
                .map(|(&k, v)| (H256::from(k.to_be_bytes()), rlp::encode(&v.original_value))),
        )
    });
    stream.append(&acc.info.code_hash.0.as_ref());
    stream.out().freeze()
}

pub fn trie_root(acc_data: Vec<(H160, Bytes)>) -> B256 {
    B256(sec_trie_root::<KeccakHasher, _, _, _>(acc_data.into_iter()).0)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct KeccakHasher;

impl hash_db::Hasher for KeccakHasher {
    type Out = H256;
    type StdHasher = PlainHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        let out = Keccak256::digest(x);
        H256::from_slice(out.as_slice())
    }
}

/// Determines if an account should be included as part of the trie root calculation.
///
/// - Pre-spurious-dragon: existing and empty status were two separate states.
/// - Post-spurious-dragon: empty means non-existent.
fn account_is_part_of_trie(acc: &Account, is_legacy: bool) -> bool {
    // Pre-spurious-dragon: exists
    (is_legacy && !acc.is_loaded_as_not_existing())
    // Post-spurious-dragon: exists
    || !is_legacy && (!(acc.info.is_empty())
    // Pre- or post-spurious-dragon: does not exist.
    || !acc.info.exists())
}
