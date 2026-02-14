use cnvx_core::{Model, Solution, SolveError, Solver};
use cnvx_math::Matrix;

/// State used internally by the simplex solver.
#[derive(Clone)]
pub struct DualSimplexState<'model, A: Matrix> {
    pub model: &'model Model,
    pub iteration: usize,
    pub basis: Vec<usize>,
    pub non_basis: Vec<usize>,
    pub x_b: Vec<f64>,
    pub a: A,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub objective: f64,
}

impl<'model, A: Matrix> DualSimplexState<'model, A> {
    pub fn new(model: &'model Model) -> Self {
        Self {
            model,
            iteration: 0,
            basis: vec![],
            non_basis: vec![],
            x_b: vec![],
            a: A::new(0, 0),
            b: vec![],
            c: vec![],
            objective: 0.0,
        }
    }
}

/// Generic Simplex solver.
pub struct DualSimplexSolver<'model, A: Matrix> {
    pub state: DualSimplexState<'model, A>,
    pub tolerance: f64,
    pub max_iter: usize,
    pub logging: bool,
}

impl<'model, A: Matrix> Default for DualSimplexSolver<'model, A> {
    fn default() -> Self {
        panic!("Use SimplexSolver::new(model) to construct");
    }
}

impl<'model, A: Matrix> Solver<'model, DualSimplexState<'model, A>>
    for DualSimplexSolver<'model, A>
{
    const ALGORITHM_NAME: &'static str = "Dual Simplex";

    fn new(model: &'model Model) -> Self {
        Self {
            state: DualSimplexState::new(model),
            tolerance: 1e-8,
            max_iter: 1000,
            logging: false,
        }
    }

    fn solve(&mut self) -> Result<Solution, SolveError> {
        // TODO: implement primal simplex iterations

        Err(SolveError::Unsupported(
            "DualSimplexSolver.solve() not implemented yet".to_string(),
        ))
    }

    fn get_state(&self) -> &DualSimplexState<'model, A> {
        &self.state
    }

    fn get_objective_value(&self) -> f64 {
        self.state.objective
    }

    fn get_solution(&self) -> Vec<f64> {
        // TODO: reconstruct full solution vector from x_b and non-basic vars
        vec![]
    }
}
