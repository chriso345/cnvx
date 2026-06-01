use cnvx::lp::{LpSolver, Solver}; // TODO: Behind a feature flag?

/// Entry point for the `cnvx solve` command.
///
/// Reads a model from a file (or stdin), parses it using the appropriate
/// [`LanguageParser`](cnvx_parse::LanguageParser), and solves it.
///
/// Solver selection is performed without a global registry: the parsed model's
/// [`Problem::kind()`](cnvx_core::problem::Problem::kind) is matched against the
/// available domain solvers by calling [`Solver::supports`].  Currently only LP
/// problems are supported; additional domain solvers can be added to the
/// `candidates` list below as new sub-crates are introduced.
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

    // Build a ranked list of candidate solvers.
    //
    // Each entry is tried in order; the first solver for which `supports`
    // returns `true` is used. Adding a new domain (e.g. `cnvx-nlp`) means
    // appending one line here and one `Cargo.toml` dependency — nothing else
    // needs to change.
    let mut candidates: Vec<Box<dyn Solver>> = vec![
        Box::new(LpSolver::new()),
        // ...
        // ...
    ];

    let solver = candidates.iter_mut().find(|s| s.supports(&model)).ok_or_else(|| {
        format!(
            "No solver supports '{}' problems. \
                 Is the required sub-crate linked?",
            cnvx_core::problem::Problem::kind(&model)
        )
    })?;

    println!("Using solver: {}", solver.name());

    let solution = solver.solve(&model).map_err(|e| format!("Solver error: {e}"))?;

    // TODO: Also support writing to a file.
    println!("{}", solution);

    Ok(())
}
