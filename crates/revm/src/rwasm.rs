use crate::{
    primitives::{Address, Bytecode, Bytes, Log, LogData, B256, U256},
    Database,
    EvmContext,
    JournalEntry,
};
use core::{cell::RefCell, ops::Deref};
use fluentbase_core::helpers::exit_code_from_evm_error;
use fluentbase_sdk::{Account, AccountStatus, SovereignAPI};
use fluentbase_types::{
    BlockContext,
    CallPrecompileResult,
    ContractContext,
    DestroyedAccountResult,
    ExitCode,
    Fuel,
    IJournaledTrie,
    IsColdAccess,
    JournalCheckpoint,
    NativeAPI,
    SovereignStateResult,
    TxContext,
};
use revm_interpreter::{Gas, InstructionResult};

pub(crate) struct RwasmDbWrapper<'a, API: NativeAPI, DB: Database> {
    evm_context: RefCell<&'a mut EvmContext<DB>>,
    native_sdk: API,
    block_context: BlockContext,
    tx_context: TxContext,
}

impl<'a, API: NativeAPI, DB: Database> RwasmDbWrapper<'a, API, DB> {
    pub(crate) fn new(
        evm_context: &'a mut EvmContext<DB>,
        native_sdk: API,
    ) -> RwasmDbWrapper<'a, API, DB> {
        let block_context = BlockContext::from(evm_context.env.deref());
        let tx_context = TxContext::from(evm_context.env.deref());
        RwasmDbWrapper {
            evm_context: RefCell::new(evm_context),
            native_sdk,
            block_context,
            tx_context,
        }
    }
}

impl<'a, API: NativeAPI, DB: Database> RwasmDbWrapper<'a, API, DB> {
    fn block_hash(&self, number: U256) -> B256 {
        self.evm_context
            .borrow_mut()
            .block_hash(number)
            .map_err(|_| "unexpected EVM error")
            .unwrap()
    }

    fn write_transient_storage(&self, address: Address, index: U256, value: U256) {
        self.evm_context.borrow_mut().tstore(address, index, value)
    }

    fn transient_storage(&self, address: Address, index: U256) -> U256 {
        self.evm_context.borrow_mut().tload(address, index)
    }
}

impl<'a, API: NativeAPI, DB: Database> SovereignAPI for RwasmDbWrapper<'a, API, DB> {
    fn native_sdk(&self) -> &impl NativeAPI {
        &self.native_sdk
    }

    fn block_context(&self) -> &BlockContext {
        &self.block_context
    }

    fn tx_context(&self) -> &TxContext {
        &self.tx_context
    }

    fn contract_context(&self) -> Option<&ContractContext> {
        None
    }

    fn checkpoint(&self) -> JournalCheckpoint {
        let mut ctx = self.evm_context.borrow_mut();
        let (a, b) = ctx.journaled_state.checkpoint().into();
        JournalCheckpoint(a, b)
    }

    fn commit(&mut self) -> SovereignStateResult {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.checkpoint_commit();
        SovereignStateResult::default()
    }

    fn rollback(&mut self, checkpoint: JournalCheckpoint) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state
            .checkpoint_revert((checkpoint.0, checkpoint.1).into());
    }

    fn write_account(&mut self, account: Account, status: AccountStatus) {
        let mut ctx = self.evm_context.borrow_mut();
        // load account with this address from journaled state
        let (db_account, _) = ctx
            .load_account_with_code(account.address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let old_nonce = db_account.info.nonce;
        // copy all account info fields
        db_account.info.balance = account.balance;
        db_account.info.nonce = account.nonce;
        db_account.info.code_hash = account.source_code_hash;
        db_account.info.rwasm_code_hash = account.rwasm_code_hash;
        // if this is an account deployment, then mark is as created (needed for SELFDESTRUCT)
        if status == AccountStatus::NewlyCreated {
            db_account.mark_created();
            let last_journal = ctx.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::AccountCreated {
                address: account.address,
            });
        }
        // if nonce has changed, then inc nonce as well
        if account.nonce - old_nonce == 1 {
            let last_journal = ctx.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::NonceChange {
                address: account.address,
            });
        }
        // mark an account as touched
        ctx.journaled_state.touch(&account.address);
    }

    fn destroy_account(&mut self, address: &Address, target: &Address) -> DestroyedAccountResult {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .selfdestruct(*address, *target)
            .map_err(|_| "unexpected EVM self destruct error")
            .unwrap();
        DestroyedAccountResult {
            had_value: result.had_value,
            target_exists: result.target_exists,
            is_cold: result.is_cold,
            previously_destroyed: result.previously_destroyed,
        }
    }

    fn account(&self, address: &Address) -> (Account, bool) {
        let mut ctx = self.evm_context.borrow_mut();
        let (account, is_cold) = ctx
            .load_account(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let mut account = Account::from(account.info.clone());
        account.address = *address;
        (account, is_cold)
    }

    fn account_committed(&self, address: &Address) -> (Account, IsColdAccess) {
        todo!()
    }

    fn write_preimage(&mut self, address: Address, hash: B256, preimage: Bytes) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state
            .set_code(address, Bytecode::new_raw(preimage), Some(hash))
    }

    fn preimage(&self, hash: &B256) -> Option<Bytes> {
        let mut ctx = self.evm_context.borrow_mut();
        let bytecode = ctx
            .code_by_hash(*hash)
            .map_err(|_| panic!("failed to get bytecode by hash"))
            .unwrap();
        Some(bytecode)
    }

    fn preimage_size(&self, hash: &B256) -> u32 {
        self.evm_context
            .borrow_mut()
            .db
            .code_by_hash(*hash)
            .map(|b| b.bytecode().len() as u32)
            .unwrap_or_default()
    }

    fn write_storage(&mut self, address: Address, slot: U256, value: U256) -> IsColdAccess {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .sstore(address, slot, value)
            .map_err(|_| panic!("failed to update storage slot"))
            .unwrap();
        result.is_cold
    }

    fn storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.sload(*address, *slot)
            .ok()
            .expect("failed to read storage slot")
    }

    fn committed_storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.evm_context.borrow_mut();
        let (account, _) = ctx
            .load_account(*address)
            .map_err(|_| panic!("failed to load account"))
            .unwrap();
        if account.is_created() {
            return (U256::ZERO, true);
        }
        let value = ctx
            .db
            .storage(*address, *slot)
            .ok()
            .expect("failed to read storage slot");
        (value, true)
    }

    fn write_transient_storage(&mut self, address: Address, index: U256, value: U256) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.tstore(address, index, value);
    }

    fn transient_storage(&self, address: Address, index: U256) -> U256 {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.tload(address, index)
    }

    fn write_log(&mut self, address: Address, data: Bytes, topics: &[B256]) {
        let mut ctx = self.evm_context.borrow_mut();
        ctx.journaled_state.log(Log {
            address,
            data: LogData::new_unchecked(topics.into(), data),
        });
    }

    //noinspection RsBorrowChecker
    fn precompile(
        &self,
        address: &Address,
        input: &Bytes,
        gas: u64,
    ) -> Option<CallPrecompileResult> {
        let mut ctx = self.evm_context.borrow_mut();
        let result = ctx
            .call_precompile(&address, input, Gas::new(gas))
            .unwrap_or(None)?;
        Some(CallPrecompileResult {
            output: result.output,
            exit_code: exit_code_from_evm_error(result.result),
            gas_remaining: result.gas.remaining(),
            gas_refund: result.gas.refunded(),
        })
    }

    fn is_precompile(&self, address: &Address) -> bool {
        let ctx = self.evm_context.borrow_mut();
        ctx.journaled_state
            .warm_preloaded_addresses
            .contains(address)
    }

    fn transfer(
        &mut self,
        from: &mut Account,
        to: &mut Account,
        value: U256,
    ) -> Result<(), ExitCode> {
        Account::transfer(from, to, value)?;
        let mut ctx = self.evm_context.borrow_mut();
        ctx.transfer(&from.address, &to.address, value)
            .map_err(|_| panic!("unexpected EVM transfer error"))
            .unwrap()
            .and_then(|err| -> Option<InstructionResult> {
                panic!(
                    "it seems there is an account balance mismatch between ECL and REVM: {:?}",
                    err
                );
            });
        Ok(())
    }
}
