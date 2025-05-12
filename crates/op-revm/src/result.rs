use revm::context_interface::result::HaltReason;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpHaltReason {
    Base(HaltReason),
    FailedDeposit,
}

impl From<HaltReason> for OpHaltReason {
    fn from(value: HaltReason) -> Self {
        Self::Base(value)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use revm::context_interface::result::OutOfGasError;

    #[test]
    fn test_serialize_json_op_halt_reason() {
        let response = r#"{"Base":{"OutOfGas":"Basic"}}"#;

        let op_halt_reason: OpHaltReason = serde_json::from_str(response).unwrap();
        assert_eq!(
            op_halt_reason,
            HaltReason::OutOfGas(OutOfGasError::Basic).into()
        );
    }
}
