use clap::builder::Styles;
use clap::{Parser, Subcommand};

use crate::style::{CLI_STYLING, after_help};

const STYLES: Styles = CLI_STYLING;

/// CNVX's command-line arguments.
#[derive(Debug, Clone, Parser)]
#[clap(
    name = "cnvx",
    about = "A CLI for modeling and solving optimization problems",
    disable_help_flag = true,
    disable_help_subcommand = false,
    after_help = after_help(),
    max_term_width = 80,
    styles = STYLES,
)]
pub struct CliArguments {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// What to do.
#[derive(Debug, Clone, Subcommand)]
#[command()]
pub enum Command {
    /// Watches an input file and recompiles on changes.
    #[command(visible_alias = "v")]
    Version(VersionCommand),
}

/// Displays the current version of CNVX.
#[derive(Debug, Clone, Parser)]
pub struct VersionCommand {}
