use hash_db::Hasher;
use plain_hasher::PlainHasher;

use triehash::sec_trie_root;

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use rlp::RlpStream;
use sha3::{Digest, Keccak256};

use hashbrown::HashMap as Map;
use revm::AccountInfo;

pub fn merkle_trie_root(
    accounts: &Map<H160, AccountInfo>,
    storage: &Map<H160, Map<U256, U256>>,
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
pub fn trie_account_rlp(info: &AccountInfo, storage: Map<U256, U256>) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&info.nonce);
    stream.append(&info.balance);
    stream.append(&{
        let storage_root = sec_trie_root::<KeccakHasher, _, _, _>(
            storage
                .into_iter()
                .filter(|(_k, v)| v != &U256::zero())
                .map(|(k, v)| {
                    let mut temp : [u8;32] = [0;32];
                    k.to_big_endian(&mut temp);
                    (H256::from(temp), rlp::encode(&v))
                }),
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
