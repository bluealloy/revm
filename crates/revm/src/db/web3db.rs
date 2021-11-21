use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use crate::{AccountInfo, Database, KECCAK_EMPTY};
use tokio::runtime::{Handle, Runtime};
use web3::{
    transports::Http,
    types::{BlockId, BlockNumber, H160 as wH160, U256 as wU256, U64 as wU64},
    Web3,
};

pub struct Web3DB {
    web3: Web3<Http>,
    runtime: Option<Runtime>,
    block_number: Option<BlockNumber>,
}

impl Web3DB {
    /// create web3 db connector inputs are url and block on what we are basing our database (None for latest)
    pub fn new(url: &str, block_number: Option<u64>) -> Option<Self> {
        let runtime = Handle::try_current()
            .is_err()
            .then(|| Runtime::new().unwrap());
        let transport = web3::transports::Http::new(url).ok()?;
        let web3 = Web3::new(transport);

        let mut out = Self {
            web3,
            runtime,
            block_number: None,
        };
        let bnum = if block_number.is_none() {
            out.block_on(out.web3.eth().block_number()).ok()?
        } else {
            block_number.unwrap().into()
        };
        out.block_number = Some(BlockNumber::Number(bnum));
        Some(out)
    }

    /// internal utility function to call tokio feature and wait for output
    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        match &self.runtime {
            Some(runtime) => runtime.block_on(f),
            None => futures::executor::block_on(f),
        }
    }
}

impl Database for Web3DB {
    fn basic(&mut self, address: H160) -> AccountInfo {
        let add = wH160(address.0);
        let f = async {
            let nonce = self.web3.eth().transaction_count(add, self.block_number);
            let balance = self.web3.eth().balance(add, self.block_number);
            let code = self.web3.eth().code(add, self.block_number);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = self.block_on(f);
        // panic on not getting data?
        let acc = AccountInfo::new(
            U256(balance.unwrap().0),
            nonce.unwrap().as_u64(),
            Bytes::from(code.unwrap_or_default().0),
        );
        acc
    }

    fn code_by_hash(&mut self, _code_hash: primitive_types::H256) -> bytes::Bytes {
        panic!("Should not be called. Code is already loaded");
        // not needed because we already load code with basic info
    }

    fn storage(
        &mut self,
        address: primitive_types::H160,
        index: primitive_types::U256,
    ) -> primitive_types::U256 {
        let add = wH160(address.0);
        let index = wU256(index.0);
        let f = async {
            let storage = self
                .web3
                .eth()
                .storage(add, index, self.block_number)
                .await
                .unwrap();
            U256::from_big_endian(storage.as_bytes())
        };
        let storage = self.block_on(f);
        storage
    }

    fn block_hash(&mut self, number: primitive_types::U256) -> primitive_types::H256 {
        if number > U256::from(u64::MAX) {
            return KECCAK_EMPTY;
        }
        let number = number.as_u64();
        if let Some(block_num) = self.block_number {
            match block_num {
                BlockNumber::Number(t) if t.as_u64() > number => return KECCAK_EMPTY,
                _ => (),
            }
        }
        let number = wU64::from(number);
        let f = async {
            self.web3
                .eth()
                .block(BlockId::Number(BlockNumber::Number(number)))
                .await
                .ok()
                .flatten()
        };
        H256(self.block_on(f).unwrap().hash.unwrap().0)
    }
}
