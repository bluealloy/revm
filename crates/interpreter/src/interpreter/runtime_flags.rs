use primitives::hardfork::SpecId;

use super::RuntimeFlag;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Runtime flags that control interpreter execution behavior and constraints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeFlags {
    /// Whether the execution context is static (read-only)
    pub is_static: bool,
    /// Whether this is EOF initialization code execution
    pub is_eof_init: bool,
    /// Whether the bytecode being executed is EOF format
    pub is_eof: bool,
    /// The Ethereum specification version in effect
    pub spec_id: SpecId,
}

impl RuntimeFlag for RuntimeFlags {
    fn is_static(&self) -> bool {
        self.is_static
    }

    fn is_eof(&self) -> bool {
        self.is_eof
    }

    fn is_eof_init(&self) -> bool {
        self.is_eof_init
    }

    fn spec_id(&self) -> SpecId {
        self.spec_id
    }
}
