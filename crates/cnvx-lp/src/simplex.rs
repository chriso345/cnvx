use cnvx_core::*;
use cnvx_math::{DenseMatrix, Matrix};

pub struct SimplexSolver {
    pub tolerance: f64,
    pub max_iterations: usize,
}

impl Default for SimplexSolver {
    fn default() -> Self {
        Self { tolerance: 1e-8, max_iterations: 1000 }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SimplexState<'model, M: Matrix> {
    pub model: &'model Model,
    pub iteration: usize,

    pub basis: Vec<usize>,
    pub non_basis: Vec<usize>,

    pub x_b: Vec<f64>,

    pub a: M,
    pub b: Vec<f64>,
    pub c: Vec<f64>,

    pub objective: f64,

    minimise: bool,
}

impl Solver for SimplexSolver {
    fn solve(&self, model: &Model) -> Result<Solution, SolveError> {
        crate::validate::check_lp(model)?;

        let mut state = SimplexState::<DenseMatrix>::new(model);

        let m = state.a.rows();
        let n = state.a.cols();
        state.basis = (0..m).collect();
        state.non_basis = (m..n).collect();

        let mut bmat = DenseMatrix::new(m, m);
        for i in 0..m {
            for j in 0..m {
                bmat.set(i, j, state.a.get(i, state.basis[j]));
            }
        }
        let mut xb = state.b.clone();
        if let Err(e) = bmat.gaussian_elimination(&mut xb) {
            return Err(SolveError::InvalidModel(format!(
                "cannot solve initial basis: {}",
                e
            )));
        }
        for &v in &xb {
            if v < -self.tolerance {
                return Err(SolveError::Infeasible);
            }
        }
        state.x_b = xb;

        state
            .remove_artificial_from_basis()
            .map_err(|s| SolveError::InvalidModel(s))?;

        state.rsm(&mut bmat, self.max_iterations, self.tolerance)?;

        let mut sol = vec![0.0; n];
        for i in 0..m {
            sol[state.basis[i]] = state.x_b[i];
        }
        let mut obj = 0.0;
        for i in 0..m {
            obj += state.c[state.basis[i]] * state.x_b[i];
        }
        if state.minimise {
            obj = -obj;
        }
        Ok(Solution { values: sol, objective_value: Some(obj) })
    }
}

impl<'model, M: Matrix> SimplexState<'model, M> {
    pub fn new(model: &'model Model) -> Self {
        let n_vars = model.vars().len();
        let n_constraints = model.constraints().len();
        let mut a = M::new(n_constraints, n_vars);
        let mut b = vec![0.0; n_constraints];
        let mut c = vec![0.0; n_vars];
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
        for (i, cons) in model.constraints().iter().enumerate() {
            b[i] = cons.rhs;
            for term in &cons.expr.terms {
                a[i][term.var.0] = term.coeff;
            }
        }
        Self {
            model,
            iteration: 0,
            basis: Vec::new(),
            non_basis: (0..n_vars).collect(),
            x_b: vec![0.0; n_constraints],
            a,
            b,
            c,
            objective: 0.0,
            minimise,
        }
    }

    pub fn rsm(
        &mut self,
        bmat: &mut DenseMatrix,
        max_iter: usize,
        tol: f64,
    ) -> Result<(), SolveError> {
        let m = bmat.rows();
        for iter in 0..max_iter {
            self.iteration = iter;

            let mut cb = vec![0.0; m];
            for i in 0..m {
                cb[i] = self.c[self.basis[i]];
            }

            let mut bt = DenseMatrix::new(m, m);
            for i in 0..m {
                for j in 0..m {
                    bt.set(i, j, bmat.get(j, i));
                }
            }
            let mut pi = cb.clone();
            if let Err(e) = bt.gaussian_elimination(&mut pi) {
                return Err(SolveError::Other(format!("dual solve failed: {}", e)));
            }

            let mut entering: Option<(usize, usize, f64)> = None;
            for (nb_pos, &col_idx) in self.non_basis.iter().enumerate() {
                let mut a_s = vec![0.0; m];
                for i in 0..m {
                    a_s[i] = self.a.get(i, col_idx);
                }
                let mut dot = 0.0;
                for i in 0..m {
                    dot += pi[i] * a_s[i];
                }
                let rc = self.c[col_idx] - dot;
                if rc > tol {
                    if entering.is_none() || rc > entering.unwrap().2 {
                        entering = Some((nb_pos, col_idx, rc));
                    }
                }
            }
            if entering.is_none() {
                return Ok(());
            }
            let (enter_pos, s_col, _rc) = entering.unwrap();

            let mut a_s = vec![0.0; m];
            for i in 0..m {
                a_s[i] = self.a.get(i, s_col);
            }
            let mut d = a_s.clone();
            if let Err(e) = bmat.gaussian_elimination(&mut d) {
                return Err(SolveError::Other(format!("direction solve failed: {}", e)));
            }

            let mut theta = f64::INFINITY;
            let mut leave: Option<usize> = None;
            for i in 0..m {
                if d[i] > tol {
                    let ratio = self.x_b[i] / d[i];
                    if ratio < theta {
                        theta = ratio;
                        leave = Some(i);
                    }
                }
            }
            if leave.is_none() {
                return Err(SolveError::Unbounded);
            }
            let leave = leave.unwrap();

            for i in 0..m {
                self.x_b[i] = self.x_b[i] - theta * d[i];
                if self.x_b[i].abs() < 1e-12 {
                    self.x_b[i] = 0.0;
                }
            }
            self.x_b[leave] = theta;

            let entering_var = s_col;
            let leaving_var = self.basis[leave];
            self.update_b(bmat, enter_pos, leave, entering_var, leaving_var);

            self.objective = 0.0;
            for i in 0..m {
                self.objective += self.c[self.basis[i]] * self.x_b[i];
            }
        }
        Err(SolveError::Other("max iterations reached".into()))
    }

    pub fn update_b(
        &mut self,
        bmat: &mut DenseMatrix,
        enter_pos: usize,
        leave_row: usize,
        entering_var: usize,
        leaving_var: usize,
    ) {
        self.basis[leave_row] = entering_var;
        self.non_basis[enter_pos] = leaving_var;
        for i in 0..bmat.rows() {
            let v = self.a.get(i, entering_var);
            bmat.set(i, leave_row, v);
        }
    }

    pub fn remove_artificial_from_basis(&mut self) -> Result<(), String> {
        Ok(())
    }
}
