use bytes::Bytes;
use hash_db::Hasher;
use plain_hasher::PlainHasher;
use revm::{db::DbAccount, Log};
use rlp::RlpStream;
use ruint::aliases::{B160, B256, U256};
use sha3::{Digest, Keccak256};
use triehash::sec_trie_root;

pub fn log_rlp_hash(logs: Vec<Log>) -> B256 {
    //https://github.com/ethereum/go-ethereum/blob/356bbe343a30789e77bb38f25983c8f2f2bfbb47/cmd/evm/internal/t8ntool/execution.go#L255
    let mut stream = RlpStream::new();
    stream.begin_unbounded_list();
    for log in logs {
        stream.begin_list(3);
        stream.append(&log.address);
        stream.append_list(&log.topics);
        stream.append(&log.data);
    }
    stream.finalize_unbounded_list();
    let out = stream.out().freeze();

    let out = Keccak256::digest(out);
    B256::try_from_be_slice(out.as_slice()).unwrap()
}

pub fn state_merkle_trie_root(accounts: impl Iterator<Item = (B160, DbAccount)>) -> B256 {
    let vec = accounts
        .map(|(address, info)| {
            let acc_root = trie_account_rlp(&info);
            (address, acc_root)
        })
        .collect();

    trie_root(vec)
}

/// Returns the RLP for this account.
pub fn trie_account_rlp(acc: &DbAccount) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&acc.info.nonce);
    stream.append(&acc.info.balance);
    stream.append(&{
        sec_trie_root::<KeccakHasher, _, _, _>(
            acc.storage
                .iter()
                .filter(|(_k, &v)| v != U256::ZERO)
                .map(|(k, v)| (B256::from(k.to_be_bytes()), rlp::encode(v))),
        )
    });
    stream.append(&acc.info.code_hash.to_be_bytes_vec());
    stream.out().freeze()
}

pub fn trie_root(acc_data: Vec<(B160, Bytes)>) -> B256 {
    sec_trie_root::<KeccakHasher, _, _, _>(acc_data.into_iter())
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
    type Out = B256;
    type StdHasher = PlainHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        let out = Keccak256::digest(x);
        B256::try_from_be_slice(out.as_slice()).unwrap()
    }
}
