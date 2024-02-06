use crate::{gas::Gas, JournalCheckpoint};
use fluentbase_types::ExitCode;
use revm_primitives::{Address, Bytes, B256, U256};

pub(crate) struct CreateInputs {
    pub(crate) caller: Address,
    pub(crate) value: U256,
    pub(crate) init_code: Bytes,
    pub(crate) salt: Option<U256>,
    pub(crate) gas_limit: u64,
}

pub(crate) struct CallInputsTransfer {
    pub(crate) source: Address,
    pub(crate) target: Address,
    pub(crate) value: U256,
}

pub(crate) struct CallInputsContext {
    pub(crate) caller: Address,
    pub(crate) address: Address,
    pub(crate) code_address: Address,
    pub(crate) apparent_value: U256,
}

pub(crate) struct CallInputs {
    pub(crate) contract: Address,
    pub(crate) gas_limit: u64,
    pub(crate) transfer: CallInputsTransfer,
    pub(crate) input: Bytes,
    pub(crate) context: CallInputsContext,
    pub(crate) is_static: bool,
}

pub(crate) struct PreparedCreate {
    pub(crate) created_address: Address,
    pub(crate) gas: Gas,
    pub(crate) checkpoint: JournalCheckpoint,
    pub(crate) bytecode: Bytes,
    pub(crate) caller: Address,
    pub(crate) value: U256,
}

pub(crate) struct CallCreateResult {
    pub(crate) result: ExitCode,
    pub(crate) created_address: Option<Address>,
    pub(crate) gas: Gas,
    pub(crate) return_value: Bytes,
}

impl CallCreateResult {
    pub(crate) fn from_error(result: ExitCode, gas: Gas) -> Self {
        Self {
            result,
            created_address: None,
            gas,
            return_value: Bytes::new(),
        }
    }
}

pub(crate) struct PreparedCall {
    pub(crate) gas: Gas,
    pub(crate) checkpoint: JournalCheckpoint,
    pub(crate) bytecode: Bytes,
    pub(crate) code_hash: B256,
    pub(crate) input: Bytes,
}

pub(crate) struct SelfDestructResult {
    pub(crate) had_value: bool,
    pub(crate) is_cold: bool,
    pub(crate) target_exists: bool,
    pub(crate) previously_destroyed: bool,
}
