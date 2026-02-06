use crate::command::run::command_run;
use clap::{Parser, Subcommand};

/// CNVX CLI
#[derive(Parser, Debug)]
#[command(name = "cnvx")]
#[command(about = "CNVX optimization CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show version
    Version,

    /// Run a model file
    Run {
        /// Path to the model file (GMPL or AMPL)
        file: String,
    },

    /// Start REPL (unimplemented)
    Repl,
}

pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Version => {
            println!("CNVX version {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Run { file } => {
            command_run(file)?;
        }
        Commands::Repl => {
            println!("REPL not implemented yet");
        }
    }

    Ok(())
}
