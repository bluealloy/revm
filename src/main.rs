#![allow(dead_code)]

mod db;
mod error;
mod evm;
mod machine;
mod models;
mod opcode;
mod spec;
mod subrutine;
mod utils;

pub use evm::{EVM, ExtHandler, Handler};
pub use machine::Machine;
pub use models::*;

use crate::{db::Database, spec::BerlinSpec};

extern crate alloc;

fn main() {
    println!("Hello, world!");
    let mut db = db::DummyDB;
    let context = GlobalContext::default();
    let evm = EVM::<BerlinSpec>::new(&mut db as &mut dyn Database, context);
}
