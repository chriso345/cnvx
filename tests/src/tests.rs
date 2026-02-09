use std::{path::Path, sync::LazyLock};

use clap::Parser;

use crate::args::CliArguments;

mod args;
mod netlib;

/// The parsed command line arguments.
static ARGS: LazyLock<CliArguments> = LazyLock::new(CliArguments::parse);

/// The tolerance for comparing floating point results in the Netlib tests.
const TOL: f64 = 1e-4;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

/// The directory to save and run the Netlib LP tests from.
static NETLIB_SUITE: &str = "tests/netlib_suite";

fn main() {
    setup();

    test();

    if ARGS.clean {
        clean();
    }
}

fn test() {
    netlib::run_tests(NETLIB_SUITE);
}

fn setup() {
    let workspace_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(std::path::Component::ParentDir);
    std::env::set_current_dir(workspace_dir).unwrap();

    std::fs::create_dir_all(NETLIB_SUITE).unwrap();
}

fn clean() {
    std::fs::remove_dir_all(NETLIB_SUITE).unwrap();
}
