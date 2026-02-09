use clap::{Parser, arg, command};

/// CNVX Integration Test Runner
#[derive(Debug, Clone, Parser)]
#[command(bin_name = "cargo test --workspace --test tests --")]
#[clap(name = "cnvx-test", author)]
pub struct CliArguments {
    /// Displays only one line per test, hiding details about failures.
    #[arg(short, long)]
    pub verbose: bool,
    // /// How many threads to spawn when running the tests.
    // #[arg(short = 'j', long)]
    // pub num_threads: Option<usize>,
    /// Removes the temporary directory used for storing the Netlib LP test files after the tests are run.
    #[arg(long)]
    pub clean: bool,
}
