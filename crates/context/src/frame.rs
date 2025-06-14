use crate::frame_data::FrameData;
use context_interface::journaled_state::JournalCheckpoint;
use derive_where::derive_where;
use interpreter::{interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterTypes};

//#[derive(Debug, Clone)]
#[derive_where(Clone, Debug; IW,
    <IW as InterpreterTypes>::Stack,
    <IW as InterpreterTypes>::Memory,
    <IW as InterpreterTypes>::Bytecode,
    <IW as InterpreterTypes>::ReturnData,
    <IW as InterpreterTypes>::Input,
    <IW as InterpreterTypes>::SubRoutineStack,
    <IW as InterpreterTypes>::RuntimeFlag,
    <IW as InterpreterTypes>::Extend,
)]
pub struct EthFrameInner<IW: InterpreterTypes = EthInterpreter> {
    pub data: FrameData,
    pub input: FrameInput,
    pub depth: usize,
    pub checkpoint: JournalCheckpoint,
    pub interpreter: Interpreter<IW>,
}
