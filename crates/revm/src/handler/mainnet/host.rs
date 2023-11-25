use crate::{
    db::Database,
    interpreter::SelfDestructResult,
    primitives::{Address, Bytes, Log, Spec, B256},
    Context,
};
use alloc::vec::Vec;

/// Handle host log call.
pub fn handle_host_log<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    address: Address,
    topics: Vec<B256>,
    data: Bytes,
) {
    let log = Log {
        address,
        topics,
        data,
    };
    context.evm.journaled_state.log(log);
}

/// Handle host selfdestruct call.
pub fn handle_selfdestruct<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    address: Address,
    target: Address,
) -> Option<SelfDestructResult> {
    context
        .evm
        .journaled_state
        .selfdestruct(address, target, &mut context.evm.db)
        .map_err(|e| context.evm.error = Some(e))
        .ok()
}
