use crate::gas::Gas;
use fluentbase_types::ExitCode;
use revm_primitives::{Address, Bytes};

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
