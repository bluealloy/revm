use clap::Parser;
use revme::cmd::MainCmd;
use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    match MainCmd::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error:\n- {e}\n- {e:#?}");
            ExitCode::FAILURE
        }
    }
}
