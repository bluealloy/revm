use crate::{
    interpreter::bytecode::Bytecode, AccountInfo, Database, B160, B256, KECCAK_EMPTY, U256,
};
use bytes::Bytes;
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
        let bnum = if let Some(block_number) = block_number {
            block_number.into()
        } else {
            out.block_on(out.web3.eth().block_number()).ok()?
        };

        out.block_number = Some(BlockNumber::Number(bnum));
        Some(out)
    }

    /// internal utility function to call tokio feature and wait for output
    fn block_on<F: core::future::Future>(&self, f: F) -> F::Output {
        match &self.runtime {
            Some(runtime) => runtime.block_on(f),
            None => futures::executor::block_on(f),
        }
    }
}

impl Database for Web3DB {
    type Error = ();

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        let add = wH160::from(address.0);
        let f = async {
            let nonce = self.web3.eth().transaction_count(add, self.block_number);
            let balance = self.web3.eth().balance(add, self.block_number);
            let code = self.web3.eth().code(add, self.block_number);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = self.block_on(f);
        // panic on not getting data?
        Ok(Some(AccountInfo::new(
            U256::from_limbs(
                balance
                    .unwrap_or_else(|e| panic!("web3 get balance error:{e:?}"))
                    .0,
            ),
            nonce
                .unwrap_or_else(|e| panic!("web3 get nonce error:{e:?}"))
                .as_u64(),
            Bytecode::new_raw(Bytes::from(
                code.unwrap_or_else(|e| panic!("web3 get node error:{e:?}"))
                    .0,
            )),
        )))
    }

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        panic!("Should not be called. Code is already loaded");
        // not needed because we already load code with basic info
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        let add = wH160::from(address.0);
        let index = wU256(*index.as_limbs());
        let f = async {
            let storage = self
                .web3
                .eth()
                .storage(add, index, self.block_number)
                .await
                .unwrap();
            U256::from_be_bytes(storage.to_fixed_bytes())
        };
        Ok(self.block_on(f))
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        // saturate usize
        if number > U256::from(u64::MAX) {
            return Ok(KECCAK_EMPTY);
        }
        let number = wU64::from(u64::try_from(number).unwrap());
        let f = async {
            self.web3
                .eth()
                .block(BlockId::Number(BlockNumber::Number(number)))
                .await
                .ok()
                .flatten()
        };
        Ok(B256(self.block_on(f).unwrap().hash.unwrap().0))
    }
}
