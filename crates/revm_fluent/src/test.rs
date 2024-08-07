use crate::{
    primitives::{
        hex,
        keccak256,
        AccountInfo,
        Bytecode,
        Env,
        ExecutionResult,
        HashMap,
        Output,
        TransactTo,
        KECCAK_EMPTY,
        POSEIDON_EMPTY,
    },
    DatabaseCommit,
    Evm,
    InMemoryDB,
};
use core::{mem::take, str::from_utf8};
use fluentbase_genesis::{
    devnet::{devnet_genesis_from_file, KECCAK_HASH_KEY, POSEIDON_HASH_KEY},
    Genesis,
    EXAMPLE_GREETING_ADDRESS,
};
use fluentbase_runtime::RuntimeContext;
use fluentbase_sdk::{
    codec::{BufferDecoder, Encoder},
    poseidon_hash,
    runtime::TestingContext,
    types::EvmCallMethodInput,
};
use fluentbase_types::{
    address,
    bytes,
    calc_create_address,
    Account,
    Address,
    Bytes,
    ExitCode,
    NativeAPI,
    SysFuncIdx,
    U256,
};
use rwasm::{
    instruction_set,
    rwasm::{BinaryFormat, RwasmModule},
};

#[allow(dead_code)]
struct EvmTestingContext {
    sdk: TestingContext,
    genesis: Genesis,
    db: InMemoryDB,
}

impl Default for EvmTestingContext {
    fn default() -> Self {
        Self::load_from_genesis(devnet_genesis_from_file())
    }
}

#[allow(dead_code)]
impl EvmTestingContext {
    fn load_from_genesis(genesis: Genesis) -> Self {
        // create jzkt and put it into testing context
        let mut db = InMemoryDB::default();
        // convert all accounts from genesis into jzkt
        for (k, v) in genesis.alloc.iter() {
            let poseidon_hash = v
                .storage
                .as_ref()
                .and_then(|v| v.get(&POSEIDON_HASH_KEY).cloned())
                .unwrap_or_else(|| {
                    v.code
                        .as_ref()
                        .map(|v| poseidon_hash(&v).into())
                        .unwrap_or(POSEIDON_EMPTY)
                });
            let keccak_hash = v
                .storage
                .as_ref()
                .and_then(|v| v.get(&KECCAK_HASH_KEY).cloned())
                .unwrap_or_else(|| {
                    v.code
                        .as_ref()
                        .map(|v| keccak256(&v))
                        .unwrap_or(KECCAK_EMPTY)
                });
            let account = Account {
                address: *k,
                balance: v.balance,
                nonce: v.nonce.unwrap_or_default(),
                // it makes not much sense to fill these fields, but it reduces hash calculation
                // time a bit
                source_code_size: v.code.as_ref().map(|v| v.len() as u64).unwrap_or_default(),
                source_code_hash: keccak_hash,
                rwasm_code_size: v.code.as_ref().map(|v| v.len() as u64).unwrap_or_default(),
                rwasm_code_hash: poseidon_hash,
            };
            let mut info: AccountInfo = account.into();
            info.code = v.code.clone().map(Bytecode::new_raw);
            info.rwasm_code = v.code.clone().map(Bytecode::new_raw);
            db.insert_account_info(*k, info);
        }
        Self {
            sdk: TestingContext::new(RuntimeContext::default()),
            genesis,
            db,
        }
    }

    pub(crate) fn add_wasm_contract<I: Into<RwasmModule>>(
        &mut self,
        address: Address,
        rwasm_module: I,
    ) -> AccountInfo {
        let rwasm_binary = {
            let rwasm_module: RwasmModule = rwasm_module.into();
            let mut result = Vec::new();
            rwasm_module.write_binary_to_vec(&mut result).unwrap();
            result
        };
        let account = Account {
            address,
            balance: U256::ZERO,
            nonce: 0,
            // it makes not much sense to fill these fields, but it optimizes hash calculation a bit
            source_code_size: 0,
            source_code_hash: KECCAK_EMPTY,
            rwasm_code_size: rwasm_binary.len() as u64,
            rwasm_code_hash: poseidon_hash(&rwasm_binary).into(),
        };
        let mut info: AccountInfo = account.into();
        info.code = None;
        if !rwasm_binary.is_empty() {
            info.rwasm_code = Some(Bytecode::new_raw(rwasm_binary.into()));
        }
        self.db.insert_account_info(address, info.clone());
        info
    }

    pub(crate) fn get_balance(&mut self, address: Address) -> U256 {
        let account = self.db.load_account(address).unwrap();
        account.info.balance
    }

    pub(crate) fn add_balance(&mut self, address: Address, value: U256) {
        let account = self.db.load_account(address).unwrap();
        account.info.balance += value;
        let mut revm_account = crate::primitives::Account::from(account.info.clone());
        revm_account.mark_touch();
        self.db.commit(HashMap::from([(address, revm_account)]));
    }
}

struct TxBuilder<'a> {
    pub(crate) ctx: &'a mut EvmTestingContext,
    pub(crate) env: Env,
}

#[allow(dead_code)]
impl<'a> TxBuilder<'a> {
    fn create(ctx: &'a mut EvmTestingContext, deployer: Address, init_code: Bytes) -> Self {
        let mut env = Env::default();
        env.tx.caller = deployer;
        env.tx.transact_to = TransactTo::Create;
        env.tx.data = init_code;
        env.tx.gas_limit = 300_000_000;
        Self { ctx, env }
    }

    fn call(ctx: &'a mut EvmTestingContext, caller: Address, callee: Address) -> Self {
        let mut env = Env::default();
        env.tx.gas_price = U256::from(1);
        env.tx.caller = caller;
        env.tx.transact_to = TransactTo::Call(callee);
        env.tx.gas_limit = 10_000_000;
        Self { ctx, env }
    }

    fn input(mut self, input: Bytes) -> Self {
        self.env.tx.data = input;
        self
    }

    fn value(mut self, value: U256) -> Self {
        self.env.tx.value = value;
        self
    }

    fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.env.tx.gas_limit = gas_limit;
        self
    }

    fn gas_price(mut self, gas_price: U256) -> Self {
        self.env.tx.gas_price = gas_price;
        self
    }

    fn exec(&mut self) -> ExecutionResult {
        let mut evm = Evm::builder()
            .with_env(Box::new(take(&mut self.env)))
            .with_db(&mut self.ctx.db)
            .build();
        evm.transact_commit().unwrap()
    }
}

fn deploy_evm_tx(ctx: &mut EvmTestingContext, deployer: Address, init_bytecode: Bytes) -> Address {
    // deploy greeting EVM contract
    let result = TxBuilder::create(ctx, deployer, init_bytecode.clone().into()).exec();
    assert!(result.is_success());
    let contract_address = calc_create_address(&ctx.sdk, &deployer, 0);
    assert_eq!(contract_address, deployer.create(0));
    let contract_account = ctx.db.accounts.get(&contract_address).unwrap();
    let source_bytecode = ctx
        .db
        .contracts
        .get(&contract_account.info.code_hash)
        .unwrap()
        .original_bytes()
        .to_vec();
    assert_eq!(contract_account.info.code_hash, keccak256(&source_bytecode));
    assert!(source_bytecode.len() > 0);
    // let rwasm_bytecode = ctx
    //     .db
    //     .contracts
    //     .get(&contract_account.info.rwasm_code_hash)
    //     .unwrap()
    //     .bytes()
    //     .to_vec();
    // let is_rwasm = rwasm_bytecode.get(0).cloned().unwrap() == 0xef;
    // assert!(is_rwasm);
    contract_address
}

fn call_evm_tx(
    ctx: &mut EvmTestingContext,
    caller: Address,
    callee: Address,
    input: Bytes,
    gas_limit: Option<u64>,
) -> ExecutionResult {
    ctx.add_balance(caller, U256::from(1e18));
    // call greeting EVM contract
    let mut tx_builder = TxBuilder::call(ctx, caller, callee).input(input);
    if let Some(gas_limit) = gas_limit {
        tx_builder = tx_builder.gas_limit(gas_limit);
    }
    tx_builder.exec()
}

#[test]
#[ignore]
fn test_genesis_greeting() {
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        EXAMPLE_GREETING_ADDRESS,
        Bytes::default(),
        None,
    );
    assert!(result.is_success());
    println!("gas used (call): {}", result.gas_used());
    let bytes = result.output().unwrap_or_default();
    assert_eq!("Hello, World", from_utf8(bytes.as_ref()).unwrap());
}

#[test]
fn test_deploy_greeting() {
    // deploy greeting WASM contract
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let contract_address = deploy_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        include_bytes!("../../../../examples/greeting/lib.wasm").into(),
    );
    // call greeting WASM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address,
        Bytes::default(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default();
    assert_eq!("Hello, World", from_utf8(bytes.as_ref()).unwrap());
}

#[test]
fn test_deploy_keccak256() {
    // deploy greeting WASM contract
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let contract_address = deploy_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        include_bytes!("../../../../examples/hashing/lib.wasm").into(),
    );
    // call greeting WASM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address,
        "Hello, World".into(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default().as_ref();
    assert_eq!(
        "a04a451028d0f9284ce82243755e245238ab1e4ecf7b9dd8bf4734d9ecfd0529",
        hex::encode(&bytes[0..32]),
    );
}

#[test]
fn test_deploy_panic() {
    // deploy greeting WASM contract
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let contract_address = deploy_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        include_bytes!("../../../../examples/panic/lib.wasm").into(),
    );
    // call greeting WASM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address,
        Bytes::default(),
        None,
    );
    assert!(!result.is_success());
    let bytes = result.output().unwrap_or_default();
    assert_eq!(
        "panicked at examples/panic/lib.rs:17:9: it is panic time",
        from_utf8(bytes.as_ref()).unwrap()
    );
}

#[test]
fn test_evm_greeting() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let contract_address = deploy_evm_tx(&mut ctx, DEPLOYER_ADDRESS, hex!("60806040526105ae806100115f395ff3fe608060405234801561000f575f80fd5b506004361061003f575f3560e01c80633b2e97481461004357806345773e4e1461007357806348b8bcc314610091575b5f80fd5b61005d600480360381019061005891906102e5565b6100af565b60405161006a919061039a565b60405180910390f35b61007b6100dd565b604051610088919061039a565b60405180910390f35b61009961011a565b6040516100a6919061039a565b60405180910390f35b60605f8273ffffffffffffffffffffffffffffffffffffffff163190506100d58161012f565b915050919050565b60606040518060400160405280600b81526020017f48656c6c6f20576f726c64000000000000000000000000000000000000000000815250905090565b60605f4790506101298161012f565b91505090565b60605f8203610175576040518060400160405280600181526020017f30000000000000000000000000000000000000000000000000000000000000008152509050610282565b5f8290505f5b5f82146101a457808061018d906103f0565b915050600a8261019d9190610464565b915061017b565b5f8167ffffffffffffffff8111156101bf576101be610494565b5b6040519080825280601f01601f1916602001820160405280156101f15781602001600182028036833780820191505090505b5090505b5f851461027b578180610207906104c1565b925050600a8561021791906104e8565b60306102239190610518565b60f81b8183815181106102395761023861054b565b5b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff191690815f1a905350600a856102749190610464565b94506101f5565b8093505050505b919050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102b48261028b565b9050919050565b6102c4816102aa565b81146102ce575f80fd5b50565b5f813590506102df816102bb565b92915050565b5f602082840312156102fa576102f9610287565b5b5f610307848285016102d1565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f5b8381101561034757808201518184015260208101905061032c565b5f8484015250505050565b5f601f19601f8301169050919050565b5f61036c82610310565b610376818561031a565b935061038681856020860161032a565b61038f81610352565b840191505092915050565b5f6020820190508181035f8301526103b28184610362565b905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f819050919050565b5f6103fa826103e7565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff820361042c5761042b6103ba565b5b600182019050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f61046e826103e7565b9150610479836103e7565b92508261048957610488610437565b5b828204905092915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b5f6104cb826103e7565b91505f82036104dd576104dc6103ba565b5b600182039050919050565b5f6104f2826103e7565b91506104fd836103e7565b92508261050d5761050c610437565b5b828206905092915050565b5f610522826103e7565b915061052d836103e7565b9250828201905080821115610545576105446103ba565b5b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52603260045260245ffdfea2646970667358221220feebf5ace29c3c3146cb63bf7ca9009c2005f349075639d267cfbd817adde3e564736f6c63430008180033").into());
    // call greeting EVM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address,
        hex!("45773e4e").into(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default();
    let bytes = &bytes[64..75];
    assert_eq!("Hello World", from_utf8(bytes.as_ref()).unwrap());
}

///
/// Test storage though constructor
///
/// ```solidity
/// // SPDX-License-Identifier: MIT
/// pragma solidity 0.8.24;
///
/// contract Storage {
///     event Test(uint256);
///     uint256 private value;
///     mapping(address => uint256) private balances;
///     mapping(address => mapping(address => uint256)) private allowances;
///     constructor() payable {
///         value = 100;
///         balances[msg.sender] = 100;
///         allowances[msg.sender][address(this)] = 100;
///     }
///     function setValue(uint256 newValue) public {
///         value = newValue;
///         balances[msg.sender] = newValue;
///         allowances[msg.sender][address(this)] = newValue;
///         emit Test(value);
///     }
///     function getValue() public view returns (uint256) {
///         require(balances[msg.sender] == value, "value mismatch");
///         require(allowances[msg.sender][address(this)] == value, "value mismatch");
///         return value;
///     }
/// }
/// ```
#[test]
fn test_evm_storage() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const DEPLOYER_ADDRESS: Address = Address::ZERO;
    let contract_address_1 = deploy_evm_tx(&mut ctx, DEPLOYER_ADDRESS, hex!("608060405260645f81905550606460015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550606460025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055506103ed806100d95f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c806320965255146100385780635524107714610056575b5f80fd5b610040610072565b60405161004d91906102cd565b60405180910390f35b610070600480360381019061006b9190610314565b6101b5565b005b5f805460015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054146100f3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100ea90610399565b60405180910390fd5b5f5460025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054146101ae576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101a590610399565b60405180910390fd5b5f54905090565b805f819055508060015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508060025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055507f63a242a632efe33c0e210e04e4173612a17efa4f16aa4890bc7e46caece80de05f546040516102aa91906102cd565b60405180910390a150565b5f819050919050565b6102c7816102b5565b82525050565b5f6020820190506102e05f8301846102be565b92915050565b5f80fd5b6102f3816102b5565b81146102fd575f80fd5b50565b5f8135905061030e816102ea565b92915050565b5f60208284031215610329576103286102e6565b5b5f61033684828501610300565b91505092915050565b5f82825260208201905092915050565b7f76616c7565206d69736d617463680000000000000000000000000000000000005f82015250565b5f610383600e8361033f565b915061038e8261034f565b602082019050919050565b5f6020820190508181035f8301526103b081610377565b905091905056fea26469706673582212204d28a306634cc4321dbd572eed851aa320f7b0ee31d73ccdffb30e2fd053355a64736f6c63430008180033").into());
    let contract_address_2 = deploy_evm_tx(&mut ctx, DEPLOYER_ADDRESS, hex!("608060405260645f81905550606460015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550606460025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055506103ed806100d95f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c806320965255146100385780635524107714610056575b5f80fd5b610040610072565b60405161004d91906102cd565b60405180910390f35b610070600480360381019061006b9190610314565b6101b5565b005b5f805460015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054146100f3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100ea90610399565b60405180910390fd5b5f5460025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054146101ae576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101a590610399565b60405180910390fd5b5f54905090565b805f819055508060015f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508060025f3373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f3073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055507f63a242a632efe33c0e210e04e4173612a17efa4f16aa4890bc7e46caece80de05f546040516102aa91906102cd565b60405180910390a150565b5f819050919050565b6102c7816102b5565b82525050565b5f6020820190506102e05f8301846102be565b92915050565b5f80fd5b6102f3816102b5565b81146102fd575f80fd5b50565b5f8135905061030e816102ea565b92915050565b5f60208284031215610329576103286102e6565b5b5f61033684828501610300565b91505092915050565b5f82825260208201905092915050565b7f76616c7565206d69736d617463680000000000000000000000000000000000005f82015250565b5f610383600e8361033f565b915061038e8261034f565b602082019050919050565b5f6020820190508181035f8301526103b081610377565b905091905056fea26469706673582212204d28a306634cc4321dbd572eed851aa320f7b0ee31d73ccdffb30e2fd053355a64736f6c63430008180033").into());
    // call greeting EVM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address_1,
        hex!("20965255").into(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default();
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000000064",
        hex::encode(bytes)
    );
    // call greeting EVM contract
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address_2,
        hex!("20965255").into(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default().iter().as_slice();
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000000064",
        hex::encode(bytes)
    );
    // set value to 0x70
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address_2,
        hex!("552410770000000000000000000000000000000000000000000000000000000000000070").into(),
        None,
    );
    assert!(result.is_success());
    // check result is 0x70
    let result = call_evm_tx(
        &mut ctx,
        DEPLOYER_ADDRESS,
        contract_address_2,
        hex!("20965255").into(),
        None,
    );
    assert!(result.is_success());
    let bytes = result.output().unwrap_or_default().iter().as_slice();
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000000070",
        hex::encode(bytes)
    );
}

#[test]
fn test_simple_send() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("1231238908230948230948209348203984029834");
    const RECIPIENT_ADDRESS: Address = address!("1092381297182319023812093812312309123132");
    ctx.add_balance(SENDER_ADDRESS, U256::from(2e18));
    let gas_price = U256::from(1e9);
    let result = TxBuilder::call(&mut ctx, SENDER_ADDRESS, RECIPIENT_ADDRESS)
        .gas_price(gas_price)
        .value(U256::from(1e18))
        .exec();
    assert!(result.is_success());
    let tx_cost = gas_price * U256::from(result.gas_used());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18) - tx_cost);
    assert_eq!(ctx.get_balance(RECIPIENT_ADDRESS), U256::from(1e18));
}

#[test]
fn test_create_send() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("1231238908230948230948209348203984029834");
    ctx.add_balance(SENDER_ADDRESS, U256::from(2e18));
    let gas_price = U256::from(2e9);
    let result = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        include_bytes!("../../../../examples/greeting/lib.wasm").into(),
    )
    .gas_price(gas_price)
    .value(U256::from(1e18))
    .exec();
    let contract_address = calc_create_address(&ctx.sdk, &SENDER_ADDRESS, 0);
    assert!(result.is_success());
    let tx_cost = gas_price * U256::from(result.gas_used());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18) - tx_cost);
    assert_eq!(ctx.get_balance(contract_address), U256::from(1e18));
}

#[test]
fn test_evm_revert() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("1231238908230948230948209348203984029834");
    ctx.add_balance(SENDER_ADDRESS, U256::from(2e18));
    let gas_price = U256::from(0);
    let result = TxBuilder::create(&mut ctx, SENDER_ADDRESS, hex!("5f5ffd").into())
        .gas_price(gas_price)
        .value(U256::from(1e18))
        .exec();
    let contract_address = calc_create_address(&ctx.sdk, &SENDER_ADDRESS, 0);
    assert!(!result.is_success());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(2e18));
    assert_eq!(ctx.get_balance(contract_address), U256::from(0e18));
    // now send success tx
    let result = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        include_bytes!("../../../../examples/greeting/lib.wasm").into(),
    )
    .gas_price(gas_price)
    .value(U256::from(1e18))
    .exec();
    // here nonce must be 1 because we increment nonce for failed txs
    let contract_address = calc_create_address(&ctx.sdk, &SENDER_ADDRESS, 1);
    println!("{}", contract_address);
    assert!(result.is_success());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18));
    assert_eq!(ctx.get_balance(contract_address), U256::from(1e18));
}

#[test]
fn test_evm_self_destruct() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("1231238908230948230948209348203984029834");
    // const DESTROYED_ADDRESS: Address = address!("f91c20c0cafbfdc150adff51bbfc5808edde7cb5");
    ctx.add_balance(SENDER_ADDRESS, U256::from(2e18));
    let gas_price = U256::from(0);
    let result = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        hex!("6003600c60003960036000F36003ff").into(),
    )
    .gas_price(gas_price)
    .value(U256::from(1e18))
    .exec();
    let contract_address = calc_create_address(&ctx.sdk, &SENDER_ADDRESS, 0);
    assert!(result.is_success());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18));
    assert_eq!(ctx.get_balance(contract_address), U256::from(1e18));
    // call self destruct contract
    let result = TxBuilder::call(&mut ctx, SENDER_ADDRESS, contract_address)
        .gas_price(gas_price)
        .exec();
    assert!(result.is_success());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18));
    assert_eq!(ctx.get_balance(contract_address), U256::from(0e18));
    assert_eq!(
        ctx.get_balance(address!("0000000000000000000000000000000000000003")),
        U256::from(1e18)
    );
    // destruct in nested call
    let result = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        hex!("6000600060006000600073f91c20c0cafbfdc150adff51bbfc5808edde7cb561FFFFF1").into(),
    )
    .exec();
    assert!(result.is_success());
    assert_eq!(ctx.get_balance(SENDER_ADDRESS), U256::from(1e18));
    assert_eq!(ctx.get_balance(contract_address), U256::from(0e18));
    assert_eq!(
        ctx.get_balance(address!("0000000000000000000000000000000000000003")),
        U256::from(1e18)
    );
}

#[test]
fn test_bridge_contract() {
    // deploy greeting EVM contract
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("d9b36c6c8bfcc633bb83372db44d80f352cdfe3f");
    ctx.add_balance(SENDER_ADDRESS, U256::from(2e18));
    let gas_price = U256::from(0);
    // now send success tx
    let contract_address = calc_create_address(&ctx.sdk, &SENDER_ADDRESS, 0);
    let mut tx_builder = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        hex!("60806040523480156200001157600080fd5b506040805160208082018352600080835283519182019093529182529060036200003c8382620000f9565b5060046200004b8282620000f9565b505050620001c5565b634e487b7160e01b600052604160045260246000fd5b600181811c908216806200007f57607f821691505b602082108103620000a057634e487b7160e01b600052602260045260246000fd5b50919050565b601f821115620000f457600081815260208120601f850160051c81016020861015620000cf5750805b601f850160051c820191505b81811015620000f057828155600101620000db565b5050505b505050565b81516001600160401b0381111562000115576200011562000054565b6200012d816200012684546200006a565b84620000a6565b602080601f8311600181146200016557600084156200014c5750858301515b600019600386901b1c1916600185901b178555620000f0565b600085815260208120601f198616915b82811015620001965788860151825594840194600190910190840162000175565b5085821015620001b55787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b610bab80620001d56000396000f3fe608060405234801561001057600080fd5b50600436106100cf5760003560e01c806370a082311161008c578063a9059cbb11610066578063a9059cbb146101a2578063c820f146146101b5578063dd62ed3e146101c8578063df1f29ee1461020157600080fd5b806370a082311461015e57806395d89b41146101875780639dc29fac1461018f57600080fd5b806306fdde03146100d4578063095ea7b3146100f257806318160ddd1461011557806323b872dd14610127578063313ce5671461013a57806340c10f1914610149575b600080fd5b6100dc610227565b6040516100e991906107a6565b60405180910390f35b610105610100366004610810565b6102b9565b60405190151581526020016100e9565b6002545b6040519081526020016100e9565b61010561013536600461083a565b6102d3565b604051601281526020016100e9565b61015c610157366004610810565b6102f7565b005b61011961016c366004610876565b6001600160a01b031660009081526020819052604090205490565b6100dc610351565b61015c61019d366004610810565b610360565b6101056101b0366004610810565b6103b1565b61015c6101c336600461093b565b6103bf565b6101196101d63660046109d9565b6001600160a01b03918216600090815260016020908152604080832093909416825291909152205490565b600954600854604080516001600160a01b039384168152929091166020830152016100e9565b60606006805461023690610a0c565b80601f016020809104026020016040519081016040528092919081815260200182805461026290610a0c565b80156102af5780601f10610284576101008083540402835291602001916102af565b820191906000526020600020905b81548152906001019060200180831161029257829003601f168201915b5050505050905090565b6000336102c781858561044c565b60019150505b92915050565b6000336102e185828561045e565b6102ec8585856104dc565b506001949350505050565b6007546001600160a01b031633146103435760405162461bcd60e51b815260206004820152600a60248201526937b7363c9037bbb732b960b11b60448201526064015b60405180910390fd5b61034d828261053b565b5050565b60606005805461023690610a0c565b6007546001600160a01b031633146103a75760405162461bcd60e51b815260206004820152600a60248201526937b7363c9037bbb732b960b11b604482015260640161033a565b61034d8282610571565b6000336102c78185856104dc565b6007546001600160a01b0316156103d557600080fd5b600780546001600160a01b0319163317905560056103f38582610a94565b5060066104008682610a94565b50600880546001600160a01b039283166001600160a01b03199091161790556009805460ff909416600160a01b026001600160a81b031990941692909116919091179190911790555050565b61045983838360016105a7565b505050565b6001600160a01b0383811660009081526001602090815260408083209386168352929052205460001981146104d657818110156104c757604051637dc7a0d960e11b81526001600160a01b0384166004820152602481018290526044810183905260640161033a565b6104d6848484840360006105a7565b50505050565b6001600160a01b03831661050657604051634b637e8f60e11b81526000600482015260240161033a565b6001600160a01b0382166105305760405163ec442f0560e01b81526000600482015260240161033a565b61045983838361067c565b6001600160a01b0382166105655760405163ec442f0560e01b81526000600482015260240161033a565b61034d6000838361067c565b6001600160a01b03821661059b57604051634b637e8f60e11b81526000600482015260240161033a565b61034d8260008361067c565b6001600160a01b0384166105d15760405163e602df0560e01b81526000600482015260240161033a565b6001600160a01b0383166105fb57604051634a1406b160e11b81526000600482015260240161033a565b6001600160a01b03808516600090815260016020908152604080832093871683529290522082905580156104d657826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161066e91815260200190565b60405180910390a350505050565b6001600160a01b0383166106a757806002600082825461069c9190610b54565b909155506107199050565b6001600160a01b038316600090815260208190526040902054818110156106fa5760405163391434e360e21b81526001600160a01b0385166004820152602481018290526044810183905260640161033a565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b03821661073557600280548290039055610754565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161079991815260200190565b60405180910390a3505050565b600060208083528351808285015260005b818110156107d3578581018301518582016040015282016107b7565b506000604082860101526040601f19601f8301168501019250505092915050565b80356001600160a01b038116811461080b57600080fd5b919050565b6000806040838503121561082357600080fd5b61082c836107f4565b946020939093013593505050565b60008060006060848603121561084f57600080fd5b610858846107f4565b9250610866602085016107f4565b9150604084013590509250925092565b60006020828403121561088857600080fd5b610891826107f4565b9392505050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126108bf57600080fd5b813567ffffffffffffffff808211156108da576108da610898565b604051601f8301601f19908116603f0116810190828211818310171561090257610902610898565b8160405283815286602085880101111561091b57600080fd5b836020870160208301376000602085830101528094505050505092915050565b600080600080600060a0868803121561095357600080fd5b853567ffffffffffffffff8082111561096b57600080fd5b61097789838a016108ae565b9650602088013591508082111561098d57600080fd5b5061099a888289016108ae565b945050604086013560ff811681146109b157600080fd5b92506109bf606087016107f4565b91506109cd608087016107f4565b90509295509295909350565b600080604083850312156109ec57600080fd5b6109f5836107f4565b9150610a03602084016107f4565b90509250929050565b600181811c90821680610a2057607f821691505b602082108103610a4057634e487b7160e01b600052602260045260246000fd5b50919050565b601f82111561045957600081815260208120601f850160051c81016020861015610a6d5750805b601f850160051c820191505b81811015610a8c57828155600101610a79565b505050505050565b815167ffffffffffffffff811115610aae57610aae610898565b610ac281610abc8454610a0c565b84610a46565b602080601f831160018114610af75760008415610adf5750858301515b600019600386901b1c1916600185901b178555610a8c565b600085815260208120601f198616915b82811015610b2657888601518255948401946001909101908401610b07565b5085821015610b445787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b808201808211156102cd57634e487b7160e01b600052601160045260246000fdfea264697066735822122020392651e573f9944e7a325289c46a3be569262ab593d01ed253c5598ffd5a9464736f6c63430008140033").into())
        .gas_price(gas_price);
    assert!(!tx_builder.ctx.db.accounts.contains_key(&contract_address));
    let exec_result = tx_builder.exec();
    assert!(tx_builder.ctx.db.accounts.contains_key(&contract_address));
    let contract_account = tx_builder.ctx.db.accounts.get(&contract_address).unwrap();
    assert!(contract_account.info.rwasm_code.is_some());
    assert!(!contract_account.info.rwasm_code_hash.is_zero());
    assert_eq!(contract_account.info.nonce, 1);
    assert!(contract_account.info.code.is_some());
    assert!(!contract_account.info.code_hash.is_zero());
    assert!(exec_result.is_success());
}

#[test]
fn test_bridge_contract2() {
    let mut ctx = EvmTestingContext::default();
    const SENDER_ADDRESS: Address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    let result = TxBuilder::create(
        &mut ctx,
        SENDER_ADDRESS,
        hex!("60806040523480156200001157600080fd5b5060405180602001604052806000815250604051806020016040528060008152508160039081620000439190620002d8565b508060049081620000559190620002d8565b505050620003bf565b600081519050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680620000e057607f821691505b602082108103620000f657620000f562000098565b5b50919050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b600060088302620001607fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000121565b6200016c868362000121565b95508019841693508086168417925050509392505050565b6000819050919050565b6000819050919050565b6000620001b9620001b3620001ad8462000184565b6200018e565b62000184565b9050919050565b6000819050919050565b620001d58362000198565b620001ed620001e482620001c0565b8484546200012e565b825550505050565b600090565b62000204620001f5565b62000211818484620001ca565b505050565b5b8181101562000239576200022d600082620001fa565b60018101905062000217565b5050565b601f82111562000288576200025281620000fc565b6200025d8462000111565b810160208510156200026d578190505b620002856200027c8562000111565b83018262000216565b50505b505050565b600082821c905092915050565b6000620002ad600019846008026200028d565b1980831691505092915050565b6000620002c883836200029a565b9150826002028217905092915050565b620002e3826200005e565b67ffffffffffffffff811115620002ff57620002fe62000069565b5b6200030b8254620000c7565b620003188282856200023d565b600060209050601f8311600181146200035057600084156200033b578287015190505b620003478582620002ba565b865550620003b7565b601f1984166200036086620000fc565b60005b828110156200038a5784890151825560018201915060208501945060208101905062000363565b86831015620003aa5784890151620003a6601f8916826200029a565b8355505b6001600288020188555050505b505050505050565b610e5580620003cf6000396000f3fe608060405234801561001057600080fd5b50600436106100935760003560e01c8063313ce56711610066578063313ce5671461013457806370a082311461015257806395d89b4114610182578063a9059cbb146101a0578063dd62ed3e146101d057610093565b806306fdde0314610098578063095ea7b3146100b657806318160ddd146100e657806323b872dd14610104575b600080fd5b6100a0610200565b6040516100ad9190610aa9565b60405180910390f35b6100d060048036038101906100cb9190610b64565b610292565b6040516100dd9190610bbf565b60405180910390f35b6100ee6102b5565b6040516100fb9190610be9565b60405180910390f35b61011e60048036038101906101199190610c04565b6102bf565b60405161012b9190610bbf565b60405180910390f35b61013c6102ee565b6040516101499190610c73565b60405180910390f35b61016c60048036038101906101679190610c8e565b6102f7565b6040516101799190610be9565b60405180910390f35b61018a61033f565b6040516101979190610aa9565b60405180910390f35b6101ba60048036038101906101b59190610b64565b6103d1565b6040516101c79190610bbf565b60405180910390f35b6101ea60048036038101906101e59190610cbb565b6103f4565b6040516101f79190610be9565b60405180910390f35b60606003805461020f90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461023b90610d2a565b80156102885780601f1061025d57610100808354040283529160200191610288565b820191906000526020600020905b81548152906001019060200180831161026b57829003601f168201915b5050505050905090565b60008061029d61047b565b90506102aa818585610483565b600191505092915050565b6000600254905090565b6000806102ca61047b565b90506102d7858285610495565b6102e2858585610529565b60019150509392505050565b60006012905090565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60606004805461034e90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461037a90610d2a565b80156103c75780601f1061039c576101008083540402835291602001916103c7565b820191906000526020600020905b8154815290600101906020018083116103aa57829003601f168201915b5050505050905090565b6000806103dc61047b565b90506103e9818585610529565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b610490838383600161061d565b505050565b60006104a184846103f4565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146105235781811015610513578281836040517ffb8f41b200000000000000000000000000000000000000000000000000000000815260040161050a93929190610d6a565b60405180910390fd5b6105228484848403600061061d565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361059b5760006040517f96c6fd1e0000000000000000000000000000000000000000000000000000000081526004016105929190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361060d5760006040517fec442f050000000000000000000000000000000000000000000000000000000081526004016106049190610da1565b60405180910390fd5b6106188383836107f4565b505050565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff160361068f5760006040517fe602df050000000000000000000000000000000000000000000000000000000081526004016106869190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036107015760006040517f94280d620000000000000000000000000000000000000000000000000000000081526004016106f89190610da1565b60405180910390fd5b81600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000208190555080156107ee578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516107e59190610be9565b60405180910390a35b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361084657806002600082825461083a9190610deb565b92505081905550610919565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050818110156108d2578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016108c993929190610d6a565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361096257806002600082825403925050819055506109af565b806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051610a0c9190610be9565b60405180910390a3505050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610a53578082015181840152602081019050610a38565b60008484015250505050565b6000601f19601f8301169050919050565b6000610a7b82610a19565b610a858185610a24565b9350610a95818560208601610a35565b610a9e81610a5f565b840191505092915050565b60006020820190508181036000830152610ac38184610a70565b905092915050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610afb82610ad0565b9050919050565b610b0b81610af0565b8114610b1657600080fd5b50565b600081359050610b2881610b02565b92915050565b6000819050919050565b610b4181610b2e565b8114610b4c57600080fd5b50565b600081359050610b5e81610b38565b92915050565b60008060408385031215610b7b57610b7a610acb565b5b6000610b8985828601610b19565b9250506020610b9a85828601610b4f565b9150509250929050565b60008115159050919050565b610bb981610ba4565b82525050565b6000602082019050610bd46000830184610bb0565b92915050565b610be381610b2e565b82525050565b6000602082019050610bfe6000830184610bda565b92915050565b600080600060608486031215610c1d57610c1c610acb565b5b6000610c2b86828701610b19565b9350506020610c3c86828701610b19565b9250506040610c4d86828701610b4f565b9150509250925092565b600060ff82169050919050565b610c6d81610c57565b82525050565b6000602082019050610c886000830184610c64565b92915050565b600060208284031215610ca457610ca3610acb565b5b6000610cb284828501610b19565b91505092915050565b60008060408385031215610cd257610cd1610acb565b5b6000610ce085828601610b19565b9250506020610cf185828601610b19565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680610d4257607f821691505b602082108103610d5557610d54610cfb565b5b50919050565b610d6481610af0565b82525050565b6000606082019050610d7f6000830186610d5b565b610d8c6020830185610bda565b610d996040830184610bda565b949350505050565b6000602082019050610db66000830184610d5b565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610df682610b2e565b9150610e0183610b2e565b9250828201905080821115610e1957610e18610dbc565b5b9291505056fea26469706673582212203f94478400ca0031ba543400a90ffc7349bed715f562669d8edd4af9ff89dcd664736f6c63430008140033").into()).gas_limit(0x989680)
        .exec();
    assert!(result.is_success());
}

#[test]
fn test_bridge_contract_with_call() {
    // {
    //     "cd596583": "bridgeContract()",
    //     "aab858dd": "computePeggedTokenAddress(address)",
    //     "8da5cb5b": "owner()",
    //     "715018a6": "renounceOwnership()",
    //     "e77772fe": "tokenFactory()",
    //     "f2fde38b": "transferOwnership(address)"
    // }

    let mut ctx = EvmTestingContext::default();
    let signer_l1_wallet_owner = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

    let pegged_token_contract_address = address!("5FbDB2315678afecb367f032d93F642f64180aa3");
    let erc20token_contract_address = address!("e7f1725e7734ce288f8367e1bb143e90bb3f0512");
    let erc20gateway_contract_address = address!("9fe46736679d2d9a65f0992f2272de9f3c7fa6e0");
    let l1token_contract_address = address!("Dc64a140Aa3E981100a9becA4E685f962f0cF6C9");

    let _random_address_address = address!("8947394629469832692836491629461498137497");

    println!("\n\npegged_token_contract:");
    let mut pegged_token_factory_tx_builder = TxBuilder::create(
        &mut ctx,
        signer_l1_wallet_owner,
        hex!("60806040523480156200001157600080fd5b5060405180602001604052806000815250604051806020016040528060008152508160039081620000439190620002d8565b508060049081620000559190620002d8565b505050620003bf565b600081519050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680620000e057607f821691505b602082108103620000f657620000f562000098565b5b50919050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b600060088302620001607fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000121565b6200016c868362000121565b95508019841693508086168417925050509392505050565b6000819050919050565b6000819050919050565b6000620001b9620001b3620001ad8462000184565b6200018e565b62000184565b9050919050565b6000819050919050565b620001d58362000198565b620001ed620001e482620001c0565b8484546200012e565b825550505050565b600090565b62000204620001f5565b62000211818484620001ca565b505050565b5b8181101562000239576200022d600082620001fa565b60018101905062000217565b5050565b601f82111562000288576200025281620000fc565b6200025d8462000111565b810160208510156200026d578190505b620002856200027c8562000111565b83018262000216565b50505b505050565b600082821c905092915050565b6000620002ad600019846008026200028d565b1980831691505092915050565b6000620002c883836200029a565b9150826002028217905092915050565b620002e3826200005e565b67ffffffffffffffff811115620002ff57620002fe62000069565b5b6200030b8254620000c7565b620003188282856200023d565b600060209050601f8311600181146200035057600084156200033b578287015190505b620003478582620002ba565b865550620003b7565b601f1984166200036086620000fc565b60005b828110156200038a5784890151825560018201915060208501945060208101905062000363565b86831015620003aa5784890151620003a6601f8916826200029a565b8355505b6001600288020188555050505b505050505050565b610e5580620003cf6000396000f3fe608060405234801561001057600080fd5b50600436106100935760003560e01c8063313ce56711610066578063313ce5671461013457806370a082311461015257806395d89b4114610182578063a9059cbb146101a0578063dd62ed3e146101d057610093565b806306fdde0314610098578063095ea7b3146100b657806318160ddd146100e657806323b872dd14610104575b600080fd5b6100a0610200565b6040516100ad9190610aa9565b60405180910390f35b6100d060048036038101906100cb9190610b64565b610292565b6040516100dd9190610bbf565b60405180910390f35b6100ee6102b5565b6040516100fb9190610be9565b60405180910390f35b61011e60048036038101906101199190610c04565b6102bf565b60405161012b9190610bbf565b60405180910390f35b61013c6102ee565b6040516101499190610c73565b60405180910390f35b61016c60048036038101906101679190610c8e565b6102f7565b6040516101799190610be9565b60405180910390f35b61018a61033f565b6040516101979190610aa9565b60405180910390f35b6101ba60048036038101906101b59190610b64565b6103d1565b6040516101c79190610bbf565b60405180910390f35b6101ea60048036038101906101e59190610cbb565b6103f4565b6040516101f79190610be9565b60405180910390f35b60606003805461020f90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461023b90610d2a565b80156102885780601f1061025d57610100808354040283529160200191610288565b820191906000526020600020905b81548152906001019060200180831161026b57829003601f168201915b5050505050905090565b60008061029d61047b565b90506102aa818585610483565b600191505092915050565b6000600254905090565b6000806102ca61047b565b90506102d7858285610495565b6102e2858585610529565b60019150509392505050565b60006012905090565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60606004805461034e90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461037a90610d2a565b80156103c75780601f1061039c576101008083540402835291602001916103c7565b820191906000526020600020905b8154815290600101906020018083116103aa57829003601f168201915b5050505050905090565b6000806103dc61047b565b90506103e9818585610529565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b610490838383600161061d565b505050565b60006104a184846103f4565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146105235781811015610513578281836040517ffb8f41b200000000000000000000000000000000000000000000000000000000815260040161050a93929190610d6a565b60405180910390fd5b6105228484848403600061061d565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361059b5760006040517f96c6fd1e0000000000000000000000000000000000000000000000000000000081526004016105929190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361060d5760006040517fec442f050000000000000000000000000000000000000000000000000000000081526004016106049190610da1565b60405180910390fd5b6106188383836107f4565b505050565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff160361068f5760006040517fe602df050000000000000000000000000000000000000000000000000000000081526004016106869190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036107015760006040517f94280d620000000000000000000000000000000000000000000000000000000081526004016106f89190610da1565b60405180910390fd5b81600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000208190555080156107ee578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516107e59190610be9565b60405180910390a35b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361084657806002600082825461083a9190610deb565b92505081905550610919565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050818110156108d2578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016108c993929190610d6a565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361096257806002600082825403925050819055506109af565b806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051610a0c9190610be9565b60405180910390a3505050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610a53578082015181840152602081019050610a38565b60008484015250505050565b6000601f19601f8301169050919050565b6000610a7b82610a19565b610a858185610a24565b9350610a95818560208601610a35565b610a9e81610a5f565b840191505092915050565b60006020820190508181036000830152610ac38184610a70565b905092915050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610afb82610ad0565b9050919050565b610b0b81610af0565b8114610b1657600080fd5b50565b600081359050610b2881610b02565b92915050565b6000819050919050565b610b4181610b2e565b8114610b4c57600080fd5b50565b600081359050610b5e81610b38565b92915050565b60008060408385031215610b7b57610b7a610acb565b5b6000610b8985828601610b19565b9250506020610b9a85828601610b4f565b9150509250929050565b60008115159050919050565b610bb981610ba4565b82525050565b6000602082019050610bd46000830184610bb0565b92915050565b610be381610b2e565b82525050565b6000602082019050610bfe6000830184610bda565b92915050565b600080600060608486031215610c1d57610c1c610acb565b5b6000610c2b86828701610b19565b9350506020610c3c86828701610b19565b9250506040610c4d86828701610b4f565b9150509250925092565b600060ff82169050919050565b610c6d81610c57565b82525050565b6000602082019050610c886000830184610c64565b92915050565b600060208284031215610ca457610ca3610acb565b5b6000610cb284828501610b19565b91505092915050565b60008060408385031215610cd257610cd1610acb565b5b6000610ce085828601610b19565b9250506020610cf185828601610b19565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680610d4257607f821691505b602082108103610d5557610d54610cfb565b5b50919050565b610d6481610af0565b82525050565b6000606082019050610d7f6000830186610d5b565b610d8c6020830185610bda565b610d996040830184610bda565b949350505050565b6000602082019050610db66000830184610d5b565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610df682610b2e565b9150610e0183610b2e565b9250828201905080821115610e1957610e18610dbc565b5b9291505056fea26469706673582212203f94478400ca0031ba543400a90ffc7349bed715f562669d8edd4af9ff89dcd664736f6c63430008140033").into());
    assert_eq!(
        signer_l1_wallet_owner,
        pegged_token_factory_tx_builder.env.tx.caller,
    );
    let result = pegged_token_factory_tx_builder.exec();
    match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Create(_, address) => {
                assert_eq!(pegged_token_contract_address, address.unwrap());
            }
            _ => panic!("expected 'create'"),
        },
        _ => panic!("expected 'success'"),
    }

    println!("\n\nerc20token_contract:");
    let mut erc20token_factory_tx_builder = TxBuilder::create(
        &mut ctx,
        signer_l1_wallet_owner,
        hex!("60806040523480156200001157600080fd5b5060405162000bc738038062000bc78339818101604052810190620000379190620002a7565b33600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1603620000ad5760006040517f1e4fbdf7000000000000000000000000000000000000000000000000000000008152600401620000a49190620002ea565b60405180910390fd5b620000be816200017960201b60201c565b50600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff160362000131576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401620001289062000368565b60405180910390fd5b80600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550506200038a565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b60006200026f8262000242565b9050919050565b620002818162000262565b81146200028d57600080fd5b50565b600081519050620002a18162000276565b92915050565b600060208284031215620002c057620002bf6200023d565b5b6000620002d08482850162000290565b91505092915050565b620002e48162000262565b82525050565b6000602082019050620003016000830184620002d9565b92915050565b600082825260208201905092915050565b7f7a65726f20696d706c656d656e746174696f6e20616464726573730000000000600082015250565b600062000350601b8362000307565b91506200035d8262000318565b602082019050919050565b60006020820190508181036000830152620003838162000341565b9050919050565b61082d806200039a6000396000f3fe608060405234801561001057600080fd5b506004361061007d5760003560e01c80638da5cb5b1161005b5780638da5cb5b146100da578063cd6f2760146100f8578063f2228ebc14610128578063f2fde38b146101585761007d565b80635c60da1b14610082578063715018a6146100a05780637ef2afdd146100aa575b600080fd5b61008a610174565b6040516100979190610663565b60405180910390f35b6100a861019a565b005b6100c460048036038101906100bf91906106af565b6101ae565b6040516100d19190610663565b60405180910390f35b6100e26101d3565b6040516100ef9190610663565b60405180910390f35b610112600480360381019061010d9190610716565b6101fc565b60405161011f9190610663565b60405180910390f35b610142600480360381019061013d9190610716565b610240565b60405161014f9190610663565b60405180910390f35b610172600480360381019061016d9190610756565b6102ed565b005b600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b6101a2610373565b6101ac60006103fa565b565b6000806101bb86866104be565b90506101c88482856104f1565b915050949350505050565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b60008061020984846104be565b9050610237600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1682610552565b91505092915050565b600061024a610373565b600061025684846104be565b90506000610286600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1683610567565b90508073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167ff9a44e6db3fb6e0eb31c4013bda8c662fecef1768dd2412270cc8f8821cbccf360405160405180910390a3809250505092915050565b6102f5610373565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16036103675760006040517f1e4fbdf700000000000000000000000000000000000000000000000000000000815260040161035e9190610663565b60405180910390fd5b610370816103fa565b50565b61037b61061a565b73ffffffffffffffffffffffffffffffffffffffff166103996101d3565b73ffffffffffffffffffffffffffffffffffffffff16146103f8576103bc61061a565b6040517f118cdaa70000000000000000000000000000000000000000000000000000000081526004016103ef9190610663565b60405180910390fd5b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600082826040516020016104d39291906107cb565b60405160208183030381529060405280519060200120905092915050565b60006040518260388201526f5af43d82803e903d91602b57fd5bf3ff6024820152846014820152733d602d80600a3d3981f3363d3d373d3d3d363d7381528360588201526037600c8201206078820152605560438201209150509392505050565b600061055f8383306104f1565b905092915050565b6000763d602d80600a3d3981f3363d3d373d3d3d363d730000008360601b60e81c176000526e5af43d82803e903d91602b57fd5bf38360781b1760205281603760096000f59050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1603610614576040517fc2f868f400000000000000000000000000000000000000000000000000000000815260040160405180910390fd5b92915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061064d82610622565b9050919050565b61065d81610642565b82525050565b60006020820190506106786000830184610654565b92915050565b600080fd5b61068c81610642565b811461069757600080fd5b50565b6000813590506106a981610683565b92915050565b600080600080608085870312156106c9576106c861067e565b5b60006106d78782880161069a565b94505060206106e88782880161069a565b93505060406106f98782880161069a565b925050606061070a8782880161069a565b91505092959194509250565b6000806040838503121561072d5761072c61067e565b5b600061073b8582860161069a565b925050602061074c8582860161069a565b9150509250929050565b60006020828403121561076c5761076b61067e565b5b600061077a8482850161069a565b91505092915050565b60008160601b9050919050565b600061079b82610783565b9050919050565b60006107ad82610790565b9050919050565b6107c56107c082610642565b6107a2565b82525050565b60006107d782856107b4565b6014820191506107e782846107b4565b601482019150819050939250505056fea2646970667358221220ef5b73c53ef571e3032bcd524e89f7cb228f279419486d398ccd722a0eff091064736f6c634300081400330000000000000000000000005fbdb2315678afecb367f032d93f642f64180aa3").into());
    assert_eq!(
        signer_l1_wallet_owner,
        erc20token_factory_tx_builder.env.tx.caller,
    );
    let result = erc20token_factory_tx_builder.exec();
    match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Create(_, address) => {
                assert_eq!(erc20token_contract_address, address.unwrap());
            }
            _ => panic!("expected 'create'"),
        },
        _ => panic!("expected 'success'"),
    }

    println!("\n\nerc20gateway_contract:");
    let mut erc20gateway_factory_tx_builder = TxBuilder::create(
        &mut ctx,
        signer_l1_wallet_owner,
        hex!("608060405260405161084c38038061084c83398181016040528101906100259190610258565b33600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16036100985760006040517f1e4fbdf700000000000000000000000000000000000000000000000000000000815260040161008f91906102a7565b60405180910390fd5b6100a78161013160201b60201c565b5081600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555080600260006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050506102c2565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610225826101fa565b9050919050565b6102358161021a565b811461024057600080fd5b50565b6000815190506102528161022c565b92915050565b6000806040838503121561026f5761026e6101f5565b5b600061027d85828601610243565b925050602061028e85828601610243565b9150509250929050565b6102a18161021a565b82525050565b60006020820190506102bc6000830184610298565b92915050565b61057b806102d16000396000f3fe608060405234801561001057600080fd5b50600436106100625760003560e01c8063715018a6146100675780638da5cb5b14610071578063aab858dd1461008f578063cd596583146100bf578063e77772fe146100dd578063f2fde38b146100fb575b600080fd5b61006f610117565b005b61007961012b565b6040516100869190610461565b60405180910390f35b6100a960048036038101906100a491906104ad565b610154565b6040516100b69190610461565b60405180910390f35b6100c76101fb565b6040516100d49190610461565b60405180910390f35b6100e5610221565b6040516100f29190610461565b60405180910390f35b610115600480360381019061011091906104ad565b610247565b005b61011f6102cd565b6101296000610354565b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b6000600260009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1663cd6f276030846040518363ffffffff1660e01b81526004016101b39291906104da565b602060405180830381865afa1580156101d0573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906101f49190610518565b9050919050565b600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b600260009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b61024f6102cd565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16036102c15760006040517f1e4fbdf70000000000000000000000000000000000000000000000000000000081526004016102b89190610461565b60405180910390fd5b6102ca81610354565b50565b6102d5610418565b73ffffffffffffffffffffffffffffffffffffffff166102f361012b565b73ffffffffffffffffffffffffffffffffffffffff161461035257610316610418565b6040517f118cdaa70000000000000000000000000000000000000000000000000000000081526004016103499190610461565b60405180910390fd5b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061044b82610420565b9050919050565b61045b81610440565b82525050565b60006020820190506104766000830184610452565b92915050565b600080fd5b61048a81610440565b811461049557600080fd5b50565b6000813590506104a781610481565b92915050565b6000602082840312156104c3576104c261047c565b5b60006104d184828501610498565b91505092915050565b60006040820190506104ef6000830185610452565b6104fc6020830184610452565b9392505050565b60008151905061051281610481565b92915050565b60006020828403121561052e5761052d61047c565b5b600061053c84828501610503565b9150509291505056fea2646970667358221220519e506cbbf37fc9099e29fbd5f09d982167f702dc6160b48912d5cfa1f294ab64736f6c634300081400330000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e7f1725e7734ce288f8367e1bb143e90bb3f0512").into())
        .value(U256::from(1000e9));
    assert_eq!(
        signer_l1_wallet_owner,
        erc20gateway_factory_tx_builder.env.tx.caller,
    );
    let result = erc20gateway_factory_tx_builder.exec();
    match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Create(_, address) => {
                assert_eq!(erc20gateway_contract_address, address.unwrap());
            }
            _ => panic!("expected 'create'"),
        },
        _ => panic!("expected 'success'"),
    }

    println!("\n\ntransferOwnership call:");
    let mut transfer_ownership_tx_builder = TxBuilder::call(
        &mut ctx,
        signer_l1_wallet_owner,
        erc20gateway_contract_address,
    )
    .input(bytes!(
        "\
        f2fde38b\
        0000000000000000000000009fe46736679d2d9a65f0992f2272de9f3c7fa6e0\
        "
    ));
    assert_eq!(
        signer_l1_wallet_owner,
        transfer_ownership_tx_builder.env.tx.caller,
    );
    let result = transfer_ownership_tx_builder.exec();
    match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(bytes) => {
                assert_eq!(Bytes::new(), bytes);
            }
            _ => panic!("expected 'call'"),
        },
        _ => panic!("expected 'success'"),
    }

    println!("\n\nl1token_contract:");
    let mut l1token_factory_tx_builder = TxBuilder::create(
        &mut ctx,
        signer_l1_wallet_owner,
        hex!("60806040523480156200001157600080fd5b50604051620018ab380380620018ab83398181016040528101906200003791906200056b565b838381600390816200004a91906200085c565b5080600490816200005c91906200085c565b5050506200007181836200007b60201b60201c565b5050505062000a46565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603620000f05760006040517fec442f05000000000000000000000000000000000000000000000000000000008152600401620000e7919062000954565b60405180910390fd5b62000104600083836200010860201b60201c565b5050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036200015e578060026000828254620001519190620009a0565b9250508190555062000234565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905081811015620001ed578381836040517fe450d38c000000000000000000000000000000000000000000000000000000008152600401620001e493929190620009ec565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036200027f5780600260008282540392505081905550620002cc565b806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516200032b919062000a29565b60405180910390a3505050565b6000604051905090565b600080fd5b600080fd5b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b620003a18262000356565b810181811067ffffffffffffffff82111715620003c357620003c262000367565b5b80604052505050565b6000620003d862000338565b9050620003e6828262000396565b919050565b600067ffffffffffffffff82111562000409576200040862000367565b5b620004148262000356565b9050602081019050919050565b60005b838110156200044157808201518184015260208101905062000424565b60008484015250505050565b6000620004646200045e84620003eb565b620003cc565b90508281526020810184848401111562000483576200048262000351565b5b6200049084828562000421565b509392505050565b600082601f830112620004b057620004af6200034c565b5b8151620004c28482602086016200044d565b91505092915050565b6000819050919050565b620004e081620004cb565b8114620004ec57600080fd5b50565b6000815190506200050081620004d5565b92915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000620005338262000506565b9050919050565b620005458162000526565b81146200055157600080fd5b50565b60008151905062000565816200053a565b92915050565b6000806000806080858703121562000588576200058762000342565b5b600085015167ffffffffffffffff811115620005a957620005a862000347565b5b620005b78782880162000498565b945050602085015167ffffffffffffffff811115620005db57620005da62000347565b5b620005e98782880162000498565b9350506040620005fc87828801620004ef565b92505060606200060f8782880162000554565b91505092959194509250565b600081519050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b600060028204905060018216806200066e57607f821691505b60208210810362000684576200068362000626565b5b50919050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b600060088302620006ee7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82620006af565b620006fa8683620006af565b95508019841693508086168417925050509392505050565b6000819050919050565b60006200073d620007376200073184620004cb565b62000712565b620004cb565b9050919050565b6000819050919050565b62000759836200071c565b62000771620007688262000744565b848454620006bc565b825550505050565b600090565b6200078862000779565b620007958184846200074e565b505050565b5b81811015620007bd57620007b16000826200077e565b6001810190506200079b565b5050565b601f8211156200080c57620007d6816200068a565b620007e1846200069f565b81016020851015620007f1578190505b6200080962000800856200069f565b8301826200079a565b50505b505050565b600082821c905092915050565b6000620008316000198460080262000811565b1980831691505092915050565b60006200084c83836200081e565b9150826002028217905092915050565b62000867826200061b565b67ffffffffffffffff81111562000883576200088262000367565b5b6200088f825462000655565b6200089c828285620007c1565b600060209050601f831160018114620008d45760008415620008bf578287015190505b620008cb85826200083e565b8655506200093b565b601f198416620008e4866200068a565b60005b828110156200090e57848901518255600182019150602085019450602081019050620008e7565b868310156200092e57848901516200092a601f8916826200081e565b8355505b6001600288020188555050505b505050505050565b6200094e8162000526565b82525050565b60006020820190506200096b600083018462000943565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000620009ad82620004cb565b9150620009ba83620004cb565b9250828201905080821115620009d557620009d462000971565b5b92915050565b620009e681620004cb565b82525050565b600060608201905062000a03600083018662000943565b62000a126020830185620009db565b62000a216040830184620009db565b949350505050565b600060208201905062000a406000830184620009db565b92915050565b610e558062000a566000396000f3fe608060405234801561001057600080fd5b50600436106100935760003560e01c8063313ce56711610066578063313ce5671461013457806370a082311461015257806395d89b4114610182578063a9059cbb146101a0578063dd62ed3e146101d057610093565b806306fdde0314610098578063095ea7b3146100b657806318160ddd146100e657806323b872dd14610104575b600080fd5b6100a0610200565b6040516100ad9190610aa9565b60405180910390f35b6100d060048036038101906100cb9190610b64565b610292565b6040516100dd9190610bbf565b60405180910390f35b6100ee6102b5565b6040516100fb9190610be9565b60405180910390f35b61011e60048036038101906101199190610c04565b6102bf565b60405161012b9190610bbf565b60405180910390f35b61013c6102ee565b6040516101499190610c73565b60405180910390f35b61016c60048036038101906101679190610c8e565b6102f7565b6040516101799190610be9565b60405180910390f35b61018a61033f565b6040516101979190610aa9565b60405180910390f35b6101ba60048036038101906101b59190610b64565b6103d1565b6040516101c79190610bbf565b60405180910390f35b6101ea60048036038101906101e59190610cbb565b6103f4565b6040516101f79190610be9565b60405180910390f35b60606003805461020f90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461023b90610d2a565b80156102885780601f1061025d57610100808354040283529160200191610288565b820191906000526020600020905b81548152906001019060200180831161026b57829003601f168201915b5050505050905090565b60008061029d61047b565b90506102aa818585610483565b600191505092915050565b6000600254905090565b6000806102ca61047b565b90506102d7858285610495565b6102e2858585610529565b60019150509392505050565b60006012905090565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60606004805461034e90610d2a565b80601f016020809104026020016040519081016040528092919081815260200182805461037a90610d2a565b80156103c75780601f1061039c576101008083540402835291602001916103c7565b820191906000526020600020905b8154815290600101906020018083116103aa57829003601f168201915b5050505050905090565b6000806103dc61047b565b90506103e9818585610529565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b610490838383600161061d565b505050565b60006104a184846103f4565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146105235781811015610513578281836040517ffb8f41b200000000000000000000000000000000000000000000000000000000815260040161050a93929190610d6a565b60405180910390fd5b6105228484848403600061061d565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361059b5760006040517f96c6fd1e0000000000000000000000000000000000000000000000000000000081526004016105929190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361060d5760006040517fec442f050000000000000000000000000000000000000000000000000000000081526004016106049190610da1565b60405180910390fd5b6106188383836107f4565b505050565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff160361068f5760006040517fe602df050000000000000000000000000000000000000000000000000000000081526004016106869190610da1565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036107015760006040517f94280d620000000000000000000000000000000000000000000000000000000081526004016106f89190610da1565b60405180910390fd5b81600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000208190555080156107ee578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516107e59190610be9565b60405180910390a35b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361084657806002600082825461083a9190610deb565b92505081905550610919565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050818110156108d2578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016108c993929190610d6a565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361096257806002600082825403925050819055506109af565b806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051610a0c9190610be9565b60405180910390a3505050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610a53578082015181840152602081019050610a38565b60008484015250505050565b6000601f19601f8301169050919050565b6000610a7b82610a19565b610a858185610a24565b9350610a95818560208601610a35565b610a9e81610a5f565b840191505092915050565b60006020820190508181036000830152610ac38184610a70565b905092915050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610afb82610ad0565b9050919050565b610b0b81610af0565b8114610b1657600080fd5b50565b600081359050610b2881610b02565b92915050565b6000819050919050565b610b4181610b2e565b8114610b4c57600080fd5b50565b600081359050610b5e81610b38565b92915050565b60008060408385031215610b7b57610b7a610acb565b5b6000610b8985828601610b19565b9250506020610b9a85828601610b4f565b9150509250929050565b60008115159050919050565b610bb981610ba4565b82525050565b6000602082019050610bd46000830184610bb0565b92915050565b610be381610b2e565b82525050565b6000602082019050610bfe6000830184610bda565b92915050565b600080600060608486031215610c1d57610c1c610acb565b5b6000610c2b86828701610b19565b9350506020610c3c86828701610b19565b9250506040610c4d86828701610b4f565b9150509250925092565b600060ff82169050919050565b610c6d81610c57565b82525050565b6000602082019050610c886000830184610c64565b92915050565b600060208284031215610ca457610ca3610acb565b5b6000610cb284828501610b19565b91505092915050565b60008060408385031215610cd257610cd1610acb565b5b6000610ce085828601610b19565b9250506020610cf185828601610b19565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680610d4257607f821691505b602082108103610d5557610d54610cfb565b5b50919050565b610d6481610af0565b82525050565b6000606082019050610d7f6000830186610d5b565b610d8c6020830185610bda565b610d996040830184610bda565b949350505050565b6000602082019050610db66000830184610d5b565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000610df682610b2e565b9150610e0183610b2e565b9250828201905080821115610e1957610e18610dbc565b5b9291505056fea26469706673582212201ce3fc06792366bb245778c84a99112faf16f0d2b31c562700d94a4821b2c94364736f6c63430008140033").into())
        .value(U256::from(1000000e9))
        .input(bytes!("\
        0000000000000000000000000000000000000000000000000000000000000080\
        00000000000000000000000000000000000000000000000000000000000000c0\
        00000000000000000000000000000000000000000000d3c21bcecceda1000000\
        000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266\
        000000000000000000000000000000000000000000000000000000000000000a\
        4d6f636b20546f6b656e00000000000000000000000000000000000000000000\
        0000000000000000000000000000000000000000000000000000000000000003\
        544b4e0000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(
        signer_l1_wallet_owner,
        l1token_factory_tx_builder.env.tx.caller,
    );
    let result = l1token_factory_tx_builder.exec();
    match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Create(_, address) => {
                assert_eq!(l1token_contract_address, address.unwrap());
            }
            _ => panic!("expected 'create'"),
        },
        _ => panic!("expected 'success'"),
    }

    println!("\n\ncomputePeggedTokenAddress call:");

    // println!("address: {}", hex::encode(&l1token_contract_address));

    assert!(ctx.db.accounts.contains_key(&erc20gateway_contract_address));
    let erc20gateway_contract_db_account =
        ctx.db.accounts.get(&erc20gateway_contract_address).unwrap();
    let erc20gateway_contract_db_account_info = erc20gateway_contract_db_account.info.clone();
    assert!(erc20gateway_contract_db_account_info.code.is_some());
    assert!(!erc20gateway_contract_db_account_info.code_hash.is_zero());
    // assert!(erc20gateway_contract_db_account_info.code.unwrap().len() > 0);
    assert!(erc20gateway_contract_db_account_info.rwasm_code.is_some());
    assert!(!erc20gateway_contract_db_account_info
        .rwasm_code_hash
        .is_zero());
    let mut erc20gateway_factory_tx_builder = TxBuilder::call(
        &mut ctx,
        signer_l1_wallet_owner,
        erc20gateway_contract_address,
    )
    // data: 0x70616e69636b6564206174206372617465732f636f72652f7372632f636f6e7472616374732f65636c2e72733a34373a31373a2063616c6c206d6574686f64206661696c65642c206578697420636f64653a202d31303232
    .input(bytes!(
        "\
        aab858dd\
        000000000000000000000000dc64a140aa3e981100a9beca4e685f962f0cf6c9\
        "
    ));

    // 70616e69636b6564206174206372617465732f636f72652f7372632f636f6e74
    // 72616374732f65636c2e72733a34373a31373a2063616c6c206d6574686f6420
    // 6661696c65642c206578697420636f64653a202d31303232

    assert_eq!(
        signer_l1_wallet_owner,
        erc20gateway_factory_tx_builder.env.tx.caller,
    );
    let result = erc20gateway_factory_tx_builder.exec();
    assert!(!result.output().unwrap().is_empty());
    assert!(result.is_success());
}

// #[test]
// fn test_codec_case() {
//     let call_method_input = EvmCallMethodInput {
//         caller: Default::default(),
//         address: address!("095e7baea6a6c7c4c2dfeb977efac326af552d87"),
//         bytecode_address: address!("095e7baea6a6c7c4c2dfeb977efac326af552d87"),
//         value: U256::from_be_slice(
//             &hex::decode("0x00000000000000000000000000000000000000000000000000000000000186a0")
//                 .unwrap(),
//         ),
//         apparent_value: Default::default(),
//         input: Bytes::copy_from_slice(&hex::decode("").unwrap()),
//         gas_limit: 9999979000,
//         depth: 0,
//         is_static: false,
//     };
//     let call_method_input_encoded = call_method_input.encode_to_vec(0);
//     let mut buffer = BufferDecoder::new(&call_method_input_encoded);
//     let mut call_method_input_decoded = EvmCallMethodInput::default();
//     EvmCallMethodInput::decode_body(&mut buffer, 0, &mut call_method_input_decoded);
//     assert_eq!(call_method_input_decoded.callee, call_method_input.callee);
// }

#[test]
fn test_simple_nested_call() {
    let mut ctx = EvmTestingContext::default();
    let account1 = ctx.add_wasm_contract(
        address!("0000000000000000000000000000000000000001"),
        instruction_set! {
            I32Const(100)
            I32Const(20)
            I32Add
            I32Const(3)
            I32Add
            Call(SysFuncIdx::EXIT)
        },
    );
    let mut memory_section = vec![0u8; 32 + 8];
    memory_section[0..32].copy_from_slice(&account1.rwasm_code_hash.0);
    let code_section = instruction_set! {
        // alloc and init memory
        I32Const(1)
        MemoryGrow
        Drop
        I32Const(0)
        I32Const(0)
        I32Const(40)
        MemoryInit(0)
        DataDrop(0)
        // sys exec hash
        I32Const(0) // bytecode_hash32_offset
        I32Const(0) // input_offset
        I32Const(0) // input_len
        I32Const(0) // return_offset
        I32Const(0) // return_len
        I32Const(32) // fuel_offset
        I32Const(0) // state
        Call(SysFuncIdx::EXEC)
        Drop
        // check error
        I32Const(ExitCode::Ok.into_i32())
        Call(SysFuncIdx::EXIT)
    };
    let code_section_len = code_section.len() as u32;
    ctx.add_wasm_contract(
        address!("0000000000000000000000000000000000000002"),
        RwasmModule {
            code_section,
            memory_section,
            func_section: vec![code_section_len],
            ..Default::default()
        },
    );
    let result = TxBuilder::call(
        &mut ctx,
        Address::ZERO,
        address!("0000000000000000000000000000000000000002"),
    )
    .gas_price(U256::ZERO)
    .exec();
    println!("{:?}", result);
    assert!(result.is_success());
}
