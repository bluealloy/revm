use crate::{
    primitives::{Address, Bytecode, Bytes, Log, LogData, B256, U256},
    rwasm::context_reader::EnvContextReader,
    Database,
    EvmContext,
    JournalEntry,
};
use core::{cell::RefCell, marker::PhantomData};
use fluentbase_core::helpers::exit_code_from_evm_error;
use fluentbase_sdk::{
    Account,
    AccountStatus,
    CallPrecompileResult,
    ContextFreeNativeAPI,
    DestroyedAccountResult,
    ExitCode,
    IsColdAccess,
    JournalCheckpoint,
    NativeAPI,
    SovereignAPI,
    SovereignContextReader,
    WriteStorageResult,
    F254,
};
use revm_interpreter::{Gas, InstructionResult, StateLoad};

struct RwasmSdkAdapterInner<'a, API: NativeAPI, DB: Database> {
    evm: &'a mut EvmContext<DB>,
    phantom_data: PhantomData<API>,
}

pub struct RwasmSdkAdapter<'a, API: NativeAPI, DB: Database> {
    inner: RefCell<RwasmSdkAdapterInner<'a, API, DB>>,
}

impl<'a, API: NativeAPI, DB: Database> RwasmSdkAdapter<'a, API, DB> {
    pub fn new(evm: &'a mut EvmContext<DB>) -> Self {
        let inner = RwasmSdkAdapterInner {
            evm,
            phantom_data: Default::default(),
        };
        Self {
            inner: RefCell::new(inner),
        }
    }
}

impl<'a, API: NativeAPI, DB: Database> ContextFreeNativeAPI for RwasmSdkAdapter<'a, API, DB> {
    fn keccak256(data: &[u8]) -> B256 {
        API::keccak256(data)
    }

    fn sha256(data: &[u8]) -> B256 {
        API::sha256(data)
    }

    fn poseidon(data: &[u8]) -> F254 {
        API::poseidon(data)
    }

    fn poseidon_hash(fa: &F254, fb: &F254, fd: &F254) -> F254 {
        API::poseidon_hash(fa, fb, fd)
    }

    fn ec_recover(digest: &B256, sig: &[u8; 64], rec_id: u8) -> [u8; 65] {
        API::ec_recover(digest, sig, rec_id)
    }

    fn debug_log(message: &str) {
        API::debug_log(message)
    }
}

impl<'a, API: NativeAPI, DB: Database> SovereignAPI for RwasmSdkAdapter<'a, API, DB> {
    fn context(&self) -> impl SovereignContextReader {
        let ctx = self.inner.borrow_mut();
        // TODO(dmitry123): "remove clone from here"
        EnvContextReader(ctx.evm.env.clone())
    }

    fn checkpoint(&self) -> JournalCheckpoint {
        let mut ctx = self.inner.borrow_mut();
        let (a, b) = ctx.evm.journaled_state.checkpoint().into();
        JournalCheckpoint(a, b)
    }

    fn commit(&self) {
        let mut ctx = self.inner.borrow_mut();
        ctx.evm.journaled_state.checkpoint_commit();
    }

    fn rollback(&self, checkpoint: JournalCheckpoint) {
        let mut ctx = self.inner.borrow_mut();
        ctx.evm
            .journaled_state
            .checkpoint_revert((checkpoint.0, checkpoint.1).into());
    }

    fn write_account(&self, account: Account, status: AccountStatus) {
        let mut ctx = self.inner.borrow_mut();
        // load account with this address from journaled state
        let StateLoad {
            data: db_account, ..
        } = ctx
            .evm
            .load_code(account.address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let old_nonce = db_account.info.nonce;
        // copy all account info fields
        db_account.info.balance = account.balance;
        db_account.info.nonce = account.nonce;
        db_account.info.code_hash = account.code_hash;
        // if this is an account deployment, then mark is as created (needed for SELFDESTRUCT)
        if status == AccountStatus::NewlyCreated {
            db_account.mark_created();
            let last_journal = ctx.evm.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::AccountCreated {
                address: account.address,
            });
        }
        // if nonce has changed, then inc nonce as well
        if account.nonce - old_nonce == 1 {
            let last_journal = ctx.evm.journaled_state.journal.last_mut().unwrap();
            last_journal.push(JournalEntry::NonceChange {
                address: account.address,
            });
        }
        // mark an account as touched
        ctx.evm.journaled_state.touch(&account.address);
    }

    fn destroy_account(&self, address: &Address, target: &Address) -> DestroyedAccountResult {
        let mut ctx = self.inner.borrow_mut();
        let result = ctx
            .evm
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
        let mut ctx = self.inner.borrow_mut();
        let StateLoad {
            data: account,
            is_cold,
        } = ctx
            .evm
            .load_account(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let mut account = Account::from(account.info.clone());
        account.address = *address;
        (account, is_cold)
    }

    fn account_committed(&self, _address: &Address) -> (Account, IsColdAccess) {
        todo!()
    }

    fn write_preimage(&self, address: Address, hash: B256, preimage: Bytes) {
        let mut ctx = self.inner.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .evm
            .load_account(address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        // println!(
        //     "writing preimage: address={}, hash={}, found_hash={}",
        //     address, hash, account.info.code_hash
        // );
        if account.info.code_hash == hash {
            ctx.evm.journaled_state.set_code_with_hash(
                address,
                Bytecode::new_raw(preimage.clone()),
                hash,
            );
            return;
        }
        // calculate preimage address
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .evm
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if !preimage_account.is_empty() {
            assert_eq!(
                preimage_account.info.code_hash, hash,
                "unexpected preimage hash"
            );
            return;
        }
        // set default preimage account fields
        preimage_account.info.nonce = 1;
        preimage_account.info.code_hash = hash;
        // write preimage as a bytecode for the account
        ctx.evm.journaled_state.set_code_with_hash(
            preimage_address,
            Bytecode::new_raw(preimage),
            hash,
        );
    }

    fn preimage(&self, address: &Address, hash: &B256) -> Option<Bytes> {
        let mut ctx = self.inner.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .evm
            .load_code(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        // println!(
        //     "loading preimage: address={}, hash={}, found_hash={}",
        //     address, hash, account.info.code_hash
        // );
        if account.info.code_hash == *hash {
            return account.info.code.as_ref().map(|v| v.original_bytes());
        }
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .evm
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        preimage_account
            .info
            .code
            .as_ref()
            .map(|v| v.original_bytes())
    }

    fn preimage_size(&self, address: &Address, hash: &B256) -> Option<u32> {
        let mut ctx = self.inner.borrow_mut();
        let StateLoad { data: account, .. } = ctx
            .evm
            .load_code(*address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        if account.info.code_hash == *hash {
            return account.info.code.as_ref().map(|v| v.len() as u32);
        }
        let preimage_address = Address::from_slice(&hash.0[12..]);
        let StateLoad {
            data: preimage_account,
            ..
        } = ctx
            .evm
            .load_account(preimage_address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        preimage_account.info.code.as_ref().map(|v| v.len() as u32)
    }

    fn write_storage(
        &self,
        address: Address,
        slot: U256,
        value: U256,
    ) -> (WriteStorageResult, IsColdAccess) {
        let mut ctx = self.inner.borrow_mut();
        let result = ctx
            .evm
            .sstore(address, slot, value)
            .map_err(|_| panic!("failed to update storage slot"))
            .unwrap();
        (
            WriteStorageResult {
                original_value: result.original_value,
                present_value: result.present_value,
            },
            result.is_cold,
        )
    }

    fn storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.inner.borrow_mut();
        let load_result = ctx
            .evm
            .load_account_delegated(*address)
            .unwrap_or_else(|_| panic!("internal storage error"));
        if load_result.is_empty {
            return (U256::ZERO, load_result.is_cold);
        }
        let state_load = ctx
            .evm
            .sload(*address, *slot)
            .ok()
            .expect("failed to read storage slot");
        (state_load.data, state_load.is_cold)
    }

    fn committed_storage(&self, address: &Address, slot: &U256) -> (U256, IsColdAccess) {
        let mut ctx = self.inner.borrow_mut();
        // TODO: "we need to check newly created account here and return zero"
        let value = ctx
            .evm
            .db
            .storage(*address, *slot)
            .map_err(|_| panic!("failed to load account"))
            .unwrap();
        (value, true)
    }

    fn write_transient_storage(&self, address: Address, index: U256, value: U256) {
        let mut ctx = self.inner.borrow_mut();
        ctx.evm.tstore(address, index, value);
    }

    fn transient_storage(&self, address: &Address, index: &U256) -> U256 {
        let mut ctx = self.inner.borrow_mut();
        ctx.evm.tload(*address, *index)
    }

    fn write_log(&self, address: Address, data: Bytes, topics: Vec<B256>) {
        let mut ctx = self.inner.borrow_mut();
        ctx.evm.journaled_state.log(Log {
            address,
            data: LogData::new_unchecked(topics, data),
        });
    }

    //noinspection RsBorrowChecker
    fn precompile(
        &self,
        address: &Address,
        input: &Bytes,
        gas: u64,
    ) -> Option<CallPrecompileResult> {
        let mut ctx = self.inner.borrow_mut();
        let result = ctx
            .evm
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
        let ctx = self.inner.borrow_mut();
        ctx.evm
            .journaled_state
            .warm_preloaded_addresses
            .contains(address)
    }

    fn transfer(&self, from: &mut Account, to: &mut Account, value: U256) -> Result<(), ExitCode> {
        Account::transfer(from, to, value)?;
        let mut ctx = self.inner.borrow_mut();
        ctx.evm
            .transfer(&from.address, &to.address, value)
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
