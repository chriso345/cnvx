use core::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::builder::{Styles, TypedValueParser};
use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};

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
    /// Compiles an input file into a supported output format.
    #[command(visible_alias = "s")]
    Solve(SolveCommand),

    /// Watches an input file and recompiles on changes.
    #[command(visible_alias = "v")]
    Version(VersionCommand),
    // /// Generates shell completion scripts.
    // Completions(CompletionsCommand),

    // /// Displays debugging information about CNVX.
    // Info(InfoCommand),
}

/// Solves a model from a file.
#[derive(Debug, Clone, Parser)]
pub struct SolveCommand {
    /// Arguments for solving a model from a file.
    #[clap(flatten)]
    pub args: SolveArgs,
}

/// Displays the current version of CNVX.
#[derive(Debug, Clone, Parser)]
pub struct VersionCommand {}

/// Arguments for compilation and watching.
#[derive(Debug, Clone, Args)]
pub struct SolveArgs {
    /// Path to input Model file file. Use `-` to read input from stdin.
    #[clap(
        required=true,
        value_parser = input_value_parser(),
        value_hint = ValueHint::FilePath,
    )]
    pub input: Input,

    /// The language of the input file. Required if reading from stdin.
    #[clap(
         required_if_eq("input", "-"),
         value_hint = ValueHint::FilePath,
     )]
    pub language_type: Option<LanguageType>,
}

/// An input that is either stdin or a real path.
#[derive(Debug, Clone)]
pub enum Input {
    /// Stdin, represented by `-`.
    Stdin,
    /// A non-empty path.
    Path(PathBuf),
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Input::Stdin => f.pad("stdin"),
            Input::Path(path) => path.display().fmt(f),
        }
    }
}

/// The clap value parser used by `SharedArgs.input`
fn input_value_parser() -> impl TypedValueParser<Value = Input> {
    clap::builder::OsStringValueParser::new().try_map(|value| {
        if value.is_empty() {
            Err(clap::Error::new(clap::error::ErrorKind::InvalidValue))
        } else if value == "-" {
            Ok(Input::Stdin)
        } else {
            Ok(Input::Path(value.into()))
        }
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum LanguageType {
    /// GNU Math Programming Language (.gmpl)
    #[value(name = "gmpl")]
    GMPL,

    /// Mathematical Programming System (.mps)
    #[value(name = "mps")]
    MPS,
}

impl Display for LanguageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LanguageType::GMPL => f.write_str("gmpl"),
            LanguageType::MPS => f.write_str("mps"),
        }
    }
}
