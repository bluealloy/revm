use crate::interpreter::ReturnData;
use primitives::Bytes;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct ReturnDataImpl(pub Bytes);

impl ReturnData for ReturnDataImpl {
    fn buffer(&self) -> &Bytes {
        &self.0
    }

    fn set_buffer(&mut self, bytes: Bytes) {
        self.0 = bytes;
    }
}
