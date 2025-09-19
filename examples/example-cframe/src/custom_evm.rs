use crate::custom_frame::{CustomFrame, FrameStats};
use revm::{
    handler::FrameTr,
    interpreter::interpreter::{EthInterpreter, Interpreter},
};

/// Simplified custom EVM demonstration for CustomFrame usage
pub struct CustomEvm<FRAME>
where
    FRAME: FrameTr + 'static,
{
    /// Frame stack for managing execution frames
    pub frame_stack: Vec<FRAME>,
    /// Statistics collector
    pub stats: FrameStats,
}

impl<FRAME> CustomEvm<FRAME>
where
    FRAME: FrameTr + 'static,
{
    /// Create a new CustomEvm instance
    pub fn new() -> Self {
        Self {
            frame_stack: Vec::new(),
            stats: FrameStats::new(),
        }
    }

    /// Get statistics
    pub fn stats(&self) -> &FrameStats {
        &self.stats
    }

    /// Print execution statistics
    pub fn print_stats(&self) {
        self.stats.print_stats();
    }

    /// Add a frame to the stack (for demonstration)
    pub fn push_frame(&mut self, frame: FRAME) {
        self.frame_stack.push(frame);
    }

    /// Pop a frame from the stack (for demonstration)
    pub fn pop_frame(&mut self) -> Option<FRAME> {
        self.frame_stack.pop()
    }

    /// Get current frame stack depth
    pub fn depth(&self) -> usize {
        self.frame_stack.len()
    }

    /// Record frame statistics if it's a CustomFrame
    pub fn record_frame_stats(&mut self, frame: &FRAME) {
        // Use type erasure to check if it's a CustomFrame
        if let Some(custom_frame) = (frame as &dyn std::any::Any).downcast_ref::<CustomFrame>() {
            self.stats.record_frame(custom_frame);
        }
    }
}

/// Example implementation showing how to create and manage custom frames
pub struct FrameManager {
    interpreter_factory: InterpreterFactory,
}

impl FrameManager {
    pub fn new() -> Self {
        Self {
            interpreter_factory: InterpreterFactory::new(),
        }
    }

    /// Create a new CustomFrame for demonstration
    pub fn create_demo_frame(&mut self, tag: &str) -> CustomFrame<EthInterpreter> {
        use revm::{
            context_interface::journaled_state::JournalCheckpoint,
            handler::{CallFrame, FrameData},
            interpreter::FrameInput,
        };

        let data = FrameData::Call(CallFrame {
            return_memory_range: 0..32,
        });

        let interpreter = self.interpreter_factory.create_interpreter();

        CustomFrame::new(
            data,
            FrameInput::Empty,
            0,
            JournalCheckpoint::default(),
            interpreter,
            tag.to_string(),
        )
    }
}

/// Factory for creating interpreters
pub struct InterpreterFactory;

impl InterpreterFactory {
    pub fn new() -> Self {
        Self
    }

    /// Create a new interpreter instance
    pub fn create_interpreter(&self) -> Interpreter<EthInterpreter> {
        use revm::interpreter::Gas;

        // Create a basic interpreter with some gas
        let mut interpreter = Interpreter::default();

        // Set some initial gas for demonstration
        interpreter.gas = Gas::new(100_000);

        interpreter
    }
}

/// Demonstrate that CustomFrame can be used with trait objects
pub fn demonstrate_frame_trait_objects() {
    // Vector of FrameTr trait objects
    let mut frames: Vec<Box<dyn FrameTr<
        FrameResult = revm::handler::FrameResult,
        FrameInit = revm::interpreter::interpreter_action::FrameInit
    >>> = Vec::new();

    let mut manager = FrameManager::new();

    // Create CustomFrames and store them as trait objects
    for i in 0..3 {
        let frame = manager.create_demo_frame(&format!("trait_object_{}", i + 1));
        frames.push(Box::new(frame));
    }

    println!("✅ Created {} frames as FrameTr trait objects", frames.len());
    println!("   This demonstrates polymorphic frame handling in REVM");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_frame_creation() {
        let mut manager = FrameManager::new();
        let frame = manager.create_demo_frame("test_frame");

        assert_eq!(frame.tag, "test_frame");
        assert_eq!(frame.depth, 0);
        assert_eq!(frame.frame_type(), "CALL");
    }

    #[test]
    fn test_custom_evm_creation() {
        let evm = CustomEvm::<CustomFrame<EthInterpreter>>::new();
        assert_eq!(evm.depth(), 0);
        assert_eq!(evm.stats.total_frames, 0);
    }

    #[test]
    fn test_frame_stack_operations() {
        let mut evm = CustomEvm::<CustomFrame<EthInterpreter>>::new();
        let mut manager = FrameManager::new();

        let frame = manager.create_demo_frame("test_frame");
        evm.push_frame(frame);

        assert_eq!(evm.depth(), 1);

        let popped = evm.pop_frame().unwrap();
        assert_eq!(popped.tag, "test_frame");
        assert_eq!(evm.depth(), 0);
    }
}