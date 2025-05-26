mod analysis;
mod analyzed;
mod jump_map;
mod raw;

pub use analysis::analyze_legacy;
pub use analyzed::LegacyAnalyzedBytecode;
pub use jump_map::JumpTable;
pub use raw::LegacyRawBytecode;
