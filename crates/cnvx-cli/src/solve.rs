use cnvx::AutoSolver;
use cnvx_math::DenseMatrix;

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
        let mut solver = AutoSolver::<DenseMatrix>::new(&model);
        // Match on the solution (Ok or Err) and print the appropriate messagej
        let sol = match solver.solve() {
            Ok(solution) => solution,
            Err(e) => {
                println!("Solver error: {}", e);
                return Err("Solver error".into());
            }
        };

        // TODO: Also support writing to a file, and saving to a file
        println!("{}", sol);
    } else {
        println!("Failed to parse model");
        return Err("Failed to parse model".into());
    }

    Ok(())
}
