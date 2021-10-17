use std::env;

mod model;

use bytes::Bytes;
use hash_db::Hasher;
use model::{AccountInfo, State};
use plain_hasher::PlainHasher;
use primitive_types::{H160, H256, U256};
use rlp::RlpStream;

use sha3::{Digest, Keccak256};
use triehash::sec_trie_root;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(2).cloned().unwrap_or("./storage.json".to_owned());

    let json_reader = std::fs::read(&path).unwrap();
    let state: State = serde_json::from_reader(&*json_reader).unwrap();
    //println!("state:{:?}", state);
    let root = merkelize(state);
    println!("MERKLE ROOT:{:?}", hex::encode(root.as_ref()));
}

pub fn merkelize(state: State) -> H256 {
    let vec: Vec<_> = state
        .0
        .into_iter()
        .map(|(address, acc)| {
            let storage_root = trie_account_rlp(acc);
            (address.clone(), storage_root)
        })
        .collect();

    trie_root(vec)
}

/// Returns the RLP for this account.
pub fn trie_account_rlp(info: AccountInfo) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    let code = info.code;
    let code_hash = Keccak256::digest(&code);
    let storage = info.storage;
    stream.append(&info.nonce);
    stream.append(&info.balance);
    stream.append(&{
        let storage_root = sec_trie_root::<KeccakHasher, _, _, _>(
            storage
                .into_iter()
                .filter(|(_k, v)| v != &H256::zero())
                .map(|(k, v)| (k, rlp::encode(&U256::from(v.as_ref() as &[u8])))),
        );
        storage_root
    });
    stream.append(&code_hash.as_slice());
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

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;

    #[test]
    fn smoke_test() {
        let input = r#"{
            "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba": {
                "balance": "0x693CA",
                "nonce": 0,
                "code": "",
                "storage": {}
            },
            "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
                "balance": "0x3635C9ADC5DE996C36",
                "nonce": 1,
                "code": "",
                "storage": {}
            },
            "0x1000000000000000000000000000000000000000": {
                "balance": "0x0",
                "nonce": 0,
                "code": "0x4660015500",
                "storage": {
                    "0x0000000000000000000000000000000000000000000000000000000000000001": "0x0000000000000000000000000000000000000000000000000000000000000001"
                }
            }
        }"#;

        let state: State = serde_json::from_str(input).unwrap();
        println!("state:{:?}", state);
        let root = merkelize(state);
        assert_eq!(
            root,
            H256::from_str("0xfe997308d1945d7632f7410775724c073765af24af155c71daac753d546bb870")
                .unwrap()
        );
    }
}
