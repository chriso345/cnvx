use crate::{Constraint, expr::LinExpr};
use std::ops::Mul;

/// A unique identifier for a variable in a model.
///
/// This is used internally by the solver and the model to index variable values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

impl VarId {
    /// Creates a `<=` constraint: `self <= rhs`.
    pub fn leq(self, rhs: f64) -> Constraint {
        LinExpr::from(self).leq(rhs)
    }

    /// Creates a `>=` constraint: `self >= rhs`.
    pub fn geq(self, rhs: f64) -> Constraint {
        LinExpr::from(self).geq(rhs)
    }

    /// Creates a `==` constraint: `self == rhs`.
    pub fn eq(self, rhs: f64) -> Constraint {
        LinExpr::from(self).eq(rhs)
    }
}

/// Represents a decision variable in a model.
///
/// Contains information about optional bounds, whether the variable is integer,
/// and whether it is an artificial variable used in simplex initialization.
#[derive(Clone, Debug)]
pub struct Var {
    /// Unique identifier for the variable.
    pub id: VarId,

    /// Optional lower bound.
    pub lb: Option<f64>,

    /// Optional upper bound.
    pub ub: Option<f64>,

    /// Whether the variable is restricted to integer values.
    pub is_integer: bool,

    /// Whether this is an artificial variable (used for inequality constraints in simplex initialization).
    pub is_artificial: bool,
}

/// A builder for setting properties of a variable using a fluent API.
///
/// Returned by [`Model::add_var()`](crate::model::Model::add_var), this allows setting bounds, integrality,
/// and other properties before finalizing the variable with [`finish()`](VarBuilder::finish).
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Model};
/// let mut model = Model::new();
/// let x = model
///     .add_var()
///     .integer()
///     .finish();
/// ```
pub struct VarBuilder<'a> {
    pub model: &'a mut crate::Model,
    pub var: VarId,
}

/// Methods for configuring a variable using a fluent API.
///
/// Returned by [`Model::add_var()`](crate::model::Model::add_var). Use these methods to set bounds,
/// integrality, or mark a variable as binary before calling [`finish()`](VarBuilder::finish).
impl<'a> VarBuilder<'a> {
    /// Sets a lower bound for the variable.
    ///
    /// This method currently panics because lower bounds are not yet implemented.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use cnvx_core::{Model};
    /// let mut model = Model::new();
    /// let x = model.add_var().lower_bound(0.0).finish();
    /// ```
    pub fn lower_bound(self, lb: f64) -> Self {
        _ = lb;
        panic!("Lower bound not implemented yet");
        // self.model.vars[self.var.0].lb = Some(lb);
        // self
    }

    /// Sets an upper bound for the variable.
    ///
    /// This method currently panics because upper bounds are not yet implemented.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use cnvx_core::{Model};
    /// let mut model = Model::new();
    /// let x = model.add_var().upper_bound(10.0).finish();
    /// ```
    pub fn upper_bound(self, ub: f64) -> Self {
        _ = ub;
        panic!("Upper bound not implemented yet");
        // self.model.vars[self.var.0].ub = Some(ub);
        // self
    }

    /// Mark the variable as an integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::{Model};
    /// let mut model = Model::new();
    /// let x = model.add_var().integer().finish();
    /// ```
    pub fn integer(self) -> Self {
        self.model.vars[self.var.0].is_integer = true;
        self
    }

    /// Mark the variable as binary (0 or 1).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::{Model};
    /// let mut model = Model::new();
    /// let x = model.add_var().binary().finish();
    /// ```
    pub fn binary(self) -> Self {
        let var = &mut self.model.vars[self.var.0];
        var.is_integer = true;
        var.lb = Some(0.0);
        var.ub = Some(1.0);
        self
    }

    /// Finalizes the variable and returns its [`VarId`].
    ///
    /// Must be called after setting any desired properties on the variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::{Model};
    /// let mut model = Model::new();
    /// let x = model.add_var().integer().finish();
    /// ```
    pub fn finish(self) -> VarId {
        self.var
    }
}

/// Allows multiplying a variable by a constant to create a linear expression.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Model};
/// let mut model = Model::new();
/// let x = model.add_var().finish();
/// let expr = x * 3.0; // LinExpr representing 3*x
/// ```
impl Mul<f64> for VarId {
    type Output = LinExpr;

    fn mul(self, rhs: f64) -> LinExpr {
        LinExpr::new(self, rhs)
    }
}

/// Allows multiplying a constant by a variable to create a linear expression.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Model};
/// let mut model = Model::new();
/// let x = model.add_var().finish();
/// let expr = 3.0 * x; // LinExpr representing 3*x
/// ```
impl Mul<VarId> for f64 {
    type Output = LinExpr;

    fn mul(self, rhs: VarId) -> LinExpr {
        rhs * self
    }
}
