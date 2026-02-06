use crate::*;
use std::ops::AddAssign;

/// Represents an optimization model containing variables, constraints, and an objective.
///
/// The [`Model`] struct is the central container for defining a linear (or eventually more general)
/// optimization problem. Users add variables, set an objective function, and add constraints.
/// Solvers then operate on a [`Model`] to produce a [`Solution`].
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::*;
/// let mut model = Model::new();
///
/// // Add variables
/// let x1 = model.add_var().finish();
/// let x2 = model.add_var().finish();
///
/// // Define objective: maximize x1 + 2*x2
/// model.add_objective(Objective::maximize(x1 + 2.0 * x2).name("Z"));
///
/// // Add constraints
/// model += (x1 + x2).leq(10.0);
/// model += x1.geq(0.0);
/// model += x2.geq(0.0);
/// ```
#[derive(Debug, Default)]
pub struct Model {
    /// List of variables in the model.
    pub vars: Vec<Var>,

    /// List of constraints in the model.
    pub constraints: Vec<Constraint>,

    /// Optional objective function. Currently supports only a single objective.
    /// TODO: Replace with `Vec<Objective>` for multi-objective optimization.
    pub objective: Option<Objective>,

    /// Enables logging during model construction or solving.
    pub logging: bool,
}

impl Model {
    /// Creates a new model with logging enabled by default.
    pub fn new() -> Self {
        Self { logging: true, ..Default::default() }
    }

    /// Adds a new variable to the model and returns a [`VarBuilder`] for ergonomic configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::*;
    /// let mut model = Model::new();
    /// let x = model.add_var().integer().finish();
    /// ```
    pub fn add_var(&mut self) -> VarBuilder<'_> {
        let id = VarId(self.vars.len());
        self.vars.push(Var {
            id,
            lb: Some(0.0),
            ub: None,
            is_integer: false,
            is_artificial: false,
        });
        VarBuilder { model: self, var: id }
    }

    /// Sets the objective function of the model.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::*;
    /// let mut model = Model::new();
    /// let x = model.add_var().finish();
    /// model.add_objective(Objective::maximize(1.0 * x).name("Profit"));
    /// ```
    pub fn add_objective(&mut self, obj: Objective) {
        self.objective = Some(obj);
    }

    /// Returns a read-only slice of variables.
    pub fn vars(&self) -> &[Var] {
        &self.vars
    }

    /// Returns a read-only slice of constraints.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Returns a reference to the model's objective function, if set.
    pub fn objective(&self) -> Option<&Objective> {
        self.objective.as_ref()
    }
}

/// Allows adding constraints to the model using the `+=` operator.
///
/// # Example
///
/// ```rust
/// # use cnvx_core::*;
/// let mut model = Model::new();
/// let x = model.add_var().finish();
/// model += x.geq(0.0);
/// ```
impl AddAssign<Constraint> for Model {
    fn add_assign(&mut self, rhs: Constraint) {
        self.constraints.push(rhs);
    }
}
