use crate::util::{check_success, wat2wasm, TestingContext};
use revm::primitives::{address, Eval, Output};

#[test]
fn test_greeting() {
    let mut ctx = TestingContext::default();
    let res = check_success(ctx.deploy_contract(
        address!("0000000000000000000000000000000000000000"),
        &wat2wasm(include_str!("../bin/greeting-deploy.wat")),
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
