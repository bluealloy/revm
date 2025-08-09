/// Macro that triggers `unreachable!` in debug builds but uses unchecked unreachable in release builds.
/// This provides better error messages during development while optimizing for performance in release.
#[macro_export]
macro_rules! debug_unreachable {
    ($($t:tt)*) => {
        if cfg!(debug_assertions) {
            unreachable!($($t)*);
        } else {
            unsafe { core::hint::unreachable_unchecked() };
        }
    };
}

/// Macro for asserting assumptions in debug builds.
/// In debug builds, this will trigger unreachable code if the assumption is false.
/// In release builds, this serves as an optimization hint.
#[macro_export]
macro_rules! assume {
    ($e:expr $(,)?) => {
        if !$e {
            debug_unreachable!(stringify!($e));
        }
    };

    ($e:expr, $($t:tt)+) => {
        if !$e {
            debug_unreachable!($($t)+);
        }
    };
}
