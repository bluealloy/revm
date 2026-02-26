//! Legacy bytecode analysis and jump table.

mod analysis;
mod jump_map;

pub(crate) use analysis::analyze_legacy;
pub use jump_map::JumpTable;
