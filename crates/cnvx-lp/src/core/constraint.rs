//! Linear constraints for optimization models.

use std::fmt::Display;

use crate::LinExpr;

/// Comparison operators used in constraints.
#[derive(Copy, Clone, Debug)]
pub enum Cmp {
    /// Equality: `==`
    EQ,

    /// Less than or equal: `<=`
    LEQ,

    /// Greater than or equal: `>=`
    GEQ,
}

/// A linear constraint of the form `expr cmp rhs`.
///
/// # Examples
///
/// ```rust
/// # use cnvx_lp::{LinExpr, LinearConstraint, VarId};
/// let x = VarId(0);
/// let expr = LinExpr::new(x, 2.0) + 3.0;
///
/// let c1 = LinearConstraint::leq(expr.clone(), 5.0);  // 2*x0 + 3 <= 5
/// let c2 = LinearConstraint::geq(expr.clone(), 1.0);  // 2*x0 + 3 >= 1
/// let c3 = LinearConstraint::eq(expr, 4.0);           // 2*x0 + 3 == 4
/// ```
#[derive(Debug)]
pub struct LinearConstraint {
    /// The left-hand side linear expression of the constraint.
    pub expr: LinExpr, // TODO: Allow for this to be a more general expression type

    /// The right-hand side value of the constraint.
    pub rhs: f64,

    /// The comparison operator (==, <=, >=).
    pub cmp: Cmp,

    /// Optional human-readable name for the constraint, used in diagnostics and
    /// dual-variable reporting.
    pub name: Option<String>,
}

impl Clone for LinearConstraint {
    fn clone(&self) -> Self {
        // ExtensionMap does not implement Clone (its values are `dyn Any`);
        // cloning a constraint preserves all fields but drops extensions.
        // Sub-crates that rely on extensions should clone them explicitly.
        Self {
            expr: self.expr.clone(),
            rhs: self.rhs,
            cmp: self.cmp,
            name: self.name.clone(),
        }
    }
}

impl LinearConstraint {
    /// Creates a `<=` constraint: `lhs <= rhs`.
    pub fn leq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::LEQ, name: None }
    }

    /// Creates a `>=` constraint: `lhs >= rhs`.
    pub fn geq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::GEQ, name: None }
    }

    /// Creates a `==` constraint: `lhs == rhs`.
    pub fn eq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::EQ, name: None }
    }

    /// Attaches a human-readable name to this constraint (builder-style).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_lp::{LinExpr, VarId, LinearConstraint};
    /// let c = LinearConstraint::leq(LinExpr::from(VarId(0)), 10.0)
    ///     .named("capacity");
    /// assert_eq!(c.name.as_deref(), Some("capacity"));
    /// ```
    pub fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

impl Display for LinearConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmp_str = match self.cmp {
            Cmp::EQ => "==",
            Cmp::LEQ => "<=",
            Cmp::GEQ => ">=",
        };
        if let Some(name) = &self.name {
            write!(f, "[{}] {} {} {}", name, self.expr, cmp_str, self.rhs)
        } else {
            write!(f, "{} {} {}", self.expr, cmp_str, self.rhs)
        }
    }
}
