//! Root hook coverage for transactions that halt before frame initialization.

use crate::{InspectEvm, Inspector};
use context::{
    result::ExecutionResult,
    transaction::{Authorization, RecoveredAuthority, RecoveredAuthorization},
    Cfg, Context, ContextTr, JournalTr, TxEnv,
};
use database::InMemoryDB;
use handler::{ExecuteEvm, FrameResult, MainBuilder, MainContext};
use interpreter::{
    CallInputs, CallOutcome, CreateInputs, CreateOutcome, FrameInput, Gas, InstructionResult,
    Interpreter, InterpreterResult,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use state::{AccountInfo, EvmState};

#[derive(Default)]
struct RootHooks {
    events: Vec<&'static str>,
    input: Option<FrameInput>,
    outcome: Option<FrameResult>,
    created_address: Option<Address>,
    spec_id: Option<SpecId>,
    initialized: usize,
    steps: usize,
    override_at: Option<&'static str>,
}

impl<CTX: ContextTr<Journal: JournalTr<State = EvmState>>> Inspector<CTX> for RootHooks {
    fn frame_start(&mut self, _: &mut CTX, input: &mut FrameInput) -> Option<FrameResult> {
        self.events.push("frame_start");
        self.input = Some(input.clone());
        (self.override_at == Some("frame_start")).then(|| revert_result(input))
    }

    fn call(&mut self, ctx: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.events.push("call");
        self.spec_id = Some(ctx.cfg().spec().into());
        if self.override_at == Some("call") {
            let FrameResult::Call(result) =
                revert_result(&FrameInput::Call(Box::new(inputs.clone())))
            else {
                unreachable!()
            };
            return Some(result);
        }
        None
    }

    fn call_end(&mut self, _: &mut CTX, _: &CallInputs, outcome: &mut CallOutcome) {
        self.events.push("call_end");
        if self.override_at == Some("call_end") {
            let FrameResult::Call(result) = revert_result(self.input.as_ref().unwrap()) else {
                unreachable!()
            };
            *outcome = result;
        }
    }

    fn create(&mut self, ctx: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.events.push("create");
        self.spec_id = Some(ctx.cfg().spec().into());
        let nonce = ctx
            .journal()
            .evm_state()
            .get(&inputs.caller())
            .unwrap()
            .info
            .nonce;
        self.created_address = Some(inputs.created_address(nonce));
        if self.override_at == Some("create") {
            let FrameResult::Create(result) =
                revert_result(&FrameInput::Create(Box::new(inputs.clone())))
            else {
                unreachable!()
            };
            return Some(result);
        }
        None
    }

    fn create_end(&mut self, _: &mut CTX, _: &CreateInputs, outcome: &mut CreateOutcome) {
        self.events.push("create_end");
        if self.override_at == Some("create_end") {
            let FrameResult::Create(result) = revert_result(self.input.as_ref().unwrap()) else {
                unreachable!()
            };
            *outcome = result;
        }
    }

    fn frame_end(&mut self, _: &mut CTX, _: &FrameInput, result: &mut FrameResult) {
        self.events.push("frame_end");
        if self.override_at == Some("frame_end") {
            *result = revert_result(self.input.as_ref().unwrap());
        }
        self.outcome = Some(result.clone());
    }

    fn initialize_interp(&mut self, _: &mut Interpreter, _: &mut CTX) {
        self.initialized += 1;
    }

    fn step(&mut self, _: &mut Interpreter, _: &mut CTX) {
        self.steps += 1;
    }
}

const CALLER: Address = address!("1000000000000000000000000000000000000001");
const RECIPIENT: Address = address!("1000000000000000000000000000000000000002");

fn funded_db() -> InMemoryDB {
    let mut db = InMemoryDB::default();
    db.insert_account_info(
        CALLER,
        AccountInfo {
            nonce: 7,
            ..AccountInfo::from_balance(U256::from(1_000_000))
        },
    );
    db
}

fn revert_result(input: &FrameInput) -> FrameResult {
    let (gas_limit, reservoir) = match input {
        FrameInput::Call(input) => (input.gas_limit, input.reservoir),
        FrameInput::Create(input) => (input.gas_limit(), input.reservoir()),
        FrameInput::Empty => unreachable!(),
    };
    let mut gas = Gas::new_with_regular_gas_and_reservoir(gas_limit, reservoir);
    assert!(gas.record_regular_cost(123));
    let result = InterpreterResult {
        result: InstructionResult::Revert,
        output: Bytes::from_static(&[0x42]),
        gas,
    };
    match input {
        FrameInput::Call(_) => FrameResult::Call(CallOutcome::new(result, 0..0)),
        FrameInput::Create(_) => FrameResult::Create(CreateOutcome::new(result, None)),
        FrameInput::Empty => unreachable!(),
    }
}

#[test]
fn runtime_oog_root_hooks() {
    for spec in [SpecId::OSAKA, SpecId::AMSTERDAM] {
        for kind in [TxKind::Call(RECIPIENT), TxKind::Create] {
            let data = if kind.is_create() {
                Bytes::from_static(&[0])
            } else {
                Bytes::from_static(&[1, 2, 3, 4])
            };
            let tx = TxEnv::builder()
                .caller(CALLER)
                .nonce(7)
                .kind(kind)
                .value(U256::ONE)
                .data(data.clone())
                .gas_limit(200_000)
                .gas_price(1)
                .build_fill();
            let ctx = Context::mainnet()
                .with_db(funded_db())
                .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(spec));
            let expected = ctx.clone().build_mainnet().transact(tx.clone()).unwrap();
            let mut evm = ctx.build_mainnet_with_inspector(RootHooks::default());
            let actual = evm.inspect_tx(tx.clone()).unwrap();
            assert_eq!(actual, expected);
            let inspector = &evm.inspector;
            let runtime_oog = spec == SpecId::AMSTERDAM;
            assert_eq!(actual.result.is_halt(), runtime_oog);
            assert_eq!(inspector.spec_id, Some(spec));
            if runtime_oog {
                assert_eq!(actual.result.tx_gas_used(), tx.gas_limit);
                assert_eq!(inspector.initialized, 0);
                assert_eq!(inspector.steps, 0);
                assert_eq!(
                    inspector.outcome.as_ref().unwrap().instruction_result(),
                    InstructionResult::OutOfGas
                );
            }
            match inspector.input.as_ref().unwrap() {
                FrameInput::Call(input) => {
                    assert_eq!(
                        inspector.events,
                        ["frame_start", "call", "call_end", "frame_end"]
                    );
                    assert_eq!(input.caller, CALLER);
                    assert_eq!(input.target_address, RECIPIENT);
                    assert_eq!(input.input, interpreter::CallInput::Bytes(data));
                    assert_eq!(input.call_value(), U256::ONE);
                }
                FrameInput::Create(input) => {
                    assert_eq!(
                        inspector.events,
                        ["frame_start", "create", "create_end", "frame_end"]
                    );
                    assert_eq!(input.caller(), CALLER);
                    assert_eq!(input.init_code(), &data);
                    assert_eq!(input.value(), U256::ONE);
                    assert_eq!(inspector.created_address, Some(CALLER.create(7)));
                }
                FrameInput::Empty => unreachable!(),
            }
        }
    }
}

#[test]
fn authorization_runtime_oog_root_hooks() {
    let authority = address!("1000000000000000000000000000000000000003");
    for funded in [false, true] {
        let mut db = funded_db();
        if funded {
            db.insert_account_info(authority, AccountInfo::from_balance(U256::ONE));
        }
        let authorization = RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::ONE,
                address: RECIPIENT,
                nonce: 0,
            },
            RecoveredAuthority::Valid(authority),
        );
        let tx = TxEnv::builder()
            .caller(CALLER)
            .nonce(7)
            .kind(TxKind::Call(RECIPIENT))
            .gas_limit(30_000)
            .data(Bytes::from_static(&[1, 2, 3, 4]))
            .authorization_list_recovered(vec![authorization])
            .build_fill();
        let ctx = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM));
        let mut plain = ctx.clone().build_mainnet();
        let mut inspected = ctx.build_mainnet_with_inspector(RootHooks::default());
        let expected = plain.transact_one(tx.clone()).unwrap();
        let actual = inspected.inspect_one_tx(tx).unwrap();
        assert_eq!(actual, expected);
        assert!(actual.is_halt());
        assert_eq!(actual.tx_gas_used(), 30_000);
        assert_eq!(
            inspected.ctx.journal().evm_state(),
            plain.ctx.journal().evm_state()
        );
        assert_eq!(
            inspected.inspector.events,
            ["frame_start", "call", "call_end", "frame_end"]
        );
        assert_eq!(inspected.inspector.initialized, 0);
        assert_eq!(inspected.inspector.steps, 0);
        assert_eq!(inspected.inspector.spec_id, Some(SpecId::AMSTERDAM));
    }
}

#[test]
fn runtime_oog_hook_overrides_are_settled() {
    for kind in [TxKind::Call(RECIPIENT), TxKind::Create] {
        let (start, end) = if kind.is_create() {
            ("create", "create_end")
        } else {
            ("call", "call_end")
        };
        for override_at in ["frame_start", start, end, "frame_end"] {
            let ctx = Context::mainnet()
                .with_db(funded_db())
                .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM));
            let inspector = RootHooks {
                override_at: Some(override_at),
                ..Default::default()
            };
            let mut evm = ctx.build_mainnet_with_inspector(inspector);
            let tx = TxEnv::builder()
                .caller(CALLER)
                .nonce(7)
                .kind(kind)
                .value(U256::ONE)
                .gas_limit(200_000)
                .build_fill();
            let result = evm.inspect_tx(tx.clone()).unwrap();
            assert!(
                matches!(result.result, ExecutionResult::Revert { .. }),
                "{override_at}: {result:?}"
            );
            assert_eq!(result.result.output(), Some(&Bytes::from_static(&[0x42])));
            let input = evm.inspector.input.as_ref().unwrap();
            let remaining = match input {
                FrameInput::Call(input) => input.gas_limit,
                FrameInput::Create(input) => input.gas_limit(),
                FrameInput::Empty => unreachable!(),
            };
            assert_eq!(result.result.tx_gas_used(), tx.gas_limit - remaining + 123);
            let mut expected = vec!["frame_start"];
            if override_at != "frame_start" {
                expected.push(start);
            }
            expected.extend([end, "frame_end"]);
            assert_eq!(evm.inspector.events, expected);
            assert_eq!(evm.inspector.initialized, 0);
            assert_eq!(evm.inspector.steps, 0);
        }
    }
}

#[test]
fn invalid_transactions_do_not_emit_root_hooks() {
    let ctx = Context::mainnet()
        .with_db(funded_db())
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM));
    let mut evm = ctx.build_mainnet_with_inspector(RootHooks::default());
    let tx = TxEnv::builder()
        .caller(CALLER)
        .nonce(7)
        .gas_limit(100)
        .build_fill();
    assert!(evm.inspect_tx(tx).is_err());
    assert!(evm.inspector.events.is_empty());
}
