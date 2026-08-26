use std::time::{Duration, Instant};

use cnvx_core::Status;
use cnvx_math::{Matrix, Vector};

use crate::standard_form::StandardForm;

/// The outcome of running the simplex engine to a terminal state.
pub(crate) struct SimplexResult {
    pub status: Status,
    /// `tableau.get(r, ..)` is row `r`'s coefficients across every column,
    /// reduced so that `basis[r]`'s own column is the unit vector `e_r`.
    pub tableau: Matrix,
    /// The right-hand side of each row in the final tableau: the value of
    /// `basis[r]` when every nonbasic column is `0`.
    pub rhs: Vector,
    /// `basis[r]` is the column index currently basic in row `r`.
    pub basis: Vec<usize>,
    /// Reduced costs at the final tableau, under whichever cost vector was
    /// active when the engine stopped (Phase 2's real objective, unless
    /// the solve ended during Phase 1).
    pub reduced_costs: Vector,
}

/// Runs Phase 1 (drive the artificial variables to `0`, or prove the
/// system infeasible) followed by Phase 2 (optimize the real objective)
/// and returns the final tableau.
pub(crate) fn solve(
    form: &StandardForm,
    tolerance: f64,
    max_iterations: u32,
    time_limit: Option<Duration>,
) -> SimplexResult {
    let deadline = time_limit.map(|d| Instant::now() + d);
    let mut tableau = form.rows.clone();
    let mut rhs: Vec<f64> = form.rhs.to_vec();
    let mut basis = form.artificial_cols.clone();

    let mut is_artificial = vec![false; form.num_cols];
    for &a in &form.artificial_cols {
        is_artificial[a] = true;
    }
    // Artificial columns are never allowed to re-enter the basis in
    // either phase: their only role is to seed a feasible Phase 1 start.
    let excluded = is_artificial.clone();

    let mut phase1_cost = vec![0.0; form.num_cols];
    for &a in &form.artificial_cols {
        phase1_cost[a] = 1.0;
    }
    let phase1_cost = Vector::from(phase1_cost);

    let mut z_row = initial_reduced_costs(&tableau, &basis, &phase1_cost);
    let phase1_status = run_phase(
        &mut tableau,
        &mut rhs,
        &mut basis,
        &mut z_row,
        &phase1_cost,
        &excluded,
        tolerance,
        max_iterations,
        deadline,
    );

    if phase1_status == Status::Unbounded {
        // The sum of nonnegative artificial variables is bounded below by
        // zero, so Phase 1 can never legitimately report unbounded.
        return SimplexResult {
            status: Status::Numerical,
            tableau,
            rhs: Vector::from(rhs),
            basis,
            reduced_costs: z_row,
        };
    }
    if phase1_status == Status::IterationLimit {
        return SimplexResult {
            status: Status::IterationLimit,
            tableau,
            rhs: Vector::from(rhs),
            basis,
            reduced_costs: z_row,
        };
    }
    if phase1_status == Status::TimeLimit {
        return SimplexResult {
            status: Status::TimeLimit,
            tableau,
            rhs: Vector::from(rhs),
            basis,
            reduced_costs: z_row,
        };
    }

    let phase1_objective: f64 = basis
        .iter()
        .zip(rhs.iter())
        .map(|(&b, &v)| if is_artificial[b] { v } else { 0.0 })
        .sum();
    if phase1_objective > tolerance.max(1e-7) {
        return SimplexResult {
            status: Status::Infeasible,
            tableau,
            rhs: Vector::from(rhs),
            basis,
            reduced_costs: z_row,
        };
    }

    let mut is_basic = vec![false; form.num_cols];
    for &b in basis.iter() {
        is_basic[b] = true;
    }

    for r in 0..basis.len() {
        if !is_artificial[basis[r]] {
            continue;
        }

        let row = tableau.get_row(r);
        let replacement = (0..form.num_cols)
            .find(|&j| !is_artificial[j] && !is_basic[j] && row[j].abs() > tolerance);

        if let Some(q) = replacement {
            // Gauss-Jordan pivot on (r, q).
            let pivot = row[q];
            let pivot_row = &row * (1.0 / pivot);
            tableau
                .set_row(r, &pivot_row)
                .expect("pivot row has the tableau's own width");
            rhs[r] /= pivot;

            for rr in 0..tableau.rows() {
                if rr == r {
                    continue;
                }
                let factor = tableau.get_col(q)[rr];
                if factor != 0.0 {
                    let row_rr = tableau.get_row(rr);
                    let updated = &row_rr - &(&pivot_row * factor);
                    tableau
                        .set_row(rr, &updated)
                        .expect("row has the tableau's own width");
                    rhs[rr] -= factor * rhs[r];
                }
            }

            is_basic[basis[r]] = false;
            is_basic[q] = true;
            basis[r] = q;
        }
    }

    let mut z_row = initial_reduced_costs(&tableau, &basis, &form.objective);
    let phase2_status = run_phase(
        &mut tableau,
        &mut rhs,
        &mut basis,
        &mut z_row,
        &form.objective,
        &excluded,
        tolerance,
        max_iterations,
        deadline,
    );

    SimplexResult {
        status: phase2_status,
        tableau,
        rhs: Vector::from(rhs),
        basis,
        reduced_costs: z_row,
    }
}

/// Computes reduced costs from scratch: `cost - c_B^T * tableau`. Called
/// exactly twice per solve (once per phase, at the moment the active cost
/// vector changes).
fn initial_reduced_costs(tableau: &Matrix, basis: &[usize], cost: &Vector) -> Vector {
    let mut z = cost.clone();
    for (r, &b) in basis.iter().enumerate() {
        let c_br = cost[b];
        if c_br != 0.0 {
            let row = tableau.get_row(r) * c_br;
            z = &z - &row;
        }
    }
    z
}

/// Pivots `tableau`/`rhs`/`basis`/`z_row` in place until no column outside
/// `excluded` has a negative reduced cost (optimal), an improving column
/// has no bounded ratio test (unbounded), `max_iterations` is
/// exhausted, or `deadline` is exceeded.
fn run_phase(
    tableau: &mut Matrix,
    rhs: &mut [f64],
    basis: &mut [usize],
    z_row: &mut Vector,
    _cost: &Vector,
    excluded: &[bool],
    tolerance: f64,
    max_iterations: u32,
    deadline: Option<Instant>,
) -> Status {
    let num_rows = tableau.rows();
    let num_cols = tableau.cols();

    let mut is_basic = vec![false; num_cols];
    for &b in basis.iter() {
        is_basic[b] = true;
    }

    for _ in 0..max_iterations {
        // Check deadline at the start of each iteration.
        if let Some(dl) = deadline
            && Instant::now() >= dl
        {
            return Status::TimeLimit;
        }

        // Bland's rule: enter the smallest-index column with a negative
        // reduced cost under the current basis.
        let mut entering = None;
        for j in 0..num_cols {
            if excluded[j] || is_basic[j] {
                continue;
            }
            if z_row[j] < -tolerance {
                entering = Some(j);
                break;
            }
        }
        let Some(q) = entering else {
            return Status::Optimal;
        };

        let column = tableau.get_col(q);

        // Ratio test: the smallest-index basic variable among those tied
        // for the minimum ratio leaves.
        let mut leaving_row = None;
        let mut min_ratio = f64::INFINITY;
        for r in 0..num_rows {
            let a_rq = column[r];
            if a_rq > tolerance {
                let ratio = rhs[r] / a_rq;
                let better = ratio < min_ratio - tolerance;
                let tied_but_smaller_index = (ratio - min_ratio).abs() <= tolerance
                    && leaving_row.is_some_and(|p| basis[r] < basis[p]);
                if better || tied_but_smaller_index {
                    min_ratio = ratio.min(min_ratio);
                    leaving_row = Some(r);
                }
            }
        }
        let Some(p) = leaving_row else {
            return Status::Unbounded;
        };

        // Gauss-Jordan pivot on (p, q).
        let pivot = column[p];
        let pivot_row = tableau.get_row(p) * (1.0 / pivot);
        tableau
            .set_row(p, &pivot_row)
            .expect("pivot row has the tableau's own width");
        rhs[p] /= pivot;

        for r in 0..num_rows {
            if r == p {
                continue;
            }
            let factor = column[r];
            if factor != 0.0 {
                let row_r = tableau.get_row(r);
                let scaled_pivot_row = &pivot_row * factor;
                let updated_row = &row_r - &scaled_pivot_row;
                tableau
                    .set_row(r, &updated_row)
                    .expect("row has the tableau's own width");
                rhs[r] -= factor * rhs[p];
            }
        }

        let z_factor = z_row[q];
        if z_factor != 0.0 {
            let scaled_pivot_row = &pivot_row * z_factor;
            *z_row = &*z_row - &scaled_pivot_row;
        }

        is_basic[basis[p]] = false;
        is_basic[q] = true;
        basis[p] = q;
    }

    Status::IterationLimit
}
