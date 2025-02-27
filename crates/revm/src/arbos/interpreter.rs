use core::mem;
use std::sync::{Arc, Mutex};

use arbutil::{
    evm::{
        api::{EvmApiMethod, VecReader},
        req::EvmApiRequestor,
        user::{UserOutcome, UserOutcomeKind},
        EvmData,
    },
    Bytes20, Bytes32,
};
use revm_interpreter::{Contract, Gas, Host, InterpreterAction, InterpreterResult};
use stylus::{
    native::NativeInstance,
    prover::programs::{
        config::{CompileConfig, StylusConfig},
        meter::MeteredMachine,
    },
    run::RunProgram,
};

use crate::{
    primitives::{keccak256, U256, U64},
    Context, Database,
};

use super::handler::{StylusFrameInputs, StylusHandler};

type EvmApiHandler<'a> =
    Arc<Box<dyn Fn(EvmApiMethod, Vec<u8>) -> (Vec<u8>, VecReader, arbutil::evm::api::Gas) + 'a>>;

pub static STYLUS_MAGIC_BYTES: &[u8] = &[0xEF, 0xF0, 0x00, 0x00];

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct StylusInterpreter {
    pub inputs: StylusFrameInputs,
}

impl Default for StylusInterpreter {
    fn default() -> Self {
        Self::new(Contract::default(), u64::MAX, false)
    }
}

impl StylusInterpreter {
    pub(crate) fn new(contract: Contract, gas_limit: u64, is_static: bool) -> Self {
        Self {
            inputs: StylusFrameInputs {
                input: contract.input,
                bytecode: contract.bytecode,
                target_address: contract.target_address,
                caller: contract.caller,
                call_value: contract.call_value,
                is_static,
                gas_limit,
            },
        }
    }

    pub(crate) fn run<EXT, DB: Database>(
        &self,
        context: &mut crate::Context<EXT, DB>,
        handler: &crate::Handler<'_, crate::Context<EXT, DB>, EXT, DB>,
    ) -> revm_interpreter::InterpreterAction {
        let evm_data = self.build_evm_data(context);

        let arbos_cfg = context.env().cfg.arbos_config.clone().unwrap_or_default();
        let compile_config = CompileConfig::version(arbos_cfg.stylus_version, arbos_cfg.debug_mode);
        let stylus_config = StylusConfig::new(
            arbos_cfg.stylus_version,
            arbos_cfg.max_depth,
            arbos_cfg.ink_price,
        );

        // Convert the mutable reference into an Arc<Mutex<Context>>
        let context = Arc::new(Mutex::new(context));
        let handler = Arc::new(Mutex::new(handler));

        // suppress type_complexity warning
        let callback = {
            let context = context.clone();
            let handler = handler.clone();
            let inputs = self.inputs.clone();

            move |req_type: arbutil::evm::api::EvmApiMethod,
                  req_data: Vec<u8>|
                  -> (Vec<u8>, VecReader, arbutil::evm::api::Gas) {
                let mut ctx = context.lock().unwrap(); // Lock the mutex to mutate
                let handler = handler.lock().unwrap(); // Lock the mutex to mutate
                super::handler::request(*ctx, *handler, inputs.clone(), req_type, req_data)
            }
        };

        let callback: EvmApiHandler<'_> = Arc::new(Box::new(callback));
        let unsafe_callback: &'static EvmApiHandler<'_> = unsafe { mem::transmute(&callback) };
        let evm_api = EvmApiRequestor::new(StylusHandler::new(unsafe_callback.clone()));

        let bytecode = self.inputs.bytecode.original_bytes();

        let bytecode = bytecode.strip_prefix(&[0xEF, 0xF0, 0x00, 0x00]).unwrap();

        let mut instance = NativeInstance::from_bytes(
            bytecode,
            evm_api,
            evm_data,
            &compile_config,
            stylus_config,
            wasmer_types::compilation::target::Target::default(),
        )
        .unwrap();

        let ink_limit = stylus_config
            .pricing
            .gas_to_ink(arbutil::evm::api::Gas(self.inputs.gas_limit));
        let mut gas = Gas::new(self.inputs.gas_limit);
        gas.spend_all();

        let outcome = match instance.run_main(&self.inputs.input, stylus_config, ink_limit) {
            Err(e) | Ok(UserOutcome::Failure(e)) => UserOutcome::Failure(e.wrap_err("call failed")),
            Ok(outcome) => outcome,
        };

        let mut gas_left = stylus_config
            .pricing
            .ink_to_gas(instance.ink_left().into())
            .0;

        let (kind, data) = outcome.into_data();

        let result = match kind {
            UserOutcomeKind::Success => crate::interpreter::InstructionResult::Return,
            UserOutcomeKind::Revert => crate::interpreter::InstructionResult::Revert,
            UserOutcomeKind::Failure => crate::interpreter::InstructionResult::Revert,
            UserOutcomeKind::OutOfInk => crate::interpreter::InstructionResult::OutOfGas,
            UserOutcomeKind::OutOfStack => {
                gas_left = 0;
                crate::interpreter::InstructionResult::StackOverflow
            }
        };

        gas.erase_cost(gas_left);

        InterpreterAction::Return {
            result: InterpreterResult {
                result,
                output: data.into(),
                gas,
            },
        }
    }

    fn build_evm_data<EXT, DB: Database>(&self, context: &Context<EXT, DB>) -> EvmData {
        // find target_address in context.evm.journaled_state.call_stack excluding last
        // if found, set reentrant to true
        // else set reentrant to false
        let reentrant = if context
            .evm
            .journaled_state
            .call_stack
            .iter()
            .filter(|&x| *x == self.inputs.target_address)
            .count()
            > 1
        {
            1
        } else {
            0
        };

        let evm_data: EvmData = EvmData {
            arbos_version: context
                .env()
                .cfg
                .arbos_config
                .clone()
                .unwrap_or_default()
                .arbos_version as u64,
            block_basefee: Bytes32::from(U256::from(context.env().block.basefee).to_be_bytes()),
            chainid: context.env().cfg.chain_id,
            block_coinbase: Bytes20::try_from(context.env().block.coinbase.as_slice()).unwrap(),
            block_gas_limit: U64::wrapping_from(context.env().block.gas_limit).to::<u64>(),
            block_number: U64::wrapping_from(context.env().block.number).to::<u64>(),
            block_timestamp: U64::wrapping_from(context.env().block.timestamp).to::<u64>(),
            contract_address: Bytes20::try_from(self.inputs.target_address.as_slice()).unwrap(),
            module_hash: Bytes32::try_from(
                keccak256(self.inputs.target_address.as_slice()).as_slice(),
            )
            .unwrap(),
            msg_sender: Bytes20::try_from(self.inputs.caller.as_slice()).unwrap(),
            msg_value: Bytes32::try_from(self.inputs.call_value.to_be_bytes_vec()).unwrap(),
            tx_gas_price: Bytes32::from(
                U256::from(context.env().effective_gas_price()).to_be_bytes(),
            ),
            tx_origin: Bytes20::try_from(context.env().tx.caller.as_slice()).unwrap(),
            reentrant,
            return_data_len: 0,
            cached: false,
            tracing: false,
        };

        evm_data
    }
}
