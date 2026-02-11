use std::{cell::Cell, process::ExitCode, sync::LazyLock};

use clap::Parser;

use crate::args::{CliArguments, Command};

mod args;
mod solve;
mod style;
mod version;

thread_local! {
    static EXIT: Cell<ExitCode> = const { Cell::new(ExitCode::SUCCESS) };
}

static ARGS: LazyLock<CliArguments> = LazyLock::new(|| {
    CliArguments::try_parse().unwrap_or_else(|error| {
        error.exit();
    })
});

fn main() -> ExitCode {
    sigpipe::reset();

    let res = dispatch();

    if let Err(msg) = res {
        set_failed();
        eprintln!("Error: {msg}");
    }

    EXIT.with(|cell| cell.get())
}

fn dispatch() -> Result<(), Box<dyn std::error::Error>> {
    match &ARGS.command {
        Command::Version(command) => crate::version::version(command)?,
        Command::Solve(command) => crate::solve::solve(command)?,
    }

    Ok(())
}

/// Ensure a failure exit code.
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}
