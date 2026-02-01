use crate::interpreter_types::{Fusion, Immediates, Jumps, LegacyBytecode, LoopControl};
use core::ops::Deref;

/// Wrapper for bytecode types that do not support fusion.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoFusionBytecode<B>(pub B);

impl<B> From<B> for NoFusionBytecode<B> {
    fn from(value: B) -> Self {
        Self(value)
    }
}

impl<B> Deref for NoFusionBytecode<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> Fusion for NoFusionBytecode<B> {}

impl<B: Jumps> Jumps for NoFusionBytecode<B> {
    #[inline]
    fn relative_jump(&mut self, offset: isize) {
        self.0.relative_jump(offset);
    }

    #[inline]
    fn absolute_jump(&mut self, offset: usize) {
        self.0.absolute_jump(offset);
    }

    #[inline]
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool {
        self.0.is_valid_legacy_jump(offset)
    }

    #[inline]
    fn pc(&self) -> usize {
        self.0.pc()
    }

    #[inline]
    fn opcode(&self) -> u8 {
        self.0.opcode()
    }
}

impl<B: Immediates> Immediates for NoFusionBytecode<B> {
    #[inline]
    fn read_u16(&self) -> u16 {
        self.0.read_u16()
    }

    #[inline]
    fn read_u8(&self) -> u8 {
        self.0.read_u8()
    }

    #[inline]
    fn read_offset_u16(&self, offset: isize) -> u16 {
        self.0.read_offset_u16(offset)
    }

    #[inline]
    fn read_slice(&self, len: usize) -> &[u8] {
        self.0.read_slice(len)
    }
}

impl<B: LoopControl> LoopControl for NoFusionBytecode<B> {
    #[inline]
    fn is_not_end(&self) -> bool {
        self.0.is_not_end()
    }

    #[inline]
    fn reset_action(&mut self) {
        self.0.reset_action();
    }

    #[inline]
    fn set_action(&mut self, action: crate::InterpreterAction) {
        self.0.set_action(action);
    }

    #[inline]
    fn action(&mut self) -> &mut Option<crate::InterpreterAction> {
        self.0.action()
    }
}

impl<B: LegacyBytecode> LegacyBytecode for NoFusionBytecode<B> {
    #[inline]
    fn bytecode_len(&self) -> usize {
        self.0.bytecode_len()
    }

    #[inline]
    fn bytecode_slice(&self) -> &[u8] {
        self.0.bytecode_slice()
    }
}
