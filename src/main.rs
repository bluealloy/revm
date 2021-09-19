#![allow(dead_code)]

mod opcode;
mod error;
mod stack;
mod machine;
mod memory;
mod context;
mod calls;
mod gasometer;

pub use machine::Machine;
pub use context::Handler;
pub use calls::*;

extern crate alloc;

fn main() {
    println!("Hello, world!");
}
