// TODO: Move to `cnvx-lp`

use std::any::Any;
use std::ops::AddAssign;

use crate::*;

/// Represents an extensible optimization model containing variables, constraints,
/// and an objective function.
///
/// `Model` is the concrete problem type for LP and MIP problems (both report
/// `kind() == "lp"` until a dedicated MIP problem type is introduced in
/// `cnvx-lp`).  It implements [`Problem`] so it can be passed directly to any
/// [`Solver`](crate::solver::Solver) without wrapping.
///
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::*;
/// let mut model = Model::new();
///
/// let x1 = model.add_var().finish();
/// let x2 = model.add_var().finish();
///
/// model.add_objective(Objective::maximize(x1 + 2.0 * x2).name("Z"));
/// model += (x1 + x2).leq(10.0);
/// model += x1.geq(0.0);
/// model += x2.geq(0.0);
///
/// use cnvx_core::problem::Problem;
/// assert_eq!(model.kind(), "lp");
/// assert_eq!(model.num_vars(), 2);
/// assert_eq!(model.num_constraints(), 3);
/// assert!(model.has_objective());
/// ```
#[derive(Debug, Default, Clone)]
pub struct Model {
    /// List of variables in the model.
    pub vars: Vec<Var>,

    /// List of constraints in the model.
    pub constraints: Vec<Constraint>,

    /// Optional objective function.
    ///
    /// Currently supports only a single objective.
    /// TODO: Replace with `Vec<Objective>` for multi-objective optimization.
    pub objective: Option<Objective>,
}

impl Model {
    /// Creates a new, empty model.
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Returns the shape of the constraint matrix as `(rows, cols)`,
    /// i.e. `(num_constraints, num_vars)`.
    pub fn shape(&self) -> (usize, usize) {
        (self.constraints.len(), self.vars.len())
    }

    /// Adds a new variable to the model and returns a [`VarBuilder`] for
    /// ergonomic configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::*;
    /// let mut model = Model::new();
    /// let x = model.add_var().integer().finish();
    /// ```
    pub fn add_var(&mut self) -> VarBuilder<'_> {
        let id = VarId(self.vars.len()); // FIXME: This should be a global counter.
        self.vars.push(Var {
            id,
            name: None,
            lb: Some(0.0),
            ub: None,
            is_integer: false,
            is_artificial: false,
        });
        VarBuilder { model: self, var: id }
    }

    /// Sets the objective function of the model, replacing any existing one.
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

    /// Returns a read-only slice of all variables.
    pub fn vars(&self) -> &[Var] {
        &self.vars
    }

    /// Returns a read-only slice of all constraints.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Returns a reference to the model's objective function, if one is set.
    pub fn objective(&self) -> Option<&Objective> {
        self.objective.as_ref()
    }
}

impl Problem for Model {
    /// Returns `"lp"` - the canonical kind tag for linear programs.
    ///
    /// Once a dedicated `MipModel` type is introduced in `cnvx-lp`, integer
    /// problems should be wrapped in that type and return `"mip"` instead.
    fn kind(&self) -> crate::problem::ProblemKind {
        "lp"
    }

    fn num_vars(&self) -> usize {
        self.vars.len()
    }

    fn num_constraints(&self) -> usize {
        self.constraints.len()
    }

    fn has_objective(&self) -> bool {
        self.objective.is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn describe(&self) -> String {
        let obj_name = self
            .objective
            .as_ref()
            .and_then(|o| o.name.as_deref())
            .unwrap_or("<unnamed>");
        format!(
            "LP model \"{}\" ({} vars, {} constraints)",
            obj_name,
            self.vars.len(),
            self.constraints.len(),
        )
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
