//! Reentrant precompile — Approach A.
//!
//! Demonstrates a "callable precompile" that, while running, makes a **nested
//! EVM call** by re-entering the recursive frame driver
//! ([`Handler::run_exec_frame`]).
//!
//! ## Why it is done at the handler level
//!
//! The [`PrecompileProvider::run`] signature only receives `&mut Context` and
//! returns a single-shot result. It cannot make a synchronous sub-call, because
//! the sub-call's `frame_init` also needs `&mut precompiles` — and the provider
//! is already uniquely borrowed as `&mut self` for the whole `run`. That borrow
//! conflict is fundamental.
//!
//! The fix is to drive the reentrant call one level up, inside
//! [`Handler::run_exec_frame`], where we hold `&mut Evm` as a whole and the
//! precompile provider is only borrowed transiently by each `frame_init`. There
//! we can freely interleave: run the precompile logic, build a sub-call, push it
//! with `frame_init`, and recurse with `run_exec_frame` — exactly like a `CALL`
//! opcode, but originating from the precompile.
//!
//! `run_exec_loop` and `run_exec_frame` are overridable `Handler` methods, so
//! this needs **no core change** — only a custom handler.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use revm::{
    bytecode::Bytecode,
    context::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, Output},
        ContextSetters,
    },
    context_interface::{Cfg, ContextTr, JournalTr, LocalContextTr},
    database::InMemoryDB,
    handler::{
        evm::FrameTr, instructions::InstructionProvider, EvmTr, FrameResult, Handler, ItemOrResult,
        PrecompileProvider,
    },
    interpreter::{
        interpreter::EthInterpreter,
        interpreter_action::{FrameInit, FrameInput},
        CallInput, CallInputs, CallOutcome, CallScheme, CallValue, Gas, InstructionResult,
        InterpreterResult, SharedMemory,
    },
    primitives::{address, bytes, hardfork::SpecId, Address, Bytes, TxKind, U256},
    state::{AccountInfo, EvmState},
    Context, Database, MainBuilder, MainContext,
};

/// Address of the callable precompile. A `CALL` to this address is intercepted
/// by [`ReentrantHandler`] instead of being routed to bytecode/precompiles.
const CALLABLE_ADDRESS: Address = address!("0000000000000000000000000000000000000b0b");

/// Scratch address in whose context the nested bytecode runs.
const SCRATCH_ADDRESS: Address = address!("00000000000000000000000000000000000000cc");

/// Flat cost charged by the callable precompile before it forwards the rest of
/// its gas to the nested call.
const CALLABLE_BASE_COST: u64 = 100;

/// Bytecode executed by the nested call: returns the 32-byte value `42`.
///
/// `PUSH1 0x2a; PUSH1 0x00; MSTORE; PUSH1 0x20; PUSH1 0x00; RETURN`
const NESTED_BYTECODE: Bytes = bytes!("602a60005260206000f3");

/// Convenience alias matching the mainnet handler's error type.
type HandlerError<EVM> =
    EVMError<<<<EVM as EvmTr>::Context as ContextTr>::Db as Database>::Error, InvalidTransaction>;

/// A handler that adds a reentrant "callable precompile" at [`CALLABLE_ADDRESS`].
#[derive(Debug)]
struct ReentrantHandler<EVM> {
    _phantom: core::marker::PhantomData<EVM>,
}

impl<EVM> Default for ReentrantHandler<EVM> {
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM> ReentrantHandler<EVM>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
        Frame: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
    >,
{
    /// Returns `true` if this frame init targets the callable precompile.
    fn is_callable(init: &FrameInit) -> bool {
        matches!(&init.frame_input, FrameInput::Call(call) if call.bytecode_address == CALLABLE_ADDRESS)
    }

    /// Runs the callable precompile: charge a base cost, forward the remaining
    /// gas to a nested EVM call, and return that call's output.
    ///
    /// The nested call is executed by re-entering [`Handler::run_exec_frame`] —
    /// this is the recursion happening "inside the precompile".
    fn run_callable_precompile(
        &mut self,
        evm: &mut EVM,
        init: FrameInit,
    ) -> Result<FrameResult, HandlerError<EVM>> {
        let FrameInput::Call(call) = init.frame_input else {
            unreachable!("is_callable guarantees a Call input");
        };

        // Charge the precompile's own base cost against the incoming gas.
        let mut gas = Gas::new(call.gas_limit);
        if !gas.record_regular_cost(CALLABLE_BASE_COST) {
            return Ok(out_of_gas(call.gas_limit, call.return_memory_offset));
        }

        // Build the memory buffer for the child frame, mirroring `first_frame_input`.
        let memory = {
            let ctx = evm.ctx();
            let mut memory =
                SharedMemory::new_with_buffer(ctx.local().shared_memory_buffer().clone());
            memory.set_memory_limit(ctx.cfg().memory_limit());
            memory
        };

        // Build the nested call, forwarding the precompile's remaining gas.
        let nested_code = Bytecode::new_raw(NESTED_BYTECODE);
        let nested_hash = nested_code.hash_slow();
        let sub_call = CallInputs {
            input: CallInput::Bytes(Bytes::new()),
            return_memory_offset: 0..0,
            gas_limit: gas.remaining(),
            reservoir: 0,
            bytecode_address: SCRATCH_ADDRESS,
            known_bytecode: (nested_hash, nested_code),
            target_address: SCRATCH_ADDRESS,
            caller: CALLABLE_ADDRESS,
            // Apparent (not Transfer) so no value movement / account touch is needed.
            value: CallValue::Apparent(U256::ZERO),
            scheme: CallScheme::StaticCall,
            is_static: true,
            charged_new_account_state_gas: false,
        };
        let sub_init = FrameInit {
            depth: init.depth + 1,
            memory,
            frame_input: FrameInput::Call(Box::new(sub_call)),
        };

        // Push the child frame and run it to completion — recursively.
        let sub_result = match evm.frame_init(sub_init)? {
            ItemOrResult::Item(_) => self.run_exec_frame(evm)?,
            ItemOrResult::Result(result) => result,
        };

        let FrameResult::Call(sub_outcome) = sub_result else {
            unreachable!("a Call sub-frame yields a Call result");
        };

        // Fold the child's gas spend into the precompile's gas.
        if !gas.record_regular_cost(sub_outcome.result.gas.total_gas_spent()) {
            return Ok(out_of_gas(call.gas_limit, call.return_memory_offset));
        }

        // Return the nested call's output as the precompile's own output.
        Ok(FrameResult::Call(CallOutcome {
            result: InterpreterResult {
                result: InstructionResult::Return,
                gas,
                output: sub_outcome.result.output,
            },
            memory_offset: call.return_memory_offset,
            was_precompile_called: true,
            precompile_call_logs: Vec::new(),
            charged_new_account_state_gas: false,
        }))
    }
}

/// Builds an out-of-gas `CallOutcome` that consumes the whole gas limit.
const fn out_of_gas(gas_limit: u64, memory_offset: core::ops::Range<usize>) -> FrameResult {
    let mut gas = Gas::new(gas_limit);
    let _ = gas.record_regular_cost(gas_limit);
    FrameResult::Call(CallOutcome {
        result: InterpreterResult {
            result: InstructionResult::OutOfGas,
            gas,
            output: Bytes::new(),
        },
        memory_offset,
        was_precompile_called: true,
        precompile_call_logs: Vec::new(),
        charged_new_account_state_gas: false,
    })
}

impl<EVM> Handler for ReentrantHandler<EVM>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
        Frame: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
    >,
{
    type Evm = EVM;
    type Error = HandlerError<EVM>;
    type HaltReason = HaltReason;

    /// Intercepts the first frame if the transaction targets the callable precompile.
    fn run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, Self::Error> {
        if Self::is_callable(&first_frame_input) {
            return self.run_callable_precompile(evm, first_frame_input);
        }
        match evm.frame_init(first_frame_input)? {
            ItemOrResult::Result(frame_result) => Ok(frame_result),
            ItemOrResult::Item(_) => self.run_exec_frame(evm),
        }
    }

    /// Recursive frame driver — same as mainnet, but intercepts nested `CALL`s to
    /// the callable precompile and services them via [`Self::run_callable_precompile`].
    fn run_exec_frame(&mut self, evm: &mut Self::Evm) -> Result<FrameResult, Self::Error> {
        loop {
            match evm.frame_run()? {
                ItemOrResult::Item(init) => {
                    let child_result = if Self::is_callable(&init) {
                        self.run_callable_precompile(evm, init)?
                    } else {
                        match evm.frame_init(init)? {
                            ItemOrResult::Item(_) => self.run_exec_frame(evm)?,
                            ItemOrResult::Result(result) => result,
                        }
                    };
                    evm.frame_return_result(child_result)?;
                }
                ItemOrResult::Result(result) => {
                    evm.frame_stack().pop();
                    return Ok(result);
                }
            }
        }
    }
}

/// Runs one transaction that calls the reentrant precompile and returns its output.
fn run_callable_tx() -> Bytes {
    let caller = address!("00000000000000000000000000000000000000a0");

    let mut db = InMemoryDB::default();
    db.insert_account_info(
        caller,
        AccountInfo {
            balance: U256::from(1_000_000_000_000u64),
            ..Default::default()
        },
    );

    let mut evm = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .build_mainnet();

    evm.ctx.set_tx(
        revm::context::TxEnv::builder()
            .caller(caller)
            .kind(TxKind::Call(CALLABLE_ADDRESS))
            .gas_limit(200_000)
            .build()
            .unwrap(),
    );

    let result = ReentrantHandler::default()
        .run(&mut evm)
        .expect("transaction should execute");

    match result {
        ExecutionResult::Success {
            output: Output::Call(bytes),
            ..
        } => bytes,
        other => panic!("unexpected execution result: {other:?}"),
    }
}

fn main() {
    let output = run_callable_tx();
    println!("callable precompile returned: 0x{}", hex_encode(&output));
    // The nested call returned the 32-byte value 42; the precompile forwarded it.
    assert_eq!(U256::from_be_slice(&output), U256::from(42));
    println!("OK: precompile made a recursive EVM call and returned its result");
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reentrant_precompile_returns_nested_output() {
        let output = run_callable_tx();
        assert_eq!(
            U256::from_be_slice(&output),
            U256::from(42),
            "precompile should return the nested call's output"
        );
    }
}
