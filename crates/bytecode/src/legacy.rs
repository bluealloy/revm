mod analyzed;
mod jump_map;
mod raw;

pub use analyzed::LegacyAnalyzedBytecode;
pub use jump_map::JumpTable;
pub use raw::{analyze_legacy, LegacyRawBytecode};
