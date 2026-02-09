use cnvx::lp::SimplexSolver;
use cnvx_core::Solver;

use crate::lang::{AMPLLanguage, GMPLLanguage, LanguageParser, MPSLanguage};
use std::fs;

pub fn solve_file(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file)?;
    let ext = file.split('.').next_back().unwrap_or("");

    let model = match ext {
        "gmpl" => GMPLLanguage::new().parse(&contents)?,
        "ampl" => AMPLLanguage::new().parse(&contents)?,
        "mps" => MPSLanguage::new().parse(&contents)?,
        _ => return Err(format!("unsupported file type: {}", ext).into()),
    };

    let solver = SimplexSolver::default();
    let sol = solver.solve(&model)?;

    println!("{}", sol);

    Ok(())
}
