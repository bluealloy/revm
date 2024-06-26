use criterion::{criterion_group, criterion_main, Criterion};
use revm::{
    db::BenchmarkDB,
    interpreter::{analysis::to_analysed, Contract, DummyHost, Interpreter},
    primitives::{address, hex, Bytecode, Bytes, ShanghaiSpec, TxKind},
    Evm,
};
use revm_interpreter::{opcode::make_instruction_table, SharedMemory, EMPTY_SHARED_MEMORY};
use std::io;

fn bytecode_bench(c: &mut Criterion) {
    let mut code_input = String::new();
    let stdin = io::stdin();
    match stdin.read_line(&mut code_input) {
        Ok(_n) => {}
        Err(_error) => {}
    }

    if code_input.trim().is_empty() {
        println!("Empty bytecode on stdin.");
        return;
    }

    let code_input = code_input.trim();
    let bytecode = to_analysed(Bytecode::new_raw(hex::decode(code_input).unwrap().into()));

    let evm = Evm::builder()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_tx_env(|tx| {
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
        })
        .build();

    let mut g = c.benchmark_group("bytecode");

    g.bench_function("bytecode-benchmark", |b| {
        let contract = Contract {
            input: Bytes::from(hex::decode("").unwrap()),
            bytecode: to_analysed(bytecode.clone()),
            ..Default::default()
        };
        let mut shared_memory = SharedMemory::new();
        let mut host = DummyHost::new(*evm.context.evm.env.clone());
        let instruction_table = make_instruction_table::<DummyHost, ShanghaiSpec>();
        b.iter(move || {
            let temp = core::mem::replace(&mut shared_memory, EMPTY_SHARED_MEMORY);
            let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);
            let res = interpreter.run(temp, &instruction_table, &mut host);
            shared_memory = interpreter.take_memory();
            host.clear();
            res
        })
    });

    g.finish();
}

#[rustfmt::skip]
criterion_group!(benches, bytecode_bench);

criterion_main!(benches);
