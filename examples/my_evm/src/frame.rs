use derive_where::derive_where;
use revm::{
    handler::{EthFrame, FrameResult, FrameTr},
    inspector::InspectorFrameTr,
    interpreter::{interpreter::EthInterpreter, interpreter_action::FrameInit, InterpreterTypes},
};

/// MyFrame wraps EthFrame and implements FrameTr
/// This demonstrates composition over inheritance while maintaining
/// the FrameTr interface for use in the EVM execution framework
/// Frame implementation for Ethereum.
#[derive_where(Clone, Debug; IW,
    <IW as InterpreterTypes>::Stack,
    <IW as InterpreterTypes>::Memory,
    <IW as InterpreterTypes>::Bytecode,
    <IW as InterpreterTypes>::ReturnData,
    <IW as InterpreterTypes>::Input,
    <IW as InterpreterTypes>::RuntimeFlag,
    <IW as InterpreterTypes>::Extend,
)]
pub struct MyFrame<IW: InterpreterTypes = EthInterpreter> {
    /// The underlying EthFrame that handles actual execution
    pub eth_frame: EthFrame<IW>,
}

/// Implement FrameTr trait for MyFrame by delegating to the inner EthFrame
impl<IW: InterpreterTypes> FrameTr for MyFrame<IW> {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

/// Default implementation that creates a default EthFrame
impl Default for MyFrame<EthInterpreter> {
    fn default() -> Self {
        Self {
            eth_frame: EthFrame::default(),
        }
    }
}

/// Implement common frame operations by delegating to EthFrame
impl MyFrame<EthInterpreter> {
    /// Create an invalid MyFrame
    pub fn invalid() -> Self {
        Self {
            eth_frame: EthFrame::invalid(),
        }
    }

    /// Check if the frame has finished execution
    pub fn is_finished(&self) -> bool {
        self.eth_frame.is_finished()
    }

    /// Set the finished state of the frame
    pub fn set_finished(&mut self, finished: bool) {
        self.eth_frame.set_finished(finished);
    }
}

impl InspectorFrameTr for MyFrame<EthInterpreter> {
    type IT = EthInterpreter;

    fn eth_frame(&mut self) -> Option<&mut EthFrame<Self::IT>> {
        Some(&mut self.eth_frame)
    }
}
