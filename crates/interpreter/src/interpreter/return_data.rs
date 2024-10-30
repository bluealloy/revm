use crate::interpreter::ReturnData;
use primitives::Bytes;

pub struct ReturnDataImpl(Bytes);

impl ReturnData for ReturnDataImpl {
    fn buffer(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn buffer_mut(&mut self) -> &mut Bytes {
        &mut self.0
    }
}
