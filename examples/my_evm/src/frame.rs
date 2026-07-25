//! Custom frame that wraps [`EthFrame`](revm::handler::EthFrame).
//!
//! Useful when a custom EVM variant needs to carry additional per-call-frame
//! state or intercept frame creation, execution, or return.
use revm::{
    handler::{evm::FrameTr, EthFrame, FrameResult},
    inspector::InspectorFrame,
    interpreter::{interpreter::EthInterpreter, interpreter_action::FrameInit},
};

/// A custom frame that wraps [`EthFrame`] and delegates execution to it.
#[derive(Debug)]
pub struct MyFrame {
    /// The wrapped Ethereum frame that performs the actual execution.
    pub eth_frame: EthFrame<EthInterpreter>,
}

impl MyFrame {
    /// Creates a new invalid [`MyFrame`].
    pub fn invalid() -> Self {
        Self {
            eth_frame: EthFrame::invalid(),
        }
    }

    /// Returns true if the frame has finished execution.
    pub const fn is_finished(&self) -> bool {
        self.eth_frame.is_finished()
    }
}

impl FrameTr for MyFrame {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

/// Exposes the inner [`EthFrame`] so inspectors can trace through the custom frame.
impl InspectorFrame for MyFrame {
    type IT = EthInterpreter;

    fn eth_frame(&mut self) -> Option<&mut EthFrame<EthInterpreter>> {
        Some(&mut self.eth_frame)
    }
}
