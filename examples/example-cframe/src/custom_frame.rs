use revm::{
    context_interface::journaled_state::JournalCheckpoint,
    handler::{FrameTr, FrameData, FrameResult, CallFrame},
    interpreter::{
        interpreter::{EthInterpreter, Interpreter},
        interpreter_action::FrameInit,
        FrameInput, InterpreterTypes,
    },
};
use std::time::Instant;

/// Custom frame that tracks execution metrics while implementing FrameTr
/// This demonstrates how to extend REVM's frame system with custom functionality
pub struct CustomFrame<IW: InterpreterTypes = EthInterpreter> {
    /// Frame-specific data (Call, Create, or EOFCreate)
    pub data: FrameData,
    /// Input data for the frame
    pub input: FrameInput,
    /// Current call depth in the execution stack
    pub depth: usize,
    /// Journal checkpoint for state reversion
    pub checkpoint: JournalCheckpoint,
    /// Interpreter instance for executing bytecode
    pub interpreter: Interpreter<IW>,
    /// Whether the frame has finished execution
    pub is_finished: bool,

    // Custom fields for tracking metrics
    /// Time when the frame was created
    pub created_at: Instant,
    /// Gas used by this frame
    pub gas_used: u64,
    /// Custom tag for identifying this frame
    pub tag: String,
}

impl<IW: InterpreterTypes> CustomFrame<IW> {
    /// Create a new custom frame with tracking
    pub fn new(
        data: FrameData,
        input: FrameInput,
        depth: usize,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter<IW>,
        tag: String,
    ) -> Self {
        Self {
            data,
            input,
            depth,
            checkpoint,
            interpreter,
            is_finished: false,
            created_at: Instant::now(),
            gas_used: 0,
            tag,
        }
    }

    /// Get the duration this frame has been executing
    pub fn duration(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Update gas usage tracking
    pub fn update_gas_usage(&mut self) {
        let initial_gas = self.interpreter.gas.limit();
        let remaining_gas = self.interpreter.gas.remaining();
        self.gas_used = initial_gas.saturating_sub(remaining_gas);
    }

    /// Get frame type as string
    pub fn frame_type(&self) -> &str {
        match &self.data {
            FrameData::Call(_) => "CALL",
            FrameData::Create(_) => "CREATE",
        }
    }

    /// Log frame execution start
    pub fn log_start(&self) {
        println!(
            "🚀 Starting {} frame '{}' at depth {} (gas limit: {})",
            self.frame_type(),
            self.tag,
            self.depth,
            self.interpreter.gas.limit()
        );
    }

    /// Log frame execution end
    pub fn log_end(&self) {
        println!(
            "✅ Finished {} frame '{}' at depth {} (gas used: {}, duration: {:?})",
            self.frame_type(),
            self.tag,
            self.depth,
            self.gas_used,
            self.duration()
        );
    }
}

/// Implement the FrameTr trait for our CustomFrame
impl<IW: InterpreterTypes> FrameTr for CustomFrame<IW> {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

/// Default implementation for CustomFrame
impl Default for CustomFrame<EthInterpreter> {
    fn default() -> Self {
        Self {
            data: FrameData::Call(CallFrame {
                return_memory_range: 0..0,
            }),
            input: FrameInput::Empty,
            depth: 0,
            checkpoint: JournalCheckpoint::default(),
            interpreter: Interpreter::default(),
            is_finished: false,
            created_at: Instant::now(),
            gas_used: 0,
            tag: "default".to_string(),
        }
    }
}

/// Frame factory that creates CustomFrames with tracking
pub struct CustomFrameFactory {
    frame_counter: usize,
}

impl CustomFrameFactory {
    pub fn new() -> Self {
        Self { frame_counter: 0 }
    }

    /// Create a new CustomFrame with automatic tagging
    pub fn create_frame<IW: InterpreterTypes>(
        &mut self,
        data: FrameData,
        input: FrameInput,
        depth: usize,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter<IW>,
    ) -> CustomFrame<IW> {
        self.frame_counter += 1;
        let tag = format!("frame_{}", self.frame_counter);

        let frame = CustomFrame::new(
            data,
            input,
            depth,
            checkpoint,
            interpreter,
            tag,
        );

        frame.log_start();
        frame
    }
}

/// Statistics collector for CustomFrames
pub struct FrameStats {
    pub total_frames: usize,
    pub total_gas_used: u64,
    pub max_depth: usize,
    pub total_duration_ms: u128,
    pub frame_types: std::collections::HashMap<String, usize>,
}

impl FrameStats {
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            total_gas_used: 0,
            max_depth: 0,
            total_duration_ms: 0,
            frame_types: std::collections::HashMap::new(),
        }
    }

    pub fn record_frame<IW: InterpreterTypes>(&mut self, frame: &CustomFrame<IW>) {
        self.total_frames += 1;
        self.total_gas_used += frame.gas_used;
        self.max_depth = self.max_depth.max(frame.depth);
        self.total_duration_ms += frame.duration().as_millis();

        *self.frame_types
            .entry(frame.frame_type().to_string())
            .or_insert(0) += 1;
    }

    pub fn print_stats(&self) {
        println!("\n📊 Frame Execution Statistics:");
        println!("  Total frames executed: {}", self.total_frames);
        println!("  Total gas used: {}", self.total_gas_used);
        println!("  Maximum call depth: {}", self.max_depth);
        println!("  Total execution time: {}ms", self.total_duration_ms);
        println!("  Frame types:");
        for (frame_type, count) in &self.frame_types {
            println!("    {}: {}", frame_type, count);
        }
    }
}