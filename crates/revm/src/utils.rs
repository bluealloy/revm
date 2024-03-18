use tokio::runtime::{Builder, Handle, RuntimeFlavor};

/// internal utility function to call tokio feature and wait for output
#[inline]
pub(crate) fn tokio_block_on<F>(f: F) -> F::Output
where
    F: core::future::Future + Send,
    F::Output: Send,
{
    match Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            // This essentially equals to tokio::task::spawn_blocking because tokio doesn't
            // allow current_thread runtime to block_in_place
            RuntimeFlavor::CurrentThread => std::thread::scope(move |s| {
                s.spawn(move || {
                    Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(f)
                })
                    .join()
                    .unwrap()
            }),
            _ => tokio::task::block_in_place(move || handle.block_on(f)),
        },
        Err(_) => Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f),
    }
}