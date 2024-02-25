pub struct FunctionFrame {
    /// The index of the code container that this frame is executing.
    pub idx: usize,
    /// The program counter where frame execution should continue.
    pub pc: usize,
}

pub struct FunctionStack {
    stack: Vec<()>,
    current_idx: usize,
}

impl FunctionStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current_idx: 0,
        }
    }
}
