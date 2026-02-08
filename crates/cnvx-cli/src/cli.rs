use crate::{
    solve::solve_file,
    style::{CLI_STYLING, after_help},
};
use clap::{Parser, Subcommand};

/// CNVX CLI
#[derive(Parser, Debug)]
#[command(
    name = "cnvx",
    about = "A CLI for modeling and solving optimization problems",
    disable_help_flag = true,
    disable_help_subcommand = false,
    infer_subcommands = true,
    after_help = after_help()
)]
#[clap(styles = CLI_STYLING)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display the current CNVX version
    #[command()]
    Version,

    /// Load and solve an optimization model from a file
    #[command(long_about = "Load and solve an optimization model from a file. \
                  Currently, CNVX only supports single-objective linear problems. \
                  Supported formats:\n\
                  - GNU Math Programming Language (.gmpl)\n\
                  - Mathematical Programming System (.mps)\n\n\
                  This command parses the model file, constructs the optimization problem, \
                  and prints the solution to the console.")]
    Solve {
        /// Path to the model file to solve. Supported formats: .gmpl, .mps
        file: String,
    },
}

pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Version) => {
            println!("CNVX version {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Solve { file }) => {
            solve_file(file)?;
        }
        None => {
            todo!("interactive mode not implemented yet");
        }
    }

    Ok(())
}
