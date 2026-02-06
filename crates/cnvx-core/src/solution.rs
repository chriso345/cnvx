use crate::{SolveStatus, VarId};
use std::fmt::Display;

/// Represents the result of solving an optimization problem.
///
/// Contains the values assigned to each variable, the value of the objective function,
/// and the solver status.
///
/// # Examples
///
/// ```rust
/// use cnvx_core::{Solution, SolveStatus, VarId};
///
/// // Example solution with 3 variables
/// let solution = Solution {
///     values: vec![1.0, 2.0, 3.0],
///     objective_value: Some(10.0),
///     status: SolveStatus::Optimal,
/// };
///
/// assert_eq!(solution.value(VarId(0)), 1.0);
/// assert_eq!(solution.value(VarId(2)), 3.0);
/// ```
#[derive(Debug)]
pub struct Solution {
    /// Variable assignments, indexed by variable ID.
    ///
    /// The value at index `i` corresponds to the variable with ID [`VarId(i)`](VarId).
    pub values: Vec<f64>,

    /// The value of the objective function at the solution.
    ///
    /// [`None`] if the solver did not produce an objective value (e.g., infeasible or unbounded problem).
    pub objective_value: Option<f64>,

    /// The solver status indicating whether the solution is optimal, feasible, infeasible, or unbounded.
    pub status: SolveStatus,
}

impl Solution {
    /// Returns the value assigned to the variable with the given [`VarId`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::{Model, Solution, SolveStatus, VarId};
    /// # let mut model = Model::new();
    /// let x1: VarId = model.add_var().finish();
    /// let solution = Solution {
    ///     values: vec![1.0], // Assuming x1 has ID 0
    ///     objective_value: Some(10.0),
    ///     status: SolveStatus::Optimal,
    /// };
    /// let value = solution.value(x1);
    /// ```
    pub fn value(&self, var: VarId) -> f64 {
        self.values[var.0]
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(obj_val) = self.objective_value {
            write!(f, "{}: {}", self.status, obj_val)?;
        } else {
            write!(f, "{}", self.status)?;
        }
        Ok(())
    }
}
