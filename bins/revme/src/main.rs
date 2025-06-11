use clap::Parser;
use revme::cmd::{Error, MainCmd};

fn main() -> Result<(), Error> {
    set_thread_panic_hook();
    MainCmd::parse().run().inspect_err(|e| println!("{e:?}"))
}

/// Sets thread panic hook, useful for having tests that panic.
fn set_thread_panic_hook() {
    use std::{
        backtrace::Backtrace,
        panic::{set_hook, take_hook},
        process::exit,
    };
    let orig_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        println!("Custom backtrace: {}", Backtrace::capture());
        orig_hook(panic_info);
        exit(1);
    }));
}
