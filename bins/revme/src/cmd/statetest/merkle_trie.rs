use std::convert::Infallible;

use alloy_trie::{root::storage_root_unhashed, HashBuilder, Nibbles, TrieAccount};
use revm::{
    context::result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction},
    database::{bal::EvmDatabaseError, EmptyDB, PlainAccount, State},
    primitives::{keccak256, Address, Log, B256},
};

pub struct TestValidationResult {
    pub logs_root: B256,
    pub state_root: B256,
}

pub fn compute_test_roots(
    exec_result: &Result<
        ExecutionResult<HaltReason>,
        EVMError<EvmDatabaseError<Infallible>, InvalidTransaction>,
    >,
    db: &State<EmptyDB>,
) -> TestValidationResult {
    TestValidationResult {
        logs_root: log_rlp_hash(exec_result.as_ref().map(|r| r.logs()).unwrap_or_default()),
        state_root: state_merkle_trie_root(db.cache.trie_account()),
    }
}

pub fn log_rlp_hash(logs: &[Log]) -> B256 {
    let mut out = Vec::with_capacity(alloy_rlp::list_length(logs));
    alloy_rlp::encode_list(logs, &mut out);
    keccak256(&out)
}

pub fn state_merkle_trie_root<'a>(
    accounts: impl IntoIterator<Item = (Address, &'a PlainAccount)>,
) -> B256 {
    let mut vec: Vec<_> = accounts
        .into_iter()
        .map(|(address, acc)| {
            let storage_root = storage_root_unhashed(
                acc.storage
                    .iter()
                    .filter(|(_k, &v)| !v.is_zero())
                    .map(|(k, v)| (B256::from(*k), *v)),
            );
            (
                keccak256(address),
                TrieAccount {
                    nonce: acc.info.nonce,
                    balance: acc.info.balance,
                    storage_root,
                    code_hash: acc.info.code_hash,
                },
            )
        })
        .collect();
    vec.sort_unstable_by_key(|(key, _)| *key);

    let mut hb = HashBuilder::default();
    let mut account_rlp_buf = Vec::new();
    for (hashed_key, account) in vec {
        account_rlp_buf.clear();
        alloy_rlp::Encodable::encode(&account, &mut account_rlp_buf);
        hb.add_leaf(Nibbles::unpack(hashed_key), &account_rlp_buf);
    }
    hb.root()
}
