use crate::util::{check_success, wat2wasm, TestingContext};
use hex_literal::hex;
use revm::primitives::{address, Eval, Output};

#[test]
fn test_greeting() {
    let mut ctx = TestingContext::default();
    let res = check_success(ctx.deploy_contract(
        address!("0000000000000000000000000000000000000000"),
        include_bytes!("../bin/greeting-deploy.wasm"),
    ));
    assert_eq!(res.reason, Eval::Return);
    let address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_contract(
        address!("0000000000000000000000000000000000000000"),
        address,
        &[],
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!(output, "Hello, World".as_bytes().to_vec());
}

#[test]
fn test_evm() {
    let hello_world_bytecode = hex!("608060405234801561000f575f80fd5b506101688061001d5f395ff3fe608060405234801561000f575f80fd5b5060043610610029575f3560e01c8063dffeadd01461002d575b5f80fd5b61003561004b565b6040516100429190610112565b60405180910390f35b60606040518060400160405280600c81526020017f48656c6c6f2c20576f726c640000000000000000000000000000000000000000815250905090565b5f81519050919050565b5f82825260208201905092915050565b5f5b838110156100bf5780820151818401526020810190506100a4565b5f8484015250505050565b5f601f19601f8301169050919050565b5f6100e482610088565b6100ee8185610092565b93506100fe8185602086016100a2565b610107816100ca565b840191505092915050565b5f6020820190508181035f83015261012a81846100da565b90509291505056fea2646970667358221220e37f1ddf5cf89f81a254d4ff46c19e6000be7de71326bd8a5106eeca92e3be6164736f6c63430008160033");
    let mut ctx = TestingContext::default();
    let res = check_success(ctx.deploy_contract(
        address!("0000000000000000000000000000000000000000"),
        &hello_world_bytecode,
    ));
    assert_eq!(res.reason, Eval::Return);
    let address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_contract(
        address!("0000000000000000000000000000000000000000"),
        address,
        &hex!("dffeadd0"),
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!(&output[64..76], "Hello, World".as_bytes());
}
