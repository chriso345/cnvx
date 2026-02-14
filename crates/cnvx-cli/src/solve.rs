use cnvx::lp::PrimalSimplexSolver;
use cnvx_core::Solver;

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

    if let Ok(model) = cnvx_parse::parse(&contents, &ext) {
        let solver = PrimalSimplexSolver::default();
        let sol = solver.solve(&model)?;

        // TODO: Also support writing to a file, and saving to a file
        println!("{}", sol);
    } else {
        println!("Failed to parse model");
        return Err("Failed to parse model".into());
    }

    Ok(())
}
