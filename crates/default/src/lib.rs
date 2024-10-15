//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod default;
pub mod wiring;

pub use wiring::{DefaultEthereumWiring, EthereumWiring};

pub struct Context<Tx, Block, DB> {
    pub tx: Tx,
    pub block: Block,
    pub db: DB,
}

pub type DContext = Context<(), (), ()>;

pub struct Frame {}

pub struct FrameOutput {}

fn init_frame(frame: Frame) {}

// impl Handler {
//
// }

pub trait Exec {
    fn exec(&self, frame: impl AsRef<Frame>) -> FrameOutput;
    fn run(&self, context: DContext) -> FrameOutput;
    fn returnn(&self, frame: impl AsRef<Frame>);
}
