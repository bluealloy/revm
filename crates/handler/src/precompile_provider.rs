use auto_impl::auto_impl;
use context::{Cfg, LocalContextTr};
use context_interface::{ContextTr, JournalTr};
use interpreter::{CallInputs, Gas, InstructionResult, InterpreterResult};
use precompile::{PrecompileOutput, PrecompileSpecId, PrecompileStatus, Precompiles};
use primitives::{hardfork::SpecId, Address, AddressSet, Bytes};
use std::string::{String, ToString};

/// Provider for precompiled contracts in the EVM.
#[auto_impl(&mut, Box)]
pub trait PrecompileProvider<CTX: ContextTr> {
    /// The output type returned by precompile execution.
    type Output;

    /// Sets the spec id and returns true if the spec id was changed. Initial call to set_spec will always return true.
    ///
    /// Returns `true` if precompile addresses should be injected into the journal.
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool;

    /// Run the precompile.
    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<Self::Output>, String>;

    /// Get the warm addresses.
    fn warm_addresses(&self) -> &AddressSet;

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool {
        self.warm_addresses().contains(address)
    }
}

/// The [`PrecompileProvider`] for ethereum precompiles.
#[derive(Debug)]
pub struct EthPrecompiles {
    /// Contains precompiles for the current spec.
    pub precompiles: &'static Precompiles,
    /// Current spec. None means that spec was not set yet.
    pub spec: SpecId,
}

impl EthPrecompiles {
    /// Create a new precompile provider with the given spec.
    pub fn new(spec: SpecId) -> Self {
        Self {
            precompiles: Precompiles::new(PrecompileSpecId::from_spec_id(spec)),
            spec,
        }
    }

    /// Returns addresses of the precompiles.
    pub const fn warm_addresses(&self) -> &AddressSet {
        self.precompiles.addresses_set()
    }

    /// Returns whether the address is a precompile.
    pub fn contains(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}

impl Clone for EthPrecompiles {
    fn clone(&self) -> Self {
        Self {
            precompiles: self.precompiles,
            spec: self.spec,
        }
    }
}

/// Converts a [`PrecompileOutput`] into an [`InterpreterResult`].
///
/// Maps precompile status to the corresponding instruction result:
/// - `Success` → `InstructionResult::Return`
/// - `Revert` → `InstructionResult::Revert`
/// - `Halt(OOG)` → `InstructionResult::PrecompileOOG`
/// - `Halt(other)` → `InstructionResult::PrecompileError`
pub fn precompile_output_to_interpreter_result(
    output: PrecompileOutput,
    gas_limit: u64,
) -> InterpreterResult {
    // set output bytes
    let bytes = if output.status.is_success_or_revert() {
        output.bytes
    } else {
        Bytes::new()
    };

    let mut result = InterpreterResult {
        result: InstructionResult::Return,
        gas: Gas::new_with_regular_gas_and_reservoir(gas_limit, output.reservoir),
        output: bytes,
    };

    // set state gas, reservoir is already set in the Gas constructor
    result.gas.set_state_gas_spent(output.state_gas_used);
    result.gas.record_refund(output.gas_refunded);

    // spend used gas.
    if output.status.is_success_or_revert() {
        if !result.gas.record_regular_cost(output.gas_used) {
            result.gas.spend_all();
            result.output = Bytes::new();
            result.result = InstructionResult::PrecompileOOG;
            return result;
        }
    } else {
        result.gas.spend_all();
    }

    // set result
    result.result = match output.status {
        PrecompileStatus::Success => InstructionResult::Return,
        PrecompileStatus::Revert => InstructionResult::Revert,
        PrecompileStatus::Halt(halt_reason) => {
            if halt_reason.is_oog() {
                InstructionResult::PrecompileOOG
            } else {
                InstructionResult::PrecompileError
            }
        }
    };

    result
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for EthPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        let spec = spec.into();
        // generate new precompiles only on new spec
        if spec == self.spec {
            return false;
        }
        self.precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec));
        self.spec = spec;
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<InterpreterResult>, String> {
        let Some(precompile) = self.precompiles.get(&inputs.bytecode_address) else {
            return Ok(None);
        };

        let output = precompile
            .execute(
                &inputs.input.as_bytes(context),
                inputs.gas_limit,
                inputs.reservoir,
            )
            .map_err(|e| e.to_string())?;

        // If this is a top-level precompile call (depth == 1), persist the error message
        // into the local context so it can be returned as output in the final result.
        // Only do this for non-OOG halt errors.
        if let Some(halt_reason) = output.halt_reason() {
            if !halt_reason.is_oog() && context.journal().depth() == 1 {
                context
                    .local_mut()
                    .set_precompile_error_context(halt_reason.to_string());
            }
        }

        let result = precompile_output_to_interpreter_result(output, inputs.gas_limit);
        Ok(Some(result))
    }

    fn warm_addresses(&self) -> &AddressSet {
        Self::warm_addresses(self)
    }

    fn contains(&self, address: &Address) -> bool {
        Self::contains(self, address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instructions::EthInstructions, ExecuteEvm, MainContext};
    use context::{Context, Evm, FrameStack, TxEnv};
    use context_interface::result::{ExecutionResult, HaltReason, OutOfGasError};
    use database::InMemoryDB;
    use interpreter::interpreter::EthInterpreter;
    use primitives::{address, hardfork::SpecId, TxKind, U256};
    use state::AccountInfo;

    /// Test-only address that hosts an over-spending precompile.
    const OVERSPEND_PRECOMPILE: Address = address!("0000000000000000000000000000000000000100");

    /// Custom precompile provider that drives the bug path: it returns a
    /// `PrecompileOutput` with `status = Success` and `gas_used = u64::MAX` while
    /// `gas_limit` is finite. Without the fix, `record_regular_cost`'s `false` return
    /// is discarded so the call lands as `Return` with the gas tracker untouched —
    /// the transaction succeeds and refunds the precompile's "free" gas. With the fix,
    /// the helper converts the over-spend into `PrecompileOOG`, halting the tx.
    #[derive(Debug)]
    struct OverspendingPrecompiles {
        inner: EthPrecompiles,
        warm: AddressSet,
    }

    impl OverspendingPrecompiles {
        fn new(spec: SpecId) -> Self {
            let inner = EthPrecompiles::new(spec);
            let mut warm = AddressSet::default();
            warm.clone_from(inner.warm_addresses());
            warm.insert(OVERSPEND_PRECOMPILE);
            Self { inner, warm }
        }
    }

    impl<CTX> PrecompileProvider<CTX> for OverspendingPrecompiles
    where
        CTX: ContextTr<Cfg: Cfg<Spec = SpecId>>,
    {
        type Output = InterpreterResult;

        fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
            let changed =
                <EthPrecompiles as PrecompileProvider<CTX>>::set_spec(&mut self.inner, spec);
            self.warm.clone_from(self.inner.warm_addresses());
            self.warm.insert(OVERSPEND_PRECOMPILE);
            changed
        }

        fn run(
            &mut self,
            context: &mut CTX,
            inputs: &CallInputs,
        ) -> Result<Option<Self::Output>, String> {
            if inputs.bytecode_address == OVERSPEND_PRECOMPILE {
                let output = PrecompileOutput {
                    status: PrecompileStatus::Success,
                    gas_used: u64::MAX,
                    gas_refunded: 0,
                    state_gas_used: 0,
                    reservoir: inputs.reservoir,
                    bytes: Bytes::from_static(b"unreliable"),
                };
                return Ok(Some(precompile_output_to_interpreter_result(
                    output,
                    inputs.gas_limit,
                )));
            }
            <EthPrecompiles as PrecompileProvider<CTX>>::run(&mut self.inner, context, inputs)
        }

        fn warm_addresses(&self) -> &AddressSet {
            &self.warm
        }
    }

    /// End-to-end regression test for Bug 3. A transaction targets a custom precompile
    /// that lies about its gas usage. The fix turns this into an `OutOfGas(Precompile)`
    /// halt; without the fix it is silently treated as a successful call.
    #[test]
    fn overspending_precompile_halts_tx_with_precompile_oog() {
        let caller = address!("0000000000000000000000000000000000000001");
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            caller,
            AccountInfo {
                balance: U256::from(10).pow(U256::from(18)),
                ..Default::default()
            },
        );

        let spec = SpecId::default();
        let ctx = Context::mainnet().with_db(db);
        let mut evm = Evm {
            ctx,
            inspector: (),
            instruction: EthInstructions::<EthInterpreter, _>::new_mainnet_with_spec(spec),
            precompiles: OverspendingPrecompiles::new(spec),
            frame_stack: FrameStack::new_prealloc(8),
        };

        let tx = TxEnv::builder()
            .caller(caller)
            .kind(TxKind::Call(OVERSPEND_PRECOMPILE))
            .gas_limit(100_000)
            .build()
            .unwrap();

        let exec = evm.transact_one(tx).expect("handler returned an error");

        match exec {
            ExecutionResult::Halt { reason, .. } => {
                assert_eq!(
                    reason,
                    HaltReason::OutOfGas(OutOfGasError::Precompile),
                    "expected precompile OOG halt for over-spending precompile",
                );
            }
            ExecutionResult::Success { .. } => panic!(
                "before-fix behavior leaked: over-spending precompile reported Success \
                 instead of halting with PrecompileOOG"
            ),
            ExecutionResult::Revert { .. } => panic!("expected Halt(PrecompileOOG), got Revert"),
        }
    }
}
