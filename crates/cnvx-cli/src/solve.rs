use cnvx_lp::{LpSolver, Solver};

/// Entry point for the `cnvx solve` command.
///
/// Reads a model from a file (or stdin), parses it using the appropriate
/// [`LanguageParser`](cnvx_parse::LanguageParser), and solves it.
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

    let model = cnvx_parse::parse(&contents, &ext)
        .map_err(|e| format!("Failed to parse model: {e}"))?;

    let mut solver = LpSolver::new();
    println!("Using solver: {}", solver.name());

    let solution = solver.solve(&model).map_err(|e| format!("Solver error: {e}"))?;

    // TODO: Also support writing to a file.
    println!("{}", solution);

    Ok(())
}
