#![allow(dead_code)]

mod opcode;
mod error;
mod stack;
mod subrutine;
mod machine;
mod memory;
mod evm;
mod models;
mod spec;
mod utils;

pub use machine::Machine;
pub use evm::{ExtHandler,Handler};
pub use models::*;

extern crate alloc;

fn main() {
    println!("Hello, world!");
}
