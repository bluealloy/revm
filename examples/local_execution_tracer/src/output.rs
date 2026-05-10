use revm::state::EvmState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct TraceOutput {
    pub(crate) summary: TraceSummary,
    pub(crate) steps: Vec<StepTrace>,
    pub(crate) calls: Vec<CallTrace>,
    pub(crate) call_tree: Vec<CallTreeNode>,
    pub(crate) logs: Vec<LogTrace>,
    pub(crate) state_diff: Vec<StateDiff>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TraceSummary {
    pub(crate) success: bool,
    pub(crate) gas_used: u64,
    pub(crate) step_count: usize,
    pub(crate) call_count: usize,
    pub(crate) log_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct StepTrace {
    pub(crate) depth: usize,
    pub(crate) pc: usize,
    pub(crate) opcode: &'static str,
    pub(crate) gas_remaining: u64,
    pub(crate) stack_top: Vec<String>,
    pub(crate) memory_size: usize,
    pub(crate) memory_preview: String,
    pub(crate) memory_truncated: bool,
    pub(crate) storage: Vec<StepStorageTrace>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StepStorageTrace {
    pub(crate) op: &'static str,
    pub(crate) address: String,
    pub(crate) slot: String,
    pub(crate) value_before: Option<String>,
    pub(crate) value_after: Option<String>,
    pub(crate) write_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CallTrace {
    pub(crate) depth: usize,
    pub(crate) kind: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) value: String,
    pub(crate) input: String,
    pub(crate) gas_limit: u64,
    pub(crate) success: Option<bool>,
    pub(crate) gas_used: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CallTreeNode {
    depth: usize,
    kind: String,
    from: String,
    to: String,
    value: String,
    gas_limit: u64,
    success: Option<bool>,
    gas_used: Option<u64>,
    children: Vec<CallTreeNode>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LogTrace {
    pub(crate) address: String,
    pub(crate) topics: Vec<String>,
    pub(crate) data: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct StateDiff {
    address: String,
    storage: Vec<StorageDiff>,
}

#[derive(Debug, Serialize)]
struct StorageDiff {
    slot: String,
    before: String,
    after: String,
}

pub(crate) fn collect_state_diff(state: &EvmState) -> Vec<StateDiff> {
    let mut diffs = state
        .iter()
        .filter_map(|(address, account)| {
            let mut storage = account
                .changed_storage_slots()
                .map(|(slot, value)| StorageDiff {
                    slot: format!("{slot:#x}"),
                    before: format!("{:#x}", value.original_value()),
                    after: format!("{:#x}", value.present_value()),
                })
                .collect::<Vec<_>>();

            storage.sort_by(|left, right| left.slot.cmp(&right.slot));
            (!storage.is_empty()).then(|| StateDiff {
                address: address.to_string(),
                storage,
            })
        })
        .collect::<Vec<_>>();
    diffs.sort_by(|left, right| left.address.cmp(&right.address));
    diffs
}

pub(crate) fn build_call_tree(calls: &[CallTrace]) -> Vec<CallTreeNode> {
    let mut roots = Vec::new();
    let mut stack: Vec<CallTreeNode> = Vec::new();

    for call in calls {
        while stack.len() > call.depth {
            flush_call_tree_node(&mut stack, &mut roots);
        }

        stack.push(CallTreeNode {
            depth: call.depth,
            kind: call.kind.clone(),
            from: call.from.clone(),
            to: call.to.clone(),
            value: call.value.clone(),
            gas_limit: call.gas_limit,
            success: call.success,
            gas_used: call.gas_used,
            children: Vec::new(),
        });
    }

    while !stack.is_empty() {
        flush_call_tree_node(&mut stack, &mut roots);
    }

    roots
}

fn flush_call_tree_node(stack: &mut Vec<CallTreeNode>, roots: &mut Vec<CallTreeNode>) {
    let Some(node) = stack.pop() else {
        return;
    };

    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}
