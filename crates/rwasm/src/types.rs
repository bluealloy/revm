use crate::{gas::Gas, journal::AccountCheckpoint};
use fluentbase_types::ExitCode;
use revm_primitives::{Address, Bytes, B256, U256};

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
