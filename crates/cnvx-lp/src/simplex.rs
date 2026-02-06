use std::ops::Neg;

use cnvx_core::*;
use cnvx_math::{DenseMatrix, Matrix};

/// A simplex solver for linear programs (LPs).
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::*;
/// # use cnvx_lp::SimplexSolver;
/// let mut model = Model::new();
/// let x = model.add_var().finish();
/// model += x.geq(0.0);
/// model += x.leq(10.0);
/// model.add_objective(Objective::maximize(x * 2.0).name("maximize_x"));
///
/// let solver = SimplexSolver::default();
/// let solution = solver.solve(&model).unwrap();
/// println!("Solution value: {}", solution.value(x));
/// ```
#[derive(Debug)]
pub struct SimplexSolver {
    /// The numerical tolerance used for feasibility and optimality checks.
    pub tolerance: f64,

    /// The maximum number of simplex iterations before terminating with an error.
    pub max_iterations: usize,
}

impl Default for SimplexSolver {
    fn default() -> Self {
        Self { tolerance: 1e-8, max_iterations: 1000 }
    }
}

impl Solver for SimplexSolver {
    fn solve(&self, model: &Model) -> Result<Solution, SolveError> {
        crate::validate::check_lp(model)?;

        let mut state = SimplexState::<DenseMatrix>::new(model);
        let (values, obj) = state.solve_lp(self.max_iterations, self.tolerance)?;

        Ok(Solution {
            values,
            objective_value: Some(obj),
            status: state.status,
        })
    }
}

/// Internal state for the simplex algorithm.
///
/// Tracks the current basis, non-basis variables, solution vector, objective value,
/// and the LP tableau.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SimplexState<'model, M: Matrix> {
    /// Reference to the LP model being solved.
    pub model: &'model Model,
    /// Current iteration count of the simplex algorithm.
    pub iteration: usize,

    /// Indices of basis variables in the tableau.
    pub basis: Vec<usize>,
    /// Indices of non-basis variables in the tableau.
    pub non_basis: Vec<usize>,
    /// Values of the basic variables.
    pub x_b: Vec<f64>,

    /// Constraint matrix `A`.
    pub a: M,
    /// Right-hand side vector `b`.
    pub b: Vec<f64>,
    /// Objective coefficients vector `c`.
    pub c: Vec<f64>,

    /// Current objective value.
    pub objective: f64,
    /// Solution status after solving (Optimal, Infeasible, Unbounded, etc.).
    pub status: SolveStatus,

    /// Whether the LP is a minimization problem.
    minimise: bool,
}

impl<'m, M: Matrix + Clone> SimplexState<'m, M> {
    /// Initialize a new simplex state from a given `Model`.
    ///
    /// Constructs the tableau, sets up artificial variables for inequalities, and
    /// computes the objective coefficients based on the problem's sense (min/max).
    pub fn new(model: &'m Model) -> Self {
        let n_vars = model.vars().len();
        let n_cons = model.constraints().len();

        let mut b = vec![0.0; n_cons];

        let mut n_total = n_vars;
        for cons in model.constraints().iter() {
            match cons.cmp {
                Cmp::Leq | Cmp::Geq => n_total += 2,
                Cmp::Eq => {}
            }
        }

        let mut a = M::new(n_cons, n_total);
        let mut c = vec![0.0; n_total];

        let minimise =
            model.objective().map(|o| o.sense == Sense::Minimize).unwrap_or(false);

        if let Some(obj) = model.objective() {
            for term in &obj.expr.terms {
                c[term.var.0] = match obj.sense {
                    Sense::Maximize => term.coeff,
                    Sense::Minimize => -term.coeff,
                };
            }
        }

        let mut extra_idx = n_vars;
        for (i, cons) in model.constraints().iter().enumerate() {
            b[i] = cons.rhs;
            for term in &cons.expr.terms {
                a.set(i, term.var.0, term.coeff);
            }
            match cons.cmp {
                Cmp::Leq => {
                    a.set(i, extra_idx, 1.0);
                    extra_idx += 1;
                }
                Cmp::Geq => {
                    a.set(i, extra_idx, -1.0);
                    extra_idx += 1;
                }
                Cmp::Eq => {}
            }
        }

        Self {
            model,
            iteration: 0,
            basis: Vec::new(),
            non_basis: (0..n_vars).collect(),
            x_b: vec![0.0; n_cons],
            a,
            b,
            c,
            objective: 0.0,
            status: SolveStatus::NotSolved,
            minimise,
        }
    }

    /// Solve the LP using the simplex method.
    ///
    /// Performs a two-phase simplex if necessary (phase 1 for feasibility, phase 2 for optimality).
    ///
    /// Returns the solution vector and the objective value.
    pub fn solve_lp(
        &mut self,
        max_iter: usize,
        tol: f64,
    ) -> Result<(Vec<f64>, f64), SolveError> {
        self.init_basis();
        let orig_n = self.a.cols();

        if self.try_phase2(max_iter, tol)? {
            return Ok(self.extract_solution(orig_n));
        }

        self.phase1(orig_n, max_iter, tol)?;
        self.phase2(max_iter, tol)?;

        Ok(self.extract_solution(orig_n))
    }

    /// Attempt to directly run phase 2 if the initial basis is feasible.
    fn try_phase2(&mut self, max_iter: usize, tol: f64) -> Result<bool, SolveError> {
        let mut bmat = self.build_bmat();
        match self.compute_basic_solution(&mut bmat) {
            Ok(xb) if xb.iter().all(|&v| v >= -tol) => {
                self.x_b = xb;
                self.remove_artificial_from_basis(&mut bmat, self.a.cols())
                    .map_err(SolveError::InvalidModel)?;
                self.run_simplex(&mut bmat, max_iter, tol)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Phase 1 of the two-phase simplex method to remove artificial variables.
    fn phase1(
        &mut self,
        orig_n: usize,
        max_iter: usize,
        tol: f64,
    ) -> Result<(), SolveError> {
        let (orig_a, orig_c, mut bmat) = self.setup_phase1(orig_n);
        self.run_simplex(&mut bmat, max_iter, tol)?;

        let sum_art: f64 = self
            .basis
            .iter()
            .enumerate()
            .map(|(i, &v)| self.c[v] * self.x_b[i])
            .sum::<f64>()
            .neg();

        if sum_art > tol {
            self.status = SolveStatus::Infeasible;
            return Ok(());
        }

        self.remove_artificial_from_basis(&mut bmat, orig_n)
            .map_err(SolveError::InvalidModel)?;

        self.a = orig_a;
        self.c = orig_c;
        Ok(())
    }

    /// Phase 2 of the simplex method to optimize the LP.
    fn phase2(&mut self, max_iter: usize, tol: f64) -> Result<(), SolveError> {
        let mut bmat = self.build_bmat();
        self.run_simplex(&mut bmat, max_iter, tol)
    }

    /// Initialize the basis using slack, surplus, and identity columns.
    pub fn init_basis(&mut self) {
        let m = self.a.rows();
        let n = self.a.cols();

        let mut basis = vec![None; m];
        let mut used = vec![false; n];

        for j in 0..n {
            let mut one_row = None;
            let mut ok = true;
            for i in 0..m {
                let v = self.a.get(i, j);
                if v.abs() > 1e-12 {
                    if (v - 1.0).abs() < 1e-12 {
                        if one_row.is_some() {
                            ok = false;
                            break;
                        }
                        one_row = Some(i);
                    } else {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                if let Some(r) = one_row {
                    if basis[r].is_none() {
                        basis[r] = Some(j);
                        used[j] = true;
                    }
                }
            }
        }

        if basis.iter().all(|b| b.is_some()) {
            self.basis = basis.into_iter().map(|b| b.unwrap()).collect();
            self.non_basis = (0..n).filter(|j| !used[*j]).collect();
        } else {
            self.basis = (0..m).collect();
            self.non_basis = (m..n).collect();
        }
    }

    /// Build the current basis matrix `B` from the full tableau `A`.
    pub fn build_bmat(&self) -> DenseMatrix {
        let m = self.a.rows();
        let mut bmat = DenseMatrix::new(m, m);
        for i in 0..m {
            for j in 0..m {
                bmat.set(i, j, self.a.get(i, self.basis[j]));
            }
        }
        bmat
    }

    /// Compute the values of the basic variables by solving `B x_B = b`.
    pub fn compute_basic_solution(
        &self,
        bmat: &mut DenseMatrix,
    ) -> Result<Vec<f64>, String> {
        let mut xb = self.b.clone();
        bmat.gaussian_elimination(&mut xb)
            .map_err(|e| format!("gauss failed: {e}"))?;
        Ok(xb)
    }

    /// Run the main simplex iteration loop.
    fn run_simplex(
        &mut self,
        bmat: &mut DenseMatrix,
        max_iter: usize,
        tol: f64,
    ) -> Result<(), SolveError> {
        let current_iter = self.iteration;
        for iter in current_iter..max_iter {
            self.iteration = iter;

            let pi = self.compute_duals(bmat)?;
            let Some((nb_pos, entering)) = self.choose_entering(&pi, tol) else {
                self.status = SolveStatus::Optimal;
                return Ok(());
            };

            let d = self.compute_direction(bmat, entering)?;
            let Some((leave_row, theta)) = self.choose_leaving(&d, tol) else {
                self.status = SolveStatus::Unbounded;
                return Ok(());
            };

            self.update_primal(&d, leave_row, theta);
            self.pivot(bmat, nb_pos, leave_row, entering);
            self.update_objective();
        }

        Err(SolveError::Other("max iterations reached".into()))
    }

    /// Compute dual variables for the current basis.
    fn compute_duals(&self, bmat: &DenseMatrix) -> Result<Vec<f64>, SolveError> {
        let m = bmat.rows();
        let mut pi = (0..m).map(|i| self.c[self.basis[i]]).collect::<Vec<_>>();

        let mut bt = DenseMatrix::new(m, m);
        for i in 0..m {
            for j in 0..m {
                bt.set(i, j, bmat.get(j, i));
            }
        }

        bt.gaussian_elimination(&mut pi)
            .map_err(|e| SolveError::Other(format!("dual solve failed: {e}")))?;

        Ok(pi)
    }

    /// Choose entering variable using reduced costs.
    fn choose_entering(&self, pi: &[f64], tol: f64) -> Option<(usize, usize)> {
        self.non_basis
            .iter()
            .enumerate()
            .filter_map(|(pos, &j)| {
                let rc = self.c[j]
                    - (0..pi.len()).map(|i| pi[i] * self.a.get(i, j)).sum::<f64>();
                (rc > tol).then_some((pos, j, rc))
            })
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            .map(|(pos, j, _)| (pos, j))
    }

    /// Compute the simplex direction `d = B^{-1} A_j`.
    fn compute_direction(
        &self,
        bmat: &mut DenseMatrix,
        entering: usize,
    ) -> Result<Vec<f64>, SolveError> {
        let mut d = (0..bmat.rows()).map(|i| self.a.get(i, entering)).collect::<Vec<_>>();

        bmat.gaussian_elimination(&mut d)
            .map_err(|e| SolveError::Other(format!("direction solve failed: {e}")))?;

        Ok(d)
    }

    /// Choose leaving variable using minimum ratio test.
    fn choose_leaving(&self, d: &[f64], tol: f64) -> Option<(usize, f64)> {
        (0..d.len())
            .filter(|&i| d[i] > tol)
            .map(|i| (i, self.x_b[i] / d[i]))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Update the primal solution vector `x_B` after a pivot.
    fn update_primal(&mut self, d: &[f64], leave: usize, theta: f64) {
        for i in 0..self.x_b.len() {
            self.x_b[i] -= theta * d[i];
            if self.x_b[i].abs() < 1e-12 {
                self.x_b[i] = 0.0;
            }
        }
        self.x_b[leave] = theta;
    }

    /// Perform pivot operations on the basis and non-basis sets.
    fn pivot(
        &mut self,
        bmat: &mut DenseMatrix,
        enter_pos: usize,
        leave_row: usize,
        entering: usize,
    ) {
        let leaving = self.basis[leave_row];
        self.basis[leave_row] = entering;
        self.non_basis[enter_pos] = leaving;

        for i in 0..bmat.rows() {
            bmat.set(i, leave_row, self.a.get(i, entering));
        }
    }

    /// Update the current objective value.
    fn update_objective(&mut self) {
        self.objective = self
            .basis
            .iter()
            .enumerate()
            .map(|(i, &v)| self.c[v] * self.x_b[i])
            .sum();
    }

    /// Prepare the LP for phase 1 of two-phase simplex by adding artificial variables.
    pub fn setup_phase1(&mut self, orig_n: usize) -> (M, Vec<f64>, DenseMatrix) {
        let m = self.a.rows();
        let n = self.a.cols();

        let mut a_aug = M::new(m, n + m);
        let mut b_aug = self.b.clone();

        for i in 0..m {
            if b_aug[i] < 0.0 {
                b_aug[i] = -b_aug[i];
                for j in 0..n {
                    a_aug.set(i, j, -self.a.get(i, j));
                }
            } else {
                for j in 0..n {
                    a_aug.set(i, j, self.a.get(i, j));
                }
            }

            for j in 0..m {
                a_aug.set(i, n + j, if i == j { 1.0 } else { 0.0 });
            }
        }

        let mut c_aug = vec![0.0; n + m];
        for j in 0..m {
            c_aug[n + j] = -1.0;
        }

        let orig_a = self.a.clone();
        let orig_c = self.c.clone();

        self.a = a_aug;
        self.c = c_aug;
        self.basis = (orig_n..orig_n + m).collect();
        self.non_basis = (0..orig_n).collect();
        self.x_b = b_aug;

        let mut bmat = DenseMatrix::new(m, m);
        for i in 0..m {
            for j in 0..m {
                bmat.set(i, j, self.a.get(i, self.basis[j]));
            }
        }

        (orig_a, orig_c, bmat)
    }

    /// Remove artificial variables from the basis once feasibility is established.
    pub fn remove_artificial_from_basis(
        &mut self,
        bmat: &mut DenseMatrix,
        orig_n: usize,
    ) -> Result<(), String> {
        let m = bmat.rows();
        for row in 0..m {
            if self.basis[row] >= orig_n {
                let mut pivot = None;
                for (nb_pos, &j) in self.non_basis.iter().enumerate() {
                    if j < orig_n && self.a.get(row, j).abs() > 1e-12 {
                        pivot = Some((nb_pos, j));
                        break;
                    }
                }

                if let Some((nb_pos, j)) = pivot {
                    let leaving = self.basis[row];
                    self.basis[row] = j;
                    self.non_basis[nb_pos] = leaving;
                    for i in 0..m {
                        bmat.set(i, row, self.a.get(i, j));
                    }
                } else if self.x_b[row].abs() > 1e-12 {
                    return Err(
                        "artificial variable left in basis with non-zero value".into()
                    );
                }
            }
        }
        Ok(())
    }

    /// Extract the final solution and objective value.
    pub fn extract_solution(&self, orig_n: usize) -> (Vec<f64>, f64) {
        let m = self.a.rows();
        let mut sol = vec![0.0; orig_n];

        for i in 0..m {
            if self.basis[i] < orig_n {
                sol[self.basis[i]] = self.x_b[i];
            }
        }

        let mut obj = self
            .basis
            .iter()
            .enumerate()
            .filter(|(_, v)| **v < orig_n)
            .map(|(i, v)| self.c[*v] * self.x_b[i])
            .sum::<f64>();

        if self.minimise {
            obj = -obj;
        }

        (sol, obj)
    }
}
