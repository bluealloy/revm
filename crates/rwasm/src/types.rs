use fluentbase_types::ExitCode;

pub(crate) type InstructionResult = ExitCode;

pub(crate) struct SelfDestructResult {
    pub(crate) had_value: bool,
    pub(crate) is_cold: bool,
    pub(crate) target_exists: bool,
    pub(crate) previously_destroyed: bool,
}
