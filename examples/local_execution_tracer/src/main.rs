//! A minimal local execution tracer built with a custom `Inspector`.
//!
//! The example is deterministic and does not require RPC access. It installs
//! runtime bytecode into an in-memory account, executes one synthetic
//! transaction, and serializes a compact JSON trace with opcode steps, calls,
//! logs, storage touches, a call tree, and final storage diffs.

mod output;
mod scenario;
mod tracer;

use output::{build_call_tree, collect_state_diff, TraceOutput, TraceSummary};
use revm::{Context, InspectEvm, MainBuilder, MainContext};
use scenario::{build_db, build_tx};
use std::error::Error;
use tracer::MiniTracer;

fn main() -> Result<(), Box<dyn Error>> {
    let db = build_db()?;
    let tx = build_tx();

    let tracer = MiniTracer::default();
    let context = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.tx_gas_limit_cap = Some(u64::MAX))
        .with_db(db);
    let mut evm = context.build_mainnet_with_inspector(tracer);

    let result_and_state = evm.inspect_tx(tx)?;
    let tracer = evm.into_inspector();

    let trace = TraceOutput {
        summary: TraceSummary {
            success: result_and_state.result.is_success(),
            gas_used: result_and_state.result.tx_gas_used(),
            step_count: tracer.steps.len(),
            call_count: tracer.calls.len(),
            log_count: tracer.logs.len(),
        },
        steps: tracer.steps,
        calls: tracer.calls.clone(),
        call_tree: build_call_tree(&tracer.calls),
        logs: tracer.logs,
        state_diff: collect_state_diff(&result_and_state.state),
    };

    println!("{}", serde_json::to_string_pretty(&trace)?);
    Ok(())
}
