use crate::LinExpr;
use std::fmt::Display;

/// Comparison operators used in constraints.
#[derive(Copy, Clone, Debug)]
pub enum Cmp {
    /// Equality: `==`
    Eq,

    /// Less than or equal: `<=`
    Leq,

    /// Greater than or equal: `>=`
    Geq,
}

/// A linear constraint of the form `expr cmp rhs`.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{LinExpr, Constraint, VarId};
/// let x = VarId(0);
/// let expr = LinExpr::new(x, 2.0) + 3.0;
///
/// let c1 = Constraint::leq(expr.clone(), 5.0);  // 2*x0 + 3 <= 5
/// let c2 = Constraint::geq(expr.clone(), 1.0);  // 2*x0 + 3 >= 1
/// let c3 = Constraint::eq(expr, 4.0);           // 2*x0 + 3 == 4
/// ```
#[derive(Clone, Debug)]
pub struct Constraint {
    /// The left-hand side linear expression of the constraint.
    pub expr: LinExpr,

    /// The right-hand side value of the constraint.
    pub rhs: f64,

    /// The comparison operator (==, <=, >=).
    pub cmp: Cmp,
}

impl Constraint {
    /// Creates a `<=` constraint: `lhs <= rhs`.
    pub fn leq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Leq }
    }

    /// Creates a `>=` constraint: `lhs >= rhs`.
    pub fn geq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Geq }
    }

    /// Creates a `==` constraint: `lhs == rhs`.
    pub fn eq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Eq }
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmp_str = match self.cmp {
            Cmp::Eq => "==",
            Cmp::Leq => "<=",
            Cmp::Geq => ">=",
        };
        write!(f, "{} {} {}", self.expr, cmp_str, self.rhs)
    }
}
