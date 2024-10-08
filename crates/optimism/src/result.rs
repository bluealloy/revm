use revm::wiring::result::HaltReason;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OptimismHaltReason {
    Base(HaltReason),
    FailedDeposit,
}

impl From<HaltReason> for OptimismHaltReason {
    fn from(value: HaltReason) -> Self {
        Self::Base(value)
    }
}
