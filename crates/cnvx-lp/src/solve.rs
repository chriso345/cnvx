use cnvx_core::{CnvxError, Model, Solve, Status};

use crate::basis::{Basis, VarStatus};
use crate::method::LpMethod;
use crate::simplex::{self, SimplexResult};
use crate::solution::{CostRange, LpSolution, RhsRange};
use crate::solver::LpSolver;
use crate::standard_form::{self, Column, StandardForm};

impl Solve<LpSolver> for Model {
    type Solution = LpSolution;

    fn solve(&self, solver: &LpSolver) -> Result<LpSolution, CnvxError> {
        // TODO: split into separate methods for each solver type.
        match solver.method() {
            LpMethod::DualSimplex => {
                return Err(CnvxError::Numerical(
                    "LpMethod::DualSimplex is not implemented yet; use LpMethod::PrimalSimplex".to_string(),
                ));
            }
            LpMethod::InteriorPoint => {
                return Err(CnvxError::Numerical(
                    "LpMethod::InteriorPoint is not implemented yet; use LpMethod::PrimalSimplex".to_string(),
                ));
            }
            LpMethod::PrimalSimplex => {}
        }

        let form = standard_form::build(self)?;
        let result = simplex::solve(
            &form,
            solver.tolerance,
            solver.max_iterations,
            solver.time_limit,
        );

        if result.status != Status::Optimal {
            return Ok(LpSolution {
                status: result.status,
                objective: f64::NAN,
                values: Vec::new(),
                duals: Vec::new(),
                reduced_costs: Vec::new(),
                cost_ranges: Vec::new(),
                rhs_ranges: Vec::new(),
                basis: Basis::new(Vec::new()),
            });
        }

        let mut y = vec![0.0; form.num_cols];
        for (r, &b) in result.basis.iter().enumerate() {
            y[b] = result.rhs[r];
        }

        let internal_objective: f64 =
            form.objective.iter().zip(&y).map(|(c, v)| c * v).sum::<f64>()
                + form.objective_constant;
        let objective = form.sense_sign * internal_objective;

        let reduced = &result.reduced_costs;

        let mut values = Vec::with_capacity(self.num_vars());
        let mut reduced_costs = Vec::with_capacity(self.num_vars());
        let mut cost_ranges = Vec::with_capacity(self.num_vars());
        let mut statuses = Vec::with_capacity(self.num_vars());
        for v in self.vars() {
            let column = form.var_columns[&v];
            let (value, reduced_cost, var_status) = match column {
                Column::Fixed(val) => (val, 0.0, VarStatus::AtLower),
                Column::Single { shift, sign, col } => {
                    let is_basic = result.basis.contains(&col);
                    let value = shift + sign * y[col];
                    let reduced_cost = form.sense_sign * reduced[col] * sign;
                    let status =
                        if is_basic { VarStatus::Basic } else { VarStatus::AtLower };
                    (value, reduced_cost, status)
                }
                Column::Split { pos, neg } => {
                    let is_basic =
                        result.basis.contains(&pos) || result.basis.contains(&neg);
                    let status =
                        if is_basic { VarStatus::Basic } else { VarStatus::AtLower };
                    (y[pos] - y[neg], 0.0, status)
                }
            };
            values.push((v, value));
            reduced_costs.push((v, reduced_cost));
            cost_ranges.push((v, cost_range_for(&form, &result, reduced, column)));
            statuses.push((v, var_status));
        }

        let mut duals = Vec::with_capacity(form.num_primary_rows);
        let mut rhs_ranges = Vec::with_capacity(form.num_primary_rows);
        for (i, c) in self.cons().enumerate() {
            let artificial = form.artificial_cols[i];
            let dual = form.sense_sign * form.row_sign[i] * -reduced[artificial];
            duals.push((c, dual));
            rhs_ranges.push((c, rhs_range_for(&form, &result, i)));
        }

        Ok(LpSolution {
            status: result.status,
            objective,
            values,
            duals,
            reduced_costs,
            cost_ranges,
            rhs_ranges,
            basis: Basis::new(statuses),
        })
    }
}

/// The range `column`'s (always-minimize, internal) objective coefficient
/// could move within while keeping the current basis optimal, converted
/// back to the original variable's coefficient and sense.
fn cost_range_for(
    form: &StandardForm,
    result: &SimplexResult,
    reduced: &[f64],
    column: Column,
) -> CostRange {
    let Column::Single { sign, col, .. } = column else {
        return CostRange { lower: f64::NEG_INFINITY, upper: f64::INFINITY };
    };

    let (delta_lower, delta_upper) = match result.basis.iter().position(|&b| b == col) {
        None => (-reduced[col], f64::INFINITY),
        Some(r) => {
            let mut delta_upper = f64::INFINITY;
            let mut delta_lower = f64::NEG_INFINITY;
            for (j, &a) in result.tableau.get_row(r).iter().enumerate() {
                if result.basis.contains(&j) {
                    continue;
                }
                if a > 1e-9 {
                    delta_upper = delta_upper.min(reduced[j] / a);
                } else if a < -1e-9 {
                    delta_lower = delta_lower.max(reduced[j] / a);
                }
            }
            (delta_lower, delta_upper)
        }
    };

    let factor = form.sense_sign * sign;
    let a = factor * (form.objective[col] + delta_lower);
    let b = factor * (form.objective[col] + delta_upper);
    CostRange { lower: a.min(b), upper: a.max(b) }
}

/// The range constraint row `row_index`'s right-hand side could move
/// within while keeping the current basis feasible, using the fact that a
/// row's artificial column, having started as the unit vector for that
/// row, still holds that row's column of `B^{-1}` in the final tableau.
fn rhs_range_for(
    form: &StandardForm,
    result: &SimplexResult,
    row_index: usize,
) -> RhsRange {
    let artificial = form.artificial_cols[row_index];
    let mut delta_upper = f64::INFINITY;
    let mut delta_lower = f64::NEG_INFINITY;
    for r in 0..result.tableau.rows() {
        let binv = result.tableau.get(r, artificial);
        let basic_value = result.rhs[r];
        if binv < -1e-9 {
            delta_upper = delta_upper.min(basic_value / -binv);
        } else if binv > 1e-9 {
            delta_lower = delta_lower.max(-basic_value / binv);
        }
    }

    let internal_rhs = form.rhs[row_index];
    let a = form.row_sign[row_index] * (internal_rhs + delta_lower);
    let b = form.row_sign[row_index] * (internal_rhs + delta_upper);
    RhsRange { lower: a.min(b), upper: a.max(b) }
}
