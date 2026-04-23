//! Local context that is filled by execution.
use context_interface::LocalContextTr;
use std::string::String;

/// Local context that is filled by execution.
#[derive(Clone, Debug, Default)]
pub struct LocalContext {
    /// Optional precompile error message to bubble up.
    pub precompile_error_message: Option<String>,
}

impl LocalContextTr for LocalContext {
    fn clear(&mut self) {
        self.precompile_error_message = None;
    }

    fn set_precompile_error_context(&mut self, output: String) {
        self.precompile_error_message = Some(output);
    }

    fn take_precompile_error_context(&mut self) -> Option<String> {
        self.precompile_error_message.take()
    }
}

impl LocalContext {
    /// Creates a new local context with default values.
    pub fn new() -> Self {
        Self::default()
    }
}
