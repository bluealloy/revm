use context::{ContextTr, Database, JournalTr};
use context_interface::{Cfg, Transaction};
use interpreter::{
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, FrameInput,
};
use primitives::{hardfork::SpecId, TxKind};
use state::Bytecode;
use std::boxed::Box;

/// Creates the first [`FrameInput`] from the transaction, spec and gas limit.
#[inline]
pub fn create_init_frame<CTX: ContextTr>(
    ctx: &mut CTX,
    gas_limit: u64,
) -> Result<FrameInput, <<CTX::Journal as JournalTr>::Database as Database>::Error> {
    let is_prague = ctx.cfg().spec().into().is_enabled_in(SpecId::PRAGUE);
    let (tx, journal) = ctx.tx_journal_mut();
    let input = tx.input().clone();

    match tx.kind() {
        TxKind::Call(target_address) => {
            let account = &journal.load_account_with_code(target_address)?.info;

            let known_bytecode = if is_prague {
                if let Some(delegated_address) =
                    account.code.as_ref().and_then(Bytecode::eip7702_address)
                {
                    let account = &journal.load_account_with_code(delegated_address)?.info;
                    (
                        account.code_hash(),
                        account.code.clone().unwrap_or_default(),
                    )
                } else {
                    (
                        account.code_hash(),
                        account.code.clone().unwrap_or_default(),
                    )
                }
            } else {
                (
                    account.code_hash(),
                    account.code.clone().unwrap_or_default(),
                )
            };
            Ok(FrameInput::Call(Box::new(CallInputs {
                input: CallInput::Bytes(input),
                gas_limit,
                target_address,
                bytecode_address: target_address,
                known_bytecode,
                caller: tx.caller(),
                value: CallValue::Transfer(tx.value()),
                scheme: CallScheme::Call,
                is_static: false,
                return_memory_offset: 0..0,
            })))
        }
        TxKind::Create => Ok(FrameInput::Create(Box::new(CreateInputs::new(
            tx.caller(),
            CreateScheme::Create,
            tx.value(),
            input,
            gas_limit,
        )))),
    }
}

#[cfg(test)]
mod tests {
    use super::create_init_frame;
    use crate::MainContext;
    use context::{Context, TxEnv};
    use context_interface::ContextSetters;
    use database::{CacheDB, EmptyDB};
    use interpreter::FrameInput;
    use primitives::{hardfork::SpecId, Address, TxKind};
    use state::{AccountInfo, Bytecode};

    #[test]
    fn create_init_frame_pre_prague_does_not_follow_delegation() {
        let target = Address::with_last_byte(0x10);
        let delegate = Address::with_last_byte(0x20);
        let target_code = Bytecode::new_eip7702(delegate);
        let delegate_code = Bytecode::new_legacy(vec![0x00].into());

        let mut db = CacheDB::<EmptyDB>::default();
        db.insert_account_info(
            target,
            AccountInfo::default().with_code(target_code.clone()),
        );
        db.insert_account_info(delegate, AccountInfo::default().with_code(delegate_code));

        let mut ctx = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::CANCUN));
        ctx.set_tx(
            TxEnv::builder()
                .kind(TxKind::Call(target))
                .caller(Address::with_last_byte(0xAA))
                .gas_limit(100_000)
                .build()
                .expect("tx setup should be valid"),
        );

        let frame = create_init_frame(&mut ctx, 100_000).expect("frame creation should succeed");

        let FrameInput::Call(inputs) = frame else {
            panic!("expected call frame");
        };

        assert_eq!(inputs.known_bytecode.1, target_code);
        assert_eq!(inputs.known_bytecode.0, target_code.hash_slow());
    }

    #[test]
    fn create_init_frame_prague_follows_delegation() {
        let target = Address::with_last_byte(0x11);
        let delegate = Address::with_last_byte(0x21);
        let delegate_code = Bytecode::new_legacy(vec![0x00].into());

        let mut db = CacheDB::<EmptyDB>::default();
        db.insert_account_info(
            target,
            AccountInfo::default().with_code(Bytecode::new_eip7702(delegate)),
        );
        db.insert_account_info(
            delegate,
            AccountInfo::default().with_code(delegate_code.clone()),
        );

        let mut ctx = Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SpecId::PRAGUE));
        ctx.set_tx(
            TxEnv::builder()
                .kind(TxKind::Call(target))
                .caller(Address::with_last_byte(0xAA))
                .gas_limit(100_000)
                .build()
                .expect("tx setup should be valid"),
        );

        let frame = create_init_frame(&mut ctx, 100_000).expect("frame creation should succeed");

        let FrameInput::Call(inputs) = frame else {
            panic!("expected call frame");
        };

        assert_eq!(inputs.known_bytecode.1, delegate_code);
        assert_eq!(inputs.known_bytecode.0, delegate_code.hash_slow());
    }
}
