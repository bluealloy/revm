use alloc::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Analysis {
    JumpDest,
    GasBlockEnd, //contains gas for next block
    None,
}

const JUMP_MASK: u32 = 0x80000000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnalysisData {
    /// This variable packs two informations:
    /// IS_JUMP (1bit) | gas block ( 31bits)
    is_jump_and_gas_block: u32,
}

impl AnalysisData {
    pub fn none() -> Self {
        AnalysisData {
            is_jump_and_gas_block: 0,
        }
    }

    pub fn set_is_jump(&mut self) {
        self.is_jump_and_gas_block |= JUMP_MASK;
    }

    pub fn set_gas_block(&mut self, gas_block: u32) {
        let jump = self.is_jump_and_gas_block & JUMP_MASK;
        self.is_jump_and_gas_block = gas_block | jump;
    }

    pub fn is_jump(&self) -> bool {
        self.is_jump_and_gas_block & JUMP_MASK == JUMP_MASK
    }

    pub fn gas_block(&self) -> u64 {
        (self.is_jump_and_gas_block & (!JUMP_MASK)) as u64
    }
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidJumpAddress {
    pub first_gas_block: u32,
    /// Rc is used here so that we dont need to copy vector. We can move it to more suitable more accessable structure
    /// without copying underlying vec.
    pub analysis: Arc<Vec<AnalysisData>>,
}

impl ValidJumpAddress {
    pub fn new(analysis: Arc<Vec<AnalysisData>>, first_gas_block: u32) -> Self {
        Self {
            analysis,
            first_gas_block,
        }
    }
    /// Get the length of the valid mapping. This is the same as the
    /// code bytes.

    pub fn len(&self) -> usize {
        self.analysis.len()
    }

    /// Returns true if the valid list is empty

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.analysis.len() {
            return false;
        }
        self.analysis[position].is_jump()
    }

    pub fn gas_block(&self, position: usize) -> u64 {
        self.analysis[position].gas_block()
    }
}
