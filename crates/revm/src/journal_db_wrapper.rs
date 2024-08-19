use crate::{primitives::{Address, Bytecode, Bytes, Log, LogData, B256, U256}, Database, EvmContext, JournalEntry};
use core::{cell::RefCell, fmt::Debug};
use fluentbase_core::{debug_log, helpers::exit_code_from_evm_error};
use fluentbase_sdk::{
    Account,
    AccountCheckpoint,
    AccountManager,
    EvmCallMethodOutput,
    JZKT_ACCOUNT_COMPRESSION_FLAGS,
    JZKT_ACCOUNT_FIELDS_COUNT,
    JZKT_ACCOUNT_RWASM_CODE_HASH_FIELD,
    JZKT_ACCOUNT_SOURCE_CODE_HASH_FIELD,
    JZKT_STORAGE_COMPRESSION_FLAGS,
    JZKT_STORAGE_FIELDS_COUNT,
};
use fluentbase_types::{
    consts::EVM_STORAGE_ADDRESS,
    ExitCode,
    IJournaledTrie,
    JournalEvent,
    JournalLog,
};
use revm_interpreter::{Gas, InstructionResult};
use std::borrow::BorrowMut;

pub(crate) struct JournalDbWrapper<'a, DB: Database> {
    ctx: RefCell<&'a mut EvmContext<DB>>,
}

impl<'a, DB: Database> JournalDbWrapper<'a, DB> {
    pub fn new(ctx: RefCell<&'a mut EvmContext<DB>>) -> JournalDbWrapper<'a, DB> {
        JournalDbWrapper { ctx }
    }
}

// impl<'a, DB: Database> IJournaledTrie for JournalDbWrapper<'a, DB> {
//     fn checkpoint(&self) -> fluentbase_types::JournalCheckpoint {
//         fluentbase_types::JournalCheckpoint::from_u64(AccountManager::checkpoint(self))
//     }
//
//     fn get(&self, key: &[u8; 32], committed: bool) -> Option<(Vec<[u8; 32]>, u32, bool)> {
//         // if first 12 bytes are empty then its account load otherwise storage
//         if key[..12] == [0u8; 12] {
//             let address = Address::from_slice(&key[12..]);
//             let (account, is_cold) = AccountManager::account(self, address);
//             Some((
//                 account.get_fields().to_vec(),
//                 JZKT_ACCOUNT_COMPRESSION_FLAGS,
//                 is_cold,
//             ))
//         } else {
//             let index = U256::from_le_bytes(*key);
//             let (value, is_cold) =
//                 AccountManager::storage(self, EVM_STORAGE_ADDRESS, index, committed);
//             Some((
//                 vec![value.to_le_bytes::<32>()],
//                 JZKT_STORAGE_COMPRESSION_FLAGS,
//                 is_cold,
//             ))
//         }
//     }
//
//     fn update(&self, key: &[u8; 32], value: &Vec<[u8; 32]>, _flags: u32) {
//         if value.len() == JZKT_ACCOUNT_FIELDS_COUNT as usize {
//             let address = Address::from_slice(&key[12..]);
//             let jzkt_account = Account::new_from_fields(address, value.as_slice());
//             AccountManager::write_account(self, &jzkt_account);
//         } else if value.len() == JZKT_STORAGE_FIELDS_COUNT as usize {
//             AccountManager::write_storage(
//                 self,
//                 EVM_STORAGE_ADDRESS,
//                 U256::from_le_bytes(*key),
//                 U256::from_le_bytes(*value.get(0).unwrap()),
//             );
//         } else {
//             panic!("not supported field count: {}", value.len())
//         }
//     }
//
//     fn remove(&self, _key: &[u8; 32]) {
//         // TODO: "account removal is not supported"
//     }
//
//     fn compute_root(&self) -> [u8; 32] {
//         // TODO: "root is not supported"
//         [0u8; 32]
//     }
//
//     fn emit_log(&self, address: Address, topics: Vec<B256>, data: Bytes) {
//         AccountManager::log(self, address, data, &topics);
//     }
//
//     fn commit(&self) -> Result<([u8; 32], Vec<JournalLog>), ExitCode> {
//         AccountManager::commit(self);
//         Ok(([0u8; 32], vec![]))
//     }
//
//     fn rollback(&self, checkpoint: fluentbase_types::JournalCheckpoint) {
//         AccountManager::rollback(self, checkpoint.to_u64());
//     }
//
//     fn update_preimage(&self, key: &[u8; 32], field: u32, preimage: &[u8]) -> bool {
//         AccountManager::update_preimage(self, key, field, preimage);
//         true
//     }
//
//     fn preimage(&self, hash: &[u8; 32]) -> Vec<u8> {
//         AccountManager::preimage(self, hash).to_vec()
//     }
//
//     fn preimage_size(&self, hash: &[u8; 32]) -> u32 {
//         AccountManager::preimage_size(self, hash)
//     }
//
//     fn journal(&self) -> Vec<JournalEvent> {
//         // TODO: "journal is not supported here"
//         vec![]
//     }
// }

impl<'a, DB: Database> AccountManager for JournalDbWrapper<'a, DB> {
    fn checkpoint(&self) -> AccountCheckpoint {
        let mut ctx = self.ctx.borrow_mut();
        let (a, b) = ctx.journaled_state.checkpoint().into();
        fluentbase_types::JournalCheckpoint::from((a, b)).to_u64()
    }

    fn commit(&self) {
        let mut ctx = self.ctx.borrow_mut();
        ctx.journaled_state.checkpoint_commit();
    }

    fn rollback(&self, checkpoint: AccountCheckpoint) {
        let checkpoint = fluentbase_types::JournalCheckpoint::from_u64(checkpoint);
        let mut ctx = self.ctx.borrow_mut();
        ctx.journaled_state
            .checkpoint_revert((checkpoint.0, checkpoint.1).into());
    }

    fn account(&self, address: Address) -> (Account, bool) {
        let mut ctx = self.ctx.borrow_mut();
        let (account, is_cold) = ctx
            .load_account(address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        let mut account = Account::from(account.info.clone());
        account.address = address;
        (account, is_cold)
    }

    fn write_account(&self, account: &Account) {
        let mut ctx = self.ctx.borrow_mut();
        // load account with this address from journaled state
        let (db_account, _) = ctx
            .load_account_with_code(account.address)
            .map_err(|_| panic!("database error"))
            .unwrap();
        // copy all account info fields
        db_account.info.balance = account.balance;
        db_account.info.nonce = account.nonce;
        db_account.info.code_hash = account.source_code_hash;
        db_account.info.rwasm_code_hash = account.rwasm_code_hash;
        // mark account as touched
        ctx.journaled_state.touch(&account.address);
    }

    fn preimage_size(&self, hash: &[u8; 32]) -> u32 {
        self.ctx
            .borrow_mut()
            .db
            .code_by_hash(B256::from(hash))
            .map(|b| b.bytecode().len() as u32)
            .unwrap_or_default()
    }

    fn preimage(&self, hash: &[u8; 32]) -> Bytes {
        let mut ctx = self.ctx.borrow_mut();
        ctx.code_by_hash(B256::from(hash))
            .map_err(|_| panic!("failed to get bytecode by hash"))
            .unwrap()
    }

    fn update_preimage(&self, key: &[u8; 32], field: u32, preimage: &[u8]) {
        let mut ctx = self.ctx.borrow_mut();
        let address = Address::from_slice(&key[12..]);
        debug_log!("am: update_preimage for address {}", address);
        if field == JZKT_ACCOUNT_SOURCE_CODE_HASH_FIELD && !preimage.is_empty() {
            ctx.journaled_state.set_code(
                address,
                Bytecode::new_raw(Bytes::copy_from_slice(preimage)),
                None,
            );
        } else if field == JZKT_ACCOUNT_RWASM_CODE_HASH_FIELD && !preimage.is_empty() {
            ctx.journaled_state.set_rwasm_code(
                address,
                Bytecode::new_raw(Bytes::copy_from_slice(preimage)),
                None,
            );
        }
    }

    fn storage(&self, address: Address, slot: U256, committed: bool) -> (U256, bool) {
        let mut ctx = self.ctx.borrow_mut();
        // let (address, slot) = if address != EVM_STORAGE_ADDRESS {
        //     // let storage_key = calc_storage_key(&address, slot.as_le_slice().as_ptr());
        //     // (EVM_STORAGE_ADDRESS, U256::from_le_bytes(storage_key))
        //     (address, slot)
        // } else {
        //     (address, slot)
        // };
        if committed {
            let (account, _) = ctx
                .load_account(address)
                .map_err(|_| panic!("failed to load account"))
                .unwrap();
            if account.is_created() {
                return (U256::ZERO, true);
            }
            let value = ctx
                .db
                .storage(address, slot)
                .ok()
                .expect("failed to read storage slot");
            (value, true)
        } else {
            ctx.sload(address, slot)
                .ok()
                .expect("failed to read storage slot")
        }
    }

    fn write_storage(&self, address: Address, slot: U256, value: U256) -> bool {
        let mut ctx = self.ctx.borrow_mut();
        // let (address, slot) = if address != EVM_STORAGE_ADDRESS {
        //     // let storage_key = calc_storage_key(&address, slot.as_le_slice().as_ptr());
        //     // (EVM_STORAGE_ADDRESS, U256::from_le_bytes(storage_key))
        //     (address, slot)
        // } else {
        //     (address, slot)
        // };
        // println!(
        //     "write_storage: address {} slot {} value {}",
        //     &address, &slot, &value
        // );
        let result = ctx
            .sstore(address, slot, value)
            .map_err(|_| panic!("failed to update storage slot"))
            .unwrap();
        result.is_cold
    }

    fn log(&self, address: Address, data: Bytes, topics: &[B256]) {
        let mut ctx = self.ctx.borrow_mut();
        ctx.journaled_state.log(Log {
            address,
            data: LogData::new_unchecked(topics.into(), data),
        });
    }

    fn exec_hash(
        &self,
        hash32_offset: *const u8,
        context: &[u8],
        input: &[u8],
        fuel_offset: *mut u32,
        state: u32,
    ) -> (Bytes, i32) {
        use fluentbase_runtime::{Runtime, RuntimeContext};
        let hash32: [u8; 32] = unsafe { &*core::ptr::slice_from_raw_parts(hash32_offset, 32) }
            .try_into()
            .unwrap();
        let rwasm_bytecode = AccountManager::preimage(self, &hash32);
        if rwasm_bytecode.is_empty() {
            return (Bytes::default(), ExitCode::Ok.into_i32());
        }
        let mut ctx = self.ctx.borrow_mut();
        let jzkt = JournalDbWrapper {
            ctx: RefCell::new(&mut ctx),
        };
        let ctx = RuntimeContext::new(rwasm_bytecode)
            .with_input(input.into())
            .with_context(context.into())
            .with_fuel_limit(unsafe { *fuel_offset } as u64)
            .with_jzkt(jzkt)
            .with_state(state);
        let mut runtime = Runtime::new(ctx);
        let result = match runtime.call() {
            Ok(result) => result,
            Err(err) => {
                let exit_code = Runtime::catch_trap(&err);
                println!("execution failed with err: {:?}", err);
                return (Bytes::default(), exit_code);
            }
        };
        unsafe {
            *fuel_offset -= result.fuel_consumed as u32;
        }
        (Bytes::from(result.output.clone()), result.exit_code.into())
    }

    fn inc_nonce(&self, account: &mut Account) -> Option<u64> {
        let mut ctx = self.ctx.borrow_mut();
        let new_nonce = ctx.journaled_state.inc_nonce(account.address)?;
        account.nonce += 1;
        Some(new_nonce - 1)
    }

    fn transfer(&self, from: &mut Account, to: &mut Account, value: U256) -> Result<(), ExitCode> {
        Account::transfer(from, to, value)?;
        let mut ctx = self.ctx.borrow_mut();
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

    fn precompile(
        &self,
        address: &Address,
        input: &Bytes,
        gas: u64,
    ) -> Option<EvmCallMethodOutput> {
        let mut ctx = self.ctx.borrow_mut();
        let result = ctx.call_precompile(*address, input, Gas::new(gas))?;
        Some(EvmCallMethodOutput {
            output: result.output,
            exit_code: exit_code_from_evm_error(result.result).into_i32(),
            gas_remaining: result.gas.remaining(),
            gas_refund: result.gas.refunded(),
        })
    }

    fn is_precompile(&self, address: &Address) -> bool {
        let ctx = self.ctx.borrow_mut();
        ctx.journaled_state
            .warm_preloaded_addresses
            .contains(address)
    }

    fn self_destruct(&self, address: Address, target: Address) -> [bool; 4] {
        let mut ctx = self.ctx.borrow_mut();
        let result = ctx
            .selfdestruct(address, target)
            .map_err(|_| "unexpected EVM self destruct error")
            .unwrap();
        [
            result.had_value,
            result.target_exists,
            result.is_cold,
            result.previously_destroyed,
        ]
    }

    fn block_hash(&self, number: U256) -> B256 {
        let mut ctx = self.ctx.borrow_mut();
        ctx.block_hash(number)
            .map_err(|_| "unexpected EVM error")
            .unwrap()
    }

    fn write_transient_storage(&self, address: Address, index: U256, value: U256) {
        let mut ctx = self.ctx.borrow_mut();
        ctx.tstore(address, index, value)
    }

    fn transient_storage(&self, address: Address, index: U256) -> U256 {
        let mut ctx = self.ctx.borrow_mut();
        ctx.tload(address, index)
    }

    fn mark_account_created(&self, address: Address) {
        let mut ctx = self.ctx.borrow_mut();
        let (account, _) = ctx
            .load_account(address)
            .map_err(|_| "unexpected EVM error")
            .unwrap();
        account.mark_created();
        let last_journal = ctx.journaled_state.journal.last_mut().unwrap();
        last_journal.push(JournalEntry::AccountCreated { address });
    }
}
