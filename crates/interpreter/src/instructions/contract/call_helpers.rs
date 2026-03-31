use crate::{
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
    InstructionContext,
};
use context_interface::{cfg::GasParams, host::LoadError, Host};
use core::{cmp::min, ops::Range};
use primitives::{
    hardfork::SpecId::{self, *},
    Address, B256, U256,
};
use state::Bytecode;

/// Gets memory input and output ranges for call instructions.
#[inline]
pub fn get_memory_input_and_out_ranges(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    gas_params: &GasParams,
) -> Option<(Range<usize>, Range<usize>)> {
    popn!([in_offset, in_len, out_offset, out_len], interpreter, None);

    let mut in_range = resize_memory(interpreter, gas_params, in_offset, in_len)?;

    if !in_range.is_empty() {
        let offset = interpreter.memory.local_memory_offset();
        in_range = in_range.start.saturating_add(offset)..in_range.end.saturating_add(offset);
    }

    let ret_range = resize_memory(interpreter, gas_params, out_offset, out_len)?;
    Some((in_range, ret_range))
}

/// Resize memory and return range of memory.
/// If `len` is 0 dont touch memory and return `usize::MAX` as offset and 0 as length.
#[inline]
pub fn resize_memory(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    gas_params: &GasParams,
    offset: U256,
    len: U256,
) -> Option<Range<usize>> {
    let len = as_usize_or_fail_ret!(interpreter, len, None);
    let offset = if len != 0 {
        let offset = as_usize_or_fail_ret!(interpreter, offset, None);
        resize_memory!(interpreter, gas_params, offset, len, None);
        offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };
    Some(offset..offset + len)
}

/// Calculates gas cost and limit for call instructions.
#[inline(never)]
pub fn load_acc_and_calc_gas<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
    stack_gas_limit: u64,
) -> Option<(u64, Bytecode, B256)> {
    // Transfer value cost
    if transfers_value {
        gas!(
            context.interpreter,
            context.host.gas_params().transfer_value_cost(),
            None
        );
    }

    // load account delegated and deduct dynamic gas.
    let (gas, bytecode, code_hash) =
        load_account_delegated_handle_error(context, to, transfers_value, create_empty_account)?;
    let interpreter = &mut context.interpreter;

    // deduct dynamic gas.
    gas!(interpreter, gas, None);

    let interpreter = &mut context.interpreter;
    let host = &mut context.host;

    // EIP-150: Gas cost changes for IO-heavy operations
    let mut gas_limit = if interpreter.runtime_flag.spec_id().is_enabled_in(TANGERINE) {
        // On mainnet this will take return 63/64 of gas_limit.
        let reduced_gas_limit = host
            .gas_params()
            .call_stipend_reduction(interpreter.gas.remaining());
        min(reduced_gas_limit, stack_gas_limit)
    } else {
        stack_gas_limit
    };
    gas!(interpreter, gas_limit, None);

    // Add call stipend if there is value to be transferred.
    if transfers_value {
        gas_limit = gas_limit.saturating_add(host.gas_params().call_stipend());
    }

    Some((gas_limit, bytecode, code_hash))
}

/// Loads accounts and its delegate account.
#[inline]
pub fn load_account_delegated_handle_error<H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    to: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Option<(u64, Bytecode, B256)> {
    // move this to static gas.
    let remaining_gas = context.interpreter.gas.remaining();
    match load_account_delegated(
        context.host,
        context.interpreter.runtime_flag.spec_id(),
        remaining_gas,
        to,
        transfers_value,
        create_empty_account,
    ) {
        Ok(out) => return Some(out),
        Err(LoadError::ColdLoadSkipped) => {
            context.interpreter.halt_oog();
        }
        Err(LoadError::DBError) => {
            context.interpreter.halt_fatal();
        }
    }
    None
}

/// Loads accounts and its delegate account.
///
/// Assumption is that warm gas is already deducted.
#[inline]
pub fn load_account_delegated<H: Host + ?Sized>(
    host: &mut H,
    spec: SpecId,
    remaining_gas: u64,
    address: Address,
    transfers_value: bool,
    create_empty_account: bool,
) -> Result<(u64, Bytecode, B256), LoadError> {
    let mut cost = 0;
    let is_berlin = spec.is_enabled_in(SpecId::BERLIN);
    let is_spurious_dragon = spec.is_enabled_in(SpecId::SPURIOUS_DRAGON);

    let additional_cold_cost = host.gas_params().cold_account_additional_cost();
    let warm_storage_read_cost = host.gas_params().warm_storage_read_cost();

    let skip_cold_load = is_berlin && remaining_gas < additional_cold_cost;
    let account = host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;
    if is_berlin && account.is_cold {
        cost += additional_cold_cost;
    }
    let mut bytecode = account.code.clone().unwrap_or_default();
    let mut code_hash = account.code_hash();
    // New account cost, as account is empty there is no delegated account and we can return early.
    if create_empty_account && account.is_empty {
        cost += host
            .gas_params()
            .new_account_cost(is_spurious_dragon, transfers_value);
        return Ok((cost, bytecode, code_hash));
    }

    // load delegate code if account is EIP-7702 and Prague is enabled.
    if spec.is_enabled_in(SpecId::PRAGUE) {
        if let Some(address) = account.code.as_ref().and_then(Bytecode::eip7702_address) {
            cost += warm_storage_read_cost;
            if cost > remaining_gas {
                return Err(LoadError::ColdLoadSkipped);
            }

            // skip cold load if there is enough gas to cover the cost.
            let skip_cold_load = remaining_gas < cost + additional_cold_cost;
            let delegate_account =
                host.load_account_info_skip_cold_load(address, true, skip_cold_load)?;

            if delegate_account.is_cold {
                cost += additional_cold_cost;
            }
            bytecode = delegate_account.code.clone().unwrap_or_default();
            code_hash = delegate_account.code_hash();
        }
    }

    Ok((cost, bytecode, code_hash))
}

#[cfg(test)]
mod tests {
    use super::load_account_delegated;
    use context_interface::{
        cfg::GasParams,
        context::{SStoreResult, SelfDestructResult, StateLoad},
        host::{Host, LoadError},
        journaled_state::AccountInfoLoad,
    };
    use primitives::{
        hardfork::SpecId, Address, HashMap, Log, StorageKey, StorageValue, B256, U256,
    };
    use state::{AccountInfo, Bytecode};
    use std::borrow::Cow;
    use std::vec::Vec;

    #[derive(Debug)]
    struct TestHost {
        gas_params: GasParams,
        accounts: HashMap<Address, AccountInfo>,
        loads: Vec<Address>,
    }

    impl TestHost {
        fn new(spec: SpecId) -> Self {
            Self {
                gas_params: GasParams::new_spec(spec),
                accounts: HashMap::default(),
                loads: Vec::new(),
            }
        }

        fn insert_account(&mut self, address: Address, account: AccountInfo) {
            self.accounts.insert(address, account);
        }
    }

    impl Host for TestHost {
        fn basefee(&self) -> U256 {
            U256::ZERO
        }

        fn blob_gasprice(&self) -> U256 {
            U256::ZERO
        }

        fn gas_limit(&self) -> U256 {
            U256::ZERO
        }

        fn difficulty(&self) -> U256 {
            U256::ZERO
        }

        fn prevrandao(&self) -> Option<U256> {
            None
        }

        fn block_number(&self) -> U256 {
            U256::ZERO
        }

        fn timestamp(&self) -> U256 {
            U256::ZERO
        }

        fn beneficiary(&self) -> Address {
            Address::ZERO
        }

        fn slot_num(&self) -> U256 {
            U256::ZERO
        }

        fn chain_id(&self) -> U256 {
            U256::ZERO
        }

        fn effective_gas_price(&self) -> U256 {
            U256::ZERO
        }

        fn caller(&self) -> Address {
            Address::ZERO
        }

        fn blob_hash(&self, _number: usize) -> Option<U256> {
            None
        }

        fn max_initcode_size(&self) -> usize {
            0
        }

        fn gas_params(&self) -> &GasParams {
            &self.gas_params
        }

        fn block_hash(&mut self, _number: u64) -> Option<B256> {
            None
        }

        fn selfdestruct(
            &mut self,
            _address: Address,
            _target: Address,
            _skip_cold_load: bool,
        ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
            Ok(StateLoad::new(SelfDestructResult::default(), false))
        }

        fn log(&mut self, _log: Log) {}

        fn sstore_skip_cold_load(
            &mut self,
            _address: Address,
            _key: StorageKey,
            _value: StorageValue,
            _skip_cold_load: bool,
        ) -> Result<StateLoad<SStoreResult>, LoadError> {
            Ok(StateLoad::new(SStoreResult::default(), false))
        }

        fn sload_skip_cold_load(
            &mut self,
            _address: Address,
            _key: StorageKey,
            _skip_cold_load: bool,
        ) -> Result<StateLoad<StorageValue>, LoadError> {
            Ok(StateLoad::new(StorageValue::ZERO, false))
        }

        fn tstore(&mut self, _address: Address, _key: StorageKey, _value: StorageValue) {}

        fn tload(&mut self, _address: Address, _key: StorageKey) -> StorageValue {
            StorageValue::ZERO
        }

        fn load_account_info_skip_cold_load(
            &mut self,
            address: Address,
            _load_code: bool,
            _skip_cold_load: bool,
        ) -> Result<AccountInfoLoad<'_>, LoadError> {
            self.loads.push(address);
            let account = self.accounts.get(&address).cloned().unwrap_or_default();
            let is_empty = account.is_empty();
            Ok(AccountInfoLoad {
                account: Cow::Owned(account),
                is_cold: false,
                is_empty,
            })
        }
    }

    #[test]
    fn pre_prague_does_not_follow_delegation() {
        let target = Address::with_last_byte(0x10);
        let delegate = Address::with_last_byte(0x20);
        let delegated_marker = Bytecode::new_eip7702(delegate);
        let delegate_code = Bytecode::new_legacy(vec![0x00].into());

        let mut host = TestHost::new(SpecId::CANCUN);
        host.insert_account(
            target,
            AccountInfo::default().with_code(delegated_marker.clone()),
        );
        host.insert_account(delegate, AccountInfo::default().with_code(delegate_code));

        let (_, bytecode, code_hash) =
            load_account_delegated(&mut host, SpecId::CANCUN, 1_000_000, target, false, false)
                .expect("account load should succeed");

        assert_eq!(bytecode, delegated_marker);
        assert_eq!(code_hash, delegated_marker.hash_slow());
        assert_eq!(host.loads, vec![target]);
    }

    #[test]
    fn prague_follows_delegation() {
        let target = Address::with_last_byte(0x11);
        let delegate = Address::with_last_byte(0x21);
        let delegated_marker = Bytecode::new_eip7702(delegate);
        let delegate_code = Bytecode::new_legacy(vec![0x00].into());

        let mut host = TestHost::new(SpecId::PRAGUE);
        host.insert_account(target, AccountInfo::default().with_code(delegated_marker));
        host.insert_account(
            delegate,
            AccountInfo::default().with_code(delegate_code.clone()),
        );

        let (_, bytecode, code_hash) =
            load_account_delegated(&mut host, SpecId::PRAGUE, 1_000_000, target, false, false)
                .expect("account load should succeed");

        assert_eq!(bytecode, delegate_code);
        assert_eq!(code_hash, delegate_code.hash_slow());
        assert_eq!(host.loads, vec![target, delegate]);
    }
}
