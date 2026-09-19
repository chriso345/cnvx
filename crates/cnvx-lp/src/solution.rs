use std::ops::Index;

use cnvx_core::{Con, Status, Var};

use crate::Basis;

/// The valid range a variable's objective coefficient can move within
/// without changing the optimal basis (classic LP cost-coefficient
/// ranging), from [`LpSolution::sensitivity`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CostRange {
    /// The smallest value the coefficient can take while the current basis
    /// stays optimal.
    pub lower: f64,
    /// The largest value the coefficient can take while the current basis
    /// stays optimal.
    pub upper: f64,
}

/// The valid range a constraint's right-hand side can move within without
/// changing the optimal basis (classic LP RHS ranging), from
/// [`LpSolution::rhs_range`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RhsRange {
    /// The smallest right-hand-side value for which the current basis
    /// stays feasible.
    pub lower: f64,
    /// The largest right-hand-side value for which the current basis stays
    /// feasible.
    pub upper: f64,
}

/// The result of a successful LP solve.
///
/// "Successful" here means the *solve call* completed; `status` may still
/// be [`Status::Infeasible`] or [`Status::Unbounded`] -- exactly as
/// described in [`cnvx_core::CnvxError`], those are well-defined outcomes,
/// not errors.
#[derive(Clone, Debug)]
pub struct LpSolution {
    /// The outcome of the solve.
    pub status: Status,
    /// The objective value, if `status` is [`Status::Optimal`]. Other
    /// terminal statuses may not have a meaningful objective value.
    pub objective: f64,
    pub(crate) values: Vec<(Var, f64)>,
    pub(crate) duals: Vec<(Con, f64)>,
    pub(crate) reduced_costs: Vec<(Var, f64)>,
    pub(crate) cost_ranges: Vec<(Var, CostRange)>,
    pub(crate) rhs_ranges: Vec<(Con, RhsRange)>,
    pub(crate) basis: Basis,
}

impl LpSolution {
    /// The value of `v` in this solution.
    ///
    /// Returns `0.0` if `v` isn't part of the solved model (e.g. a foreign
    /// handle).
    pub fn value(&self, v: Var) -> f64 {
        self.values
            .iter()
            .find(|(var, _)| *var == v)
            .map(|(_, val)| *val)
            .unwrap_or(0.0)
    }

    /// Iterates over every variable's value in this solution.
    pub fn values(&self) -> impl Iterator<Item = (Var, f64)> + '_ {
        self.values.iter().copied()
    }

    /// Shadow price (dual value) of a constraint at the optimum: how much
    /// the objective would improve per unit relaxation of the constraint's
    /// right-hand side.
    pub fn dual(&self, c: Con) -> f64 {
        self.duals
            .iter()
            .find(|(con, _)| *con == c)
            .map(|(_, val)| *val)
            .unwrap_or(0.0)
    }

    /// Shadow price (dual value) of a constraint at the optimum: how much
    /// the objective would improve per unit relaxation of the constraint's
    /// right-hand side.
    pub fn shadow(&self, c: Con) -> f64 {
        self.dual(c)
    }

    /// Reduced cost of a variable at the optimum: how much the objective
    /// would change per unit relaxation of the variable's binding bound.
    pub fn reduced_cost(&self, v: Var) -> f64 {
        self.reduced_costs
            .iter()
            .find(|(var, _)| *var == v)
            .map(|(_, val)| *val)
            .unwrap_or(0.0)
    }

    /// The range `v`'s objective coefficient could move within without
    /// changing the optimal basis.
    pub fn sensitivity(&self, v: Var) -> CostRange {
        self.cost_ranges
            .iter()
            .find(|(var, _)| *var == v)
            .map(|(_, range)| *range)
            .unwrap_or(CostRange { lower: f64::NEG_INFINITY, upper: f64::INFINITY })
    }

    /// The range `c`'s right-hand side could move within without changing
    /// the optimal basis.
    pub fn rhs_range(&self, c: Con) -> RhsRange {
        self.rhs_ranges
            .iter()
            .find(|(con, _)| *con == c)
            .map(|(_, range)| *range)
            .unwrap_or(RhsRange { lower: f64::NEG_INFINITY, upper: f64::INFINITY })
    }

    /// The optimal basis, for [`crate::LpSolver::warm_start`] on a
    /// subsequent, related solve.
    pub fn basis(&self) -> &Basis {
        &self.basis
    }
}

/// Indexing into an [`LpSolution`] with a [`Var`] returns the variable's
/// value in the solution, or panics if the variable isn't part of the solved
/// model. Use [`LpSolution::value`] if that's a possibility.
///
/// # Panics
/// Panics if `v` isn't part of the solved model. Prefer [`LpSolution::value`]
/// if that's a possibility.
impl Index<Var> for LpSolution {
    type Output = f64;

    fn index(&self, v: Var) -> &f64 {
        self.values
            .iter()
            .find(|(var, _)| *var == v)
            .map(|(_, val)| val)
            .expect("LpSolution: variable is not part of the solved model")
    }
}
