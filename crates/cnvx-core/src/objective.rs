use crate::LinExpr;
use std::fmt::Display;

/// The optimization direction of an objective function.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sense {
    /// Minimize the objective function.
    Minimize,

    /// Maximize the objective function.
    Maximize,
}

/// Represents an objective function in a model.
///
/// Contains the linear expression to optimize, the optimization sense (min/max),
/// an optional name, and an optional priority (useful for multi-objective problems).
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Objective, LinExpr, Model};
/// # let mut model = Model::new();
///
/// let x = model.add_var().finish(); // VarId
/// let objective = Objective::maximize(3.0 * x).name("Profit");
/// ```
#[derive(Clone, Debug)]
pub struct Objective {
    /// Whether to minimize or maximize the objective.
    pub sense: Sense,

    /// The linear expression representing the objective.
    pub expr: LinExpr,

    /// Optional human-readable name.
    pub name: Option<String>,

    /// Optional priority for multi-objective optimization (not used)
    pub priority: Option<u32>,
}

/// Builder for ergonomic creation of objectives.
///
/// Returned by [`Objective::minimize`] or [`Objective::maximize`],
/// allows setting optional fields like `name` or `priority`.
pub struct ObjectiveBuilder {
    objective: Objective,
}

impl ObjectiveBuilder {
    /// Sets a priority for multi-objective optimization.
    ///
    /// Higher priority objectives are optimized first.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # use cnvx_core::{Objective, LinExpr, Model};
    /// # let mut model = Model::new();
    /// let x = model.add_var().finish(); // VarId
    /// let obj = Objective::maximize(2.0 * x).priority(1);
    /// ```
    pub fn priority(mut self, p: u32) -> Self {
        self.objective.priority = Some(p);
        unimplemented!(
            "Multi-objective optimization is not yet implemented, so priority has no effect"
        );
    }

    /// Sets a human-readable name for the objective and returns the final [`Objective`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::{Objective, LinExpr, Model};
    /// # let mut model = Model::new();
    /// let x = model.add_var().finish(); // VarId
    /// let obj = Objective::maximize(2.0 * x).name("Profit");
    /// ```
    pub fn name<S: Into<String>>(mut self, name: S) -> Objective {
        self.objective.name = Some(name.into());
        self.objective
    }
}

impl Objective {
    /// Creates a minimization objective.
    ///
    /// Returns an [`ObjectiveBuilder`] to optionally set name or priority.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::{Objective, LinExpr, Model};
    /// # let mut model = Model::new();
    /// let x = model.add_var().finish(); // VarId
    /// let obj = Objective::minimize(3.0 * x).name("Cost");
    /// ```
    pub fn minimize(expr: LinExpr) -> ObjectiveBuilder {
        ObjectiveBuilder {
            objective: Objective {
                sense: Sense::Minimize,
                expr,
                name: None,
                priority: None,
            },
        }
    }

    /// Creates a maximization objective.
    ///
    /// Returns an [`ObjectiveBuilder`] to optionally set name or priority.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::{Objective, LinExpr, Model};
    /// # let mut model = Model::new();
    /// let x = model.add_var().finish(); // VarId
    /// let obj = Objective::maximize(3.0 * x).name("Profit");
    /// ```
    pub fn maximize(expr: LinExpr) -> ObjectiveBuilder {
        ObjectiveBuilder {
            objective: Objective {
                sense: Sense::Maximize,
                expr,
                name: None,
                priority: None,
            },
        }
    }
}

impl Display for Objective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sense_str = match self.sense {
            Sense::Minimize => "Minimize",
            Sense::Maximize => "Maximize",
        };
        if let Some(name) = &self.name {
            write!(f, "{} {}: {}", sense_str, name, self.expr)
        } else {
            write!(f, "{}: {}", sense_str, self.expr)
        }
    }
}
