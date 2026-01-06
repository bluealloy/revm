use clap::Parser;
use color_eyre::eyre::Result;
use revme::cmd::MainCmd;

fn main() -> Result<()> {
    color_eyre::install()?;

    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    MainCmd::parse().run()?;

    Ok(())
}
