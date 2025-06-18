use primitives::hardfork::SpecId;

use super::RuntimeFlag;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeFlags {
    pub is_static: bool,
    pub spec_id: SpecId,
}

impl RuntimeFlag for RuntimeFlags {
    fn is_static(&self) -> bool {
        self.is_static
    }

    fn spec_id(&self) -> SpecId {
        self.spec_id
    }
}
