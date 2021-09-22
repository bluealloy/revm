#![allow(dead_code)]

mod db;
mod error;
mod evm;
mod machine;
mod models;
mod opcode;
mod spec;
mod subroutine;
mod util;

pub use evm::{ExtHandler, Handler, EVM};
pub use machine::Machine;
pub use models::*;

use crate::{db::{Database, DummyDB}, spec::BerlinSpec};

extern crate alloc;

fn main() {
    println!("Hello, world!");
    let mut db = db::DummyDB;
    let envs = GlobalEnv::default();
    let evm = EVM::<BerlinSpec,DummyDB>::new(&mut db, envs);
}
