use std::ops::Index;

use cnvx_core::{Status, Var};

/// The result of a solve.
///
/// Unlike [`cnvx_lp::LpSolution`], there's no `dual`/`reduced_cost`/
/// `sensitivity` here: once integer restrictions are enforced, those
/// LP-duality concepts don't have a single well-defined meaning for the
/// original problem.
#[derive(Clone, Debug)]
pub struct MilpSolution {
    /// The outcome of the solve.
    pub status: Status,
    /// The objective value, if `status` is [`Status::Optimal`].
    pub objective: f64,
    /// A valid bound on the true optimum, as a result of
    /// a mip gap or node limit.
    pub bound: f64,
    /// The number of branch-and-bound nodes explored.
    pub nodes_explored: u64,
    pub(crate) values: Vec<(Var, f64)>,
}

impl MilpSolution {
    /// The value of `v` in this solution.
    ///
    /// Returns `0.0` if `v` isn't part of the solved model (e.g. a
    /// foreign handle) or if `status` isn't [`Status::Optimal`].
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

    /// The relative gap between `objective` and `bound`:
    /// $|objective - bound| / max(|objective|, 1.0)$.
    ///
    /// Returns `f64::INFINITY` if either isn't finite (e.g. no feasible
    /// solution was found).
    pub fn gap(&self) -> f64 {
        if !self.objective.is_finite() || !self.bound.is_finite() {
            return f64::INFINITY;
        }
        (self.objective - self.bound).abs() / self.objective.abs().max(1.0)
    }
}

/// `solution[x]` instead of `solution.value(x)`.
///
/// # Panics
/// Panics if `v` isn't part of the solved model. Prefer
/// [`MilpSolution::value`] if that's a possibility.
impl Index<Var> for MilpSolution {
    type Output = f64;

    fn index(&self, v: Var) -> &f64 {
        self.values
            .iter()
            .find(|(var, _)| *var == v)
            .map(|(_, val)| val)
            .expect("MilpSolution: variable is not part of the solved model")
    }
}
