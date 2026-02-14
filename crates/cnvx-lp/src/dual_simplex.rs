use cnvx_core::{Model, Solution, SolveError, Solver};
use cnvx_math::{DenseMatrix, Matrix};

/// Generic Simplex solver.
pub struct DualSimplexSolver<'model> {
    state: State<'model>,
    pub tolerance: f64,
    pub max_iter: usize,
    pub logging: bool,
}

impl<'model> Default for DualSimplexSolver<'model> {
    fn default() -> Self {
        panic!("Use SimplexSolver::new(model) to construct");
    }
}

impl<'model> Solver<'model> for DualSimplexSolver<'model> {
    fn new(model: &'model Model) -> Self {
        Self {
            state: State::Dense(DualSimplexState::new(model)),
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

    fn get_objective_value(&self) -> f64 {
        match &self.state {
            State::Dense(s) => s.objective,
        }
    }

    fn get_solution(&self) -> Vec<f64> {
        // TODO: reconstruct full solution vector from x_b and non-basic vars
        vec![]
    }
}

enum State<'model> {
    Dense(DualSimplexState<'model, DenseMatrix>),
}

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
