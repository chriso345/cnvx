use crate::{Constraint, VarId};
use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

/// A single term in a linear expression: `coeff * var`.
#[derive(Clone, Debug)]
pub struct LinTerm {
    /// The variable involved in this term.
    pub var: VarId,
    /// The coefficient for the variable.
    pub coeff: f64,
}

/// Represents a linear expression of the form `a1*x1 + a2*x2 + ... + c`.
#[derive(Clone, Debug, Default)]
pub struct LinExpr {
    /// All variable terms in the expression.
    pub terms: Vec<LinTerm>,
    /// Constant term in the expression.
    pub constant: f64,
}

// TODO: Currently LinExpr only implements addition, but we want support for subtraction and negation.
// This will later likely pivot to a more general `Expr` type for non-linear support.

impl LinExpr {
    /// Creates a new linear expression from a single variable and coefficient.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::{LinExpr, VarId};
    /// let x = VarId(0);
    /// let expr = LinExpr::new(x, 3.0); // 3*VarId(0)
    /// ```
    pub fn new(var: VarId, coeff: f64) -> Self {
        Self { terms: vec![LinTerm { var, coeff }], constant: 0.0 }
    }

    /// Creates a constant-only linear expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_core::LinExpr;
    /// let expr = LinExpr::constant(5.0); // 5
    /// ```
    pub fn constant(c: f64) -> Self {
        Self { terms: vec![], constant: c }
    }

    /// Creates a `<=` constraint from this linear expression.
    pub fn leq(self, rhs: f64) -> Constraint {
        Constraint::leq(self, rhs)
    }

    /// Creates a `>=` constraint from this linear expression.
    pub fn geq(self, rhs: f64) -> Constraint {
        Constraint::geq(self, rhs)
    }

    /// Creates a `==` constraint from this linear expression.
    pub fn eq(self, rhs: f64) -> Constraint {
        Constraint::eq(self, rhs)
    }
}

impl Display for LinExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        for term in &self.terms {
            parts.push(format!("{}*VarId({})", term.coeff, term.var.0));
        }
        if self.constant != 0.0 || parts.is_empty() {
            parts.push(self.constant.to_string());
        }
        write!(f, "{}", parts.join(" + "))
    }
}

/////////////////////////////////////////////////////////////////////////////
// Operator Overloads for LinExpr
/////////////////////////////////////////////////////////////////////////////

/// LinExpr + LinExpr
impl Add for LinExpr {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut terms = self.terms;
        terms.extend(rhs.terms);
        LinExpr { terms, constant: self.constant + rhs.constant }
    }
}

/// LinExpr + VarId
impl Add<VarId> for LinExpr {
    type Output = LinExpr;

    fn add(mut self, rhs: VarId) -> LinExpr {
        self.terms.push(LinTerm { var: rhs, coeff: 1.0 });
        self
    }
}

/// VarId + LinExpr
impl Add<LinExpr> for VarId {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut terms = vec![LinTerm { var: self, coeff: 1.0 }];
        terms.extend(rhs.terms);
        LinExpr { terms, constant: rhs.constant }
    }
}

/// VarId + VarId
impl Add for VarId {
    type Output = LinExpr;

    fn add(self, rhs: VarId) -> LinExpr {
        LinExpr {
            terms: vec![
                LinTerm { var: self, coeff: 1.0 },
                LinTerm { var: rhs, coeff: 1.0 },
            ],
            constant: 0.0,
        }
    }
}

/// LinExpr += LinExpr
impl AddAssign for LinExpr {
    fn add_assign(&mut self, rhs: LinExpr) {
        self.terms.extend(rhs.terms);
        self.constant += rhs.constant;
    }
}

/// LinExpr += VarId
impl AddAssign<VarId> for LinExpr {
    fn add_assign(&mut self, rhs: VarId) {
        self.terms.push(LinTerm { var: rhs, coeff: 1.0 });
    }
}

/// f64 + LinExpr
impl Add<LinExpr> for f64 {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut expr = rhs.clone();
        expr.constant += self;
        expr
    }
}

/// LinExpr + f64
impl Add<f64> for LinExpr {
    type Output = LinExpr;

    fn add(mut self, rhs: f64) -> LinExpr {
        self.constant += rhs;
        self
    }
}

/// Allows converting a single variable into a linear expression with coefficient 1.0.
impl From<VarId> for LinExpr {
    fn from(var: VarId) -> Self {
        LinExpr::new(var, 1.0)
    }
}
