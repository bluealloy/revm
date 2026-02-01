//! Helpers for opcode fusion analysis.

use crate::opcode;
use std::vec::Vec;

/// No fusion for this opcode.
pub const FUSION_NONE: u8 = 0;
/// Fused PUSH1 + ADD.
pub const FUSION_PUSH1_ADD: u8 = 1;
/// Fused PUSH1 + SUB.
pub const FUSION_PUSH1_SUB: u8 = 2;
/// Fused PUSH1 + MUL.
pub const FUSION_PUSH1_MUL: u8 = 3;
/// Fused PUSH1 + JUMP.
pub const FUSION_PUSH1_JUMP: u8 = 4;
/// Fused PUSH1 + JUMPI.
pub const FUSION_PUSH1_JUMPI: u8 = 5;

/// Length of the fusion table.
pub const FUSION_TABLE_LEN: usize = 6;

/// Minimum ratio of fusible opcodes to total opcodes to enable fusion.
/// Below this threshold, the overhead of checking fusion outweighs the benefit.
pub const FUSION_DENSITY_THRESHOLD: f64 = 0.02;

/// Result of fusion analysis.
pub struct FusionAnalysis {
    /// Fusion map indexed by PC, or None if fusion density is below threshold.
    pub map: Option<Vec<u8>>,
    /// Number of fusible opcode pairs found.
    pub fusible_count: usize,
    /// Total number of opcodes analyzed.
    pub op_count: usize,
}

/// Analyzes bytecode for fusible opcode patterns.
///
/// Returns a fusion map only if the density of fusible patterns exceeds
/// the threshold. This avoids the per-instruction overhead for contracts
/// that wouldn't benefit from fusion.
pub fn analyze_fusion(bytecode: &[u8]) -> FusionAnalysis {
    if bytecode.is_empty() {
        return FusionAnalysis {
            map: None,
            fusible_count: 0,
            op_count: 0,
        };
    }

    let mut map = vec![FUSION_NONE; bytecode.len()];
    let mut fusible_count = 0usize;
    let mut op_count = 0usize;
    let mut i = 0usize;

    while i < bytecode.len() {
        let op = bytecode[i];
        op_count += 1;

        if op >= opcode::PUSH1 && op <= opcode::PUSH32 {
            let push_len = (op - opcode::PUSH1) as usize + 1;
            if op == opcode::PUSH1 && i + 2 < bytecode.len() {
                let next = bytecode[i + 2];
                let fusion = match next {
                    opcode::ADD => FUSION_PUSH1_ADD,
                    opcode::SUB => FUSION_PUSH1_SUB,
                    opcode::MUL => FUSION_PUSH1_MUL,
                    opcode::JUMP => FUSION_PUSH1_JUMP,
                    opcode::JUMPI => FUSION_PUSH1_JUMPI,
                    _ => FUSION_NONE,
                };
                if fusion != FUSION_NONE {
                    map[i] = fusion;
                    fusible_count += 1;
                }
            }
            i = i.saturating_add(1 + push_len);
            continue;
        }
        i += 1;
    }

    // Only return fusion map if density exceeds threshold
    let density = if op_count > 0 {
        fusible_count as f64 / op_count as f64
    } else {
        0.0
    };

    let map = if density >= FUSION_DENSITY_THRESHOLD {
        Some(map)
    } else {
        None
    };

    FusionAnalysis {
        map,
        fusible_count,
        op_count,
    }
}
