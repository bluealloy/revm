use core::marker::PhantomData;

use revm_interpreter::primitives::SpecId;

use crate::{
    db::{Database, EmptyDB},
    handler::{MainnetHandle, RegisterHandler},
    primitives::Spec,
    Evm, EvmContext, Handler,
};

// /// Evm Builder
// pub struct EvmBuilder<'a, SPEC: Spec + 'static, EXT, DB: Database + 'a> {
//     database: DB,
//     evm: EvmContext<DB>,
//     handler: Handler<'a, Evm<'a, SPEC, EXT, DB>, EXT, DB>,
//     spec_id: SpecId,
// }

// impl<'a, EXT: RegisterHandler<'a, DB, EXT>, DB: Database + 'a> EvmBuilder<'a, SPEC, EXT, DB> {
//     pub fn new() -> EvmBuilder<'a, MainnetHandle, EmptyDB> {
//         EvmBuilder {
//             database: (),
//             //evm: EvmContext::default(),
//             //external: MainnetHandle::default(),
//             spec_id: SpecId::LATEST,
//         }
//     }

//     pub fn db<ODB: Database, OEXT: RegisterHandler<'a, ODB, OEXT>>(
//         self,
//         db: ODB,
//     ) -> EvmBuilder<'a, OEXT, ODB> {
//         EvmBuilder {
//             database: db,
//             //evm: self.evm,
//             //external: MainnetHandle::default(),
//             spec_id: self.spec_id,
//         }
//     }
// }
