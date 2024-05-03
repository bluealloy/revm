pub enum OptimismHaltReason {
    FailedDeposit,
}

impl From<OptimismHaltReason> for crate::primitives::HaltReason<OptimismHaltReason> {
    fn from(value: OptimismHaltReason) -> Self {
        Self::Custom(value)
    }
}
