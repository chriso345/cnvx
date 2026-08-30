use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::bounds::Bounds;
use crate::constraint::{Constraint, ConstraintKind};
use crate::env::{Env, Params};
use crate::error::CnvxError;
use crate::expr::Expression;
use crate::sense::Sense;
use crate::var::{Con, Var, VarKind};

/// Generation ids are handed out from a single process-wide counter, so
/// every `Model` gets a distinct one and a `Var`/`Con` created by one
/// `Model` can never collide with a handle from another.
static NEXT_GENERATION: AtomicU32 = AtomicU32::new(1);

fn next_generation() -> u32 {
    NEXT_GENERATION.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug)]
struct VarData {
    bounds: Bounds,
    kind: VarKind,
    name: Option<String>,
}

/// An optimization problem: variables, an objective, and constraints.
///
/// `Model` is pure problem *definition*. It knows nothing about how it will
/// be solved. `Model::clone` is cheap: the variable and constraint
/// lists are reference-counted and copied lazily (copy-on-write) the first
/// time a clone diverges from its source.
#[derive(Clone, Debug)]
pub struct Model {
    name: String,
    generation: u32,
    params: Params,
    vars: Rc<Vec<VarData>>,
    objective_sense: Sense,
    objective: Expression,
    constraints: Rc<Vec<Constraint>>,
}

impl Model {
    /// Creates an empty model with default solve parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::Model;
    /// let model = Model::new("diet_problem");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Model {
            name: name.into(),
            generation: next_generation(),
            params: Params::default(),
            vars: Rc::new(Vec::new()),
            objective_sense: Sense::Minimize,
            objective: Expression::zero(),
            constraints: Rc::new(Vec::new()),
        }
    }

    /// Creates an empty model whose default solve parameters come from
    /// `env`. Per-solve overrides on a solver builder still take precedence
    /// over these defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::{Env, Model, Params};
    /// let env = Env::with_params(Params { time_limit: Some(30.0), ..Default::default() });
    /// let model = Model::with_env(&env, "diet_problem");
    /// ```
    pub fn with_env(env: &Env, name: impl Into<String>) -> Self {
        Model { params: env.params.clone(), ..Model::new(name) }
    }

    /// The model's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The default solve parameters this model was created with, from
    /// [`Model::with_env`] (or [`Params::default`] if the model was created
    /// with [`Model::new`]).
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// Adds a continuous, unnamed variable and returns its handle.
    pub fn add_var(&mut self, bounds: impl Into<Bounds>) -> Var {
        self.add_kind_var(bounds, VarKind::Continuous, "")
    }

    /// Adds a continuous variable with a name, for reporting/debugging.
    pub fn add_named_var(&mut self, bounds: impl Into<Bounds>, name: &str) -> Var {
        self.add_kind_var(bounds, VarKind::Continuous, name)
    }

    /// Adds a variable of the given [`VarKind`] and name.
    pub fn add_kind_var(
        &mut self,
        bounds: impl Into<Bounds>,
        kind: VarKind,
        name: &str,
    ) -> Var {
        let index = self.vars.len() as u32;
        let name = if name.is_empty() { None } else { Some(name.to_string()) };
        Rc::make_mut(&mut self.vars).push(VarData { bounds: bounds.into(), kind, name });
        Var { index, generation: self.generation }
    }

    /// Adds an integer variable.
    pub fn add_integer(&mut self, bounds: impl Into<Bounds>, name: &str) -> Var {
        self.add_kind_var(bounds, VarKind::Integer, name)
    }

    /// Adds a binary variable (bounds fixed to `[0, 1]`).
    pub fn add_binary(&mut self, name: &str) -> Var {
        self.add_kind_var(Bounds::new(0.0, 1.0), VarKind::Binary, name)
    }

    /// Adds `n` continuous, unnamed variables sharing the same bounds.
    pub fn add_vars(&mut self, n: usize, bounds: impl Into<Bounds>) -> Vec<Var> {
        let bounds = bounds.into();
        (0..n).map(|_| self.add_var(bounds)).collect()
    }

    /// Adds `n` continuous variables sharing the same bounds, named
    /// `"{prefix}[0]"`, `"{prefix}[1]"`, ...
    pub fn add_named_vars(
        &mut self,
        n: usize,
        bounds: impl Into<Bounds>,
        prefix: &str,
    ) -> Vec<Var> {
        let bounds = bounds.into();
        (0..n)
            .map(|i| self.add_named_var(bounds, &format!("{prefix}[{i}]")))
            .collect()
    }

    /// Iterates over every variable handle in this model, in creation order.
    pub fn vars(&self) -> impl Iterator<Item = Var> + '_ {
        let generation = self.generation;
        (0..self.vars.len() as u32).map(move |index| Var { index, generation })
    }

    /// The number of variables in the model.
    pub fn num_vars(&self) -> usize {
        self.vars.len()
    }

    /// The bounds of `v`.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `v` came from a different
    /// `Model`.
    pub fn bounds(&self, v: Var) -> Result<Bounds, CnvxError> {
        self.resolve_var(v).map(|i| self.vars[i].bounds)
    }

    /// The [`VarKind`] of `v`.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `v` came from a different
    /// `Model`.
    pub fn kind(&self, v: Var) -> Result<VarKind, CnvxError> {
        self.resolve_var(v).map(|i| self.vars[i].kind)
    }

    /// The name of `v`, if one was given when it was created.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `v` came from a different
    /// `Model`.
    pub fn var_name(&self, v: Var) -> Result<Option<&str>, CnvxError> {
        let i = self.resolve_var(v)?;
        Ok(self.vars[i].name.as_deref())
    }

    fn resolve_var(&self, v: Var) -> Result<usize, CnvxError> {
        if v.generation != self.generation {
            return Err(CnvxError::ForeignHandle);
        }
        Ok(v.index as usize)
    }

    fn resolve_con(&self, c: Con) -> Result<usize, CnvxError> {
        if c.generation != self.generation {
            return Err(CnvxError::ForeignHandle);
        }
        Ok(c.index as usize)
    }

    /// Checks that every variable referenced by `expr` belongs to this
    /// model.
    fn check_expr(&self, expr: &Expression) -> Result<(), CnvxError> {
        for (v, _) in expr.terms() {
            self.resolve_var(v)?;
        }
        Ok(())
    }

    /// Sets the objective sense and expression, replacing any previous
    /// objective.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `expr` references a
    /// variable from a different `Model`.
    pub fn set_objective(
        &mut self,
        sense: Sense,
        expr: Expression,
    ) -> Result<(), CnvxError> {
        self.check_expr(&expr)?;
        self.objective_sense = sense;
        self.objective = expr;
        Ok(())
    }

    /// The objective's optimization direction.
    pub fn sense(&self) -> Sense {
        self.objective_sense
    }

    /// The objective expression.
    pub fn objective(&self) -> &Expression {
        &self.objective
    }

    /// Adds a constraint built from [`Expression::leq`], [`Expression::geq`],
    /// [`Expression::eq`], or [`Expression::between`], and returns its
    /// handle.
    ///
    /// # Errors
    /// - `Err(CnvxError::ForeignHandle)` if `constraint` references a variable
    ///   from a different `Model`.
    /// - `Err(CnvxError::InvalidBounds)` if `constraint` is a ranged constraint
    ///   (from [`Expression::between`]) with `lower > upper`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::Model;
    /// let mut model = Model::new("example");
    /// let x = model.add_var(0.0..);
    /// let y = model.add_var(0.0..=10.0);
    /// let c1 = model.add_constraint((x + y).geq(5.0))?;
    /// let c2 = model.add_constraint((2.0 * x - y).leq(12.0))?;
    /// # Ok::<(), cnvx_core::CnvxError>(())
    /// ```
    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<Con, CnvxError> {
        self.check_expr(constraint.expr())?;
        if let ConstraintKind::Ranged { lower, upper } = constraint.kind()
            && lower > upper
        {
            return Err(CnvxError::InvalidBounds { lower, upper });
        }
        let index = self.constraints.len() as u32;
        Rc::make_mut(&mut self.constraints).push(constraint);
        Ok(Con { index, generation: self.generation })
    }

    /// Equivalent to `model.add_constraint(constraint.named(name))`.
    pub fn add_named_constraint(
        &mut self,
        constraint: Constraint,
        name: &str,
    ) -> Result<Con, CnvxError> {
        self.add_constraint(constraint.named(name))
    }

    /// Sets (or replaces) the name of an existing constraint, for
    /// reporting/debugging.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `con` came from a
    /// different `Model`.
    pub fn set_name(
        &mut self,
        con: Con,
        name: impl Into<String>,
    ) -> Result<(), CnvxError> {
        let i = self.resolve_con(con)?;
        let renamed = self.constraints[i].clone().named(name.into());
        Rc::make_mut(&mut self.constraints)[i] = renamed;
        Ok(())
    }

    /// Iterates over every constraint handle in this model, in creation
    /// order.
    pub fn cons(&self) -> impl Iterator<Item = Con> + '_ {
        let generation = self.generation;
        (0..self.constraints.len() as u32).map(move |index| Con { index, generation })
    }

    /// The number of constraints in the model.
    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }

    /// The constraint referenced by `con`.
    ///
    /// # Errors
    /// Returns `Err(CnvxError::ForeignHandle)` if `con` came from a
    /// different `Model`.
    pub fn constraint(&self, con: Con) -> Result<&Constraint, CnvxError> {
        let i = self.resolve_con(con)?;
        Ok(&self.constraints[i])
    }

    /// Iterates over every `(handle, constraint)` pair, in creation order.
    pub fn constraints(&self) -> impl Iterator<Item = (Con, &Constraint)> + '_ {
        let generation = self.generation;
        self.constraints
            .iter()
            .enumerate()
            .map(move |(i, c)| (Con { index: i as u32, generation }, c))
    }
}
