use cnvx::lp::SimplexSolver;
use cnvx_core::Solver;

use crate::lang::LanguageParser;

pub fn solve(
    command: &crate::args::SolveCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Solving model from file: {}", command.args.input);

    let contents = match &command.args.input {
        crate::args::Input::Stdin => {
            use std::io::{self, Read};
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
        crate::args::Input::Path(path) => std::fs::read_to_string(path)?,
    };

    let ext = match &command.args.input {
        crate::args::Input::Stdin => command
            .args
            .language_type
            .as_ref()
            .ok_or("language type is required when reading from stdin")?
            .to_string(),
        crate::args::Input::Path(path) => {
            path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase()
        }
    };

    let model = match ext.as_str() {
        "gmpl" => crate::lang::GMPLLanguage::new().parse(&contents)?,
        "ampl" => crate::lang::AMPLLanguage::new().parse(&contents)?,
        "mps" => crate::lang::MPSLanguage::new().parse(&contents)?,
        _ => return Err(format!("unsupported file type: {}", ext).into()),
    };

    let solver = SimplexSolver::default();
    let sol = solver.solve(&model)?;

    // TODO: Also support writing to a file, and saving to a file
    println!("{}", sol);

    Ok(())
}
