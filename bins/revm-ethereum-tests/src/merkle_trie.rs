use hash_db::Hasher;
use plain_hasher::PlainHasher;

use triehash::sec_trie_root;

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use rlp::RlpStream;
use sha3::{Digest, Keccak256};

use revm::AccountInfo;
use hashbrown::HashMap as Map;

pub fn merkle_trie_root(
    accounts: &Map<H160, AccountInfo>,
    storage: &Map<H160, Map<H256, H256>>,
) -> H256 {
    let vec = accounts
        .iter()
        .map(|(address, info)| {
            let storage = storage.get(address).cloned().unwrap_or_default();
            let storage_root = trie_account_rlp(info, storage);
            (address.clone(), storage_root)
        })
        .collect();

    trie_root(vec)
}

/// Returns the RLP for this account.
pub fn trie_account_rlp(info: &AccountInfo, storage: Map<H256, H256>) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&info.nonce);
    stream.append(&info.balance);
    stream.append(&{
        let storage_root = sec_trie_root::<KeccakHasher, _, _, _>(
            storage
                .into_iter()
                .filter(|(_k, v)| v != &H256::zero())
                .map(|(k, v)| (k, rlp::encode(&U256::from(v.as_ref() as &[u8])))),
        );
        storage_root.clone()
    });
    stream.append(&info.code_hash.as_bytes());
    stream.out().freeze()
}

pub fn trie_root(acc_data: Vec<(H160, Bytes)>) -> H256 {
    sec_trie_root::<KeccakHasher, _, _, _>(acc_data.into_iter())
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;
impl Hasher for KeccakHasher {
    type Out = H256;
    type StdHasher = PlainHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        let out = Keccak256::digest(x);
        H256::from_slice(out.as_slice())
    }
}
