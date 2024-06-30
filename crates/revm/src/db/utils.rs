use tokio::runtime::{Handle, Runtime};

// Hold a tokio runtime handle or full runtime
#[derive(Debug)]
pub(crate) enum HandleOrRuntime {
    Handle(Handle),
    Runtime(Runtime),
}

impl HandleOrRuntime {
    #[inline]
    pub(crate) fn block_on<F>(&self, f: F) -> F::Output
    where
        F: std::future::Future + Send,
        F::Output: Send,
    {
        match self {
            Self::Handle(handle) => tokio::task::block_in_place(move || handle.block_on(f)),
            Self::Runtime(rt) => rt.block_on(f),
        }
    }
}
