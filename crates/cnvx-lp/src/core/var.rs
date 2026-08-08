//! Variable types and builder API for optimization models.

use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::LinearConstraint;
use crate::expr::LinExpr;

/// A unique identifier for a variable in a model.
///
/// This is used internally by the solver and the model to index variable
/// values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

impl VarId {
    /// Creates a `<=` constraint: `self <= rhs`.
    pub fn leq<T: Into<LinExpr>>(self, rhs: T) -> LinearConstraint {
        (LinExpr::from(self) - rhs.into()).leq(0.0)
    }

    /// Creates a `>=` constraint: `self >= rhs`.
    pub fn geq<T: Into<LinExpr>>(self, rhs: T) -> LinearConstraint {
        (LinExpr::from(self) - rhs.into()).geq(0.0)
    }

    /// Creates a `==` constraint: `self == rhs`.
    pub fn eq<T: Into<LinExpr>>(self, rhs: T) -> LinearConstraint {
        (LinExpr::from(self) - rhs.into()).eq(0.0)
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

    /// Optional name for the variable. Currently unused.
    pub name: Option<String>,

    /// Optional lower bound.
    pub lb: Option<f64>,

    /// Optional upper bound.
    pub ub: Option<f64>,

    /// Whether the variable is restricted to integer values.
    pub is_integer: bool,

    /// Whether this is an artificial variable (used for inequality constraints
    /// in simplex initialization).
    pub is_artificial: bool,
}

/// A builder for setting properties of a variable using a fluent API.
///
/// Returned by [`LpModel::add_var()`](crate::model::LpModel::add_var), this
/// allows setting bounds, integrality, and other properties before finalizing
/// the variable with [`finish()`](VarBuilder::finish).
///
/// # Examples
///
/// ```rust
/// # use cnvx_lp::LpModel;
/// let mut model = LpModel::new();
/// let x = model.add_var().integer().finish();
/// ```
pub struct VarBuilder<'a> {
    pub model: &'a mut crate::LpModel,
    pub var: VarId,
}

/// Methods for configuring a variable using a fluent API.
///
/// Returned by [`LpModel::add_var()`](crate::model::LpModel::add_var). Use
/// these methods to set bounds, integrality, or mark a variable as binary
/// before calling [`finish()`](VarBuilder::finish).
impl<'a> VarBuilder<'a> {
    /// Sets a name for the variable.
    ///
    /// ```rust, no_run
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
    /// let x = model.add_var().name("x").finish();
    /// ```
    pub fn name(self, name: &str) -> Self {
        self.model.vars[self.var.0].name = Some(name.to_string());
        self
    }

    /// Sets a lower bound for the variable.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
    /// let x = model.add_var().lower_bound(0.0).finish();
    /// ```
    pub fn lower_bound(self, lb: f64) -> Self {
        self.model.vars[self.var.0].lb = Some(lb);
        self
    }

    /// Sets an upper bound for the variable.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
    /// let x = model.add_var().upper_bound(10.0).finish();
    /// ```
    pub fn upper_bound(self, ub: f64) -> Self {
        self.model.vars[self.var.0].ub = Some(ub);
        self
    }

    /// Mark the variable as an integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
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
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
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
    /// # use cnvx_lp::LpModel;
    /// let mut model = LpModel::new();
    /// let x = model.add_var().integer().finish();
    /// ```
    pub fn finish(self) -> VarId {
        self.var
    }
}

/////////////////////////////////////////////////////////////////////////////
// Operator Overloads for VarId
/////////////////////////////////////////////////////////////////////////////

/// -VarId
impl Neg for VarId {
    type Output = LinExpr;

    fn neg(self) -> LinExpr {
        LinExpr::new(self, -1.0)
    }
}

/// VarId + f64
impl Add<f64> for VarId {
    type Output = LinExpr;

    fn add(self, rhs: f64) -> LinExpr {
        LinExpr {
            terms: vec![crate::LinTerm { var: self, coeff: 1.0 }],
            constant: rhs,
        }
    }
}

/// f64 + VarId
impl Add<VarId> for f64 {
    type Output = LinExpr;

    fn add(self, rhs: VarId) -> LinExpr {
        rhs + self
    }
}

/// VarId - VarId
impl Sub for VarId {
    type Output = LinExpr;

    fn sub(self, rhs: VarId) -> LinExpr {
        LinExpr {
            terms: vec![
                crate::LinTerm { var: self, coeff: 1.0 },
                crate::LinTerm { var: rhs, coeff: -1.0 },
            ],
            constant: 0.0,
        }
    }
}

/// VarId - f64
impl Sub<f64> for VarId {
    type Output = LinExpr;

    fn sub(self, rhs: f64) -> LinExpr {
        LinExpr {
            terms: vec![crate::LinTerm { var: self, coeff: 1.0 }],
            constant: -rhs,
        }
    }
}

/// f64 - VarId
impl Sub<VarId> for f64 {
    type Output = LinExpr;

    fn sub(self, rhs: VarId) -> LinExpr {
        LinExpr {
            terms: vec![crate::LinTerm { var: rhs, coeff: -1.0 }],
            constant: self,
        }
    }
}

/// VarId * f64
impl Mul<f64> for VarId {
    type Output = LinExpr;

    fn mul(self, rhs: f64) -> LinExpr {
        LinExpr::new(self, rhs)
    }
}

/// f64 * VarId
impl Mul<VarId> for f64 {
    type Output = LinExpr;

    fn mul(self, rhs: VarId) -> LinExpr {
        rhs * self
    }
}

/// VarId / f64
impl Div<f64> for VarId {
    type Output = LinExpr;

    fn div(self, rhs: f64) -> LinExpr {
        LinExpr::new(self, 1.0 / rhs)
    }
}
