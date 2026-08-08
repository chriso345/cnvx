use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{LinearConstraint, VarId};

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

// TODO: This will later likely pivot to a more general `Expr` type for
// non-linear support.

impl LinExpr {
    /// Creates a new linear expression from a single variable and coefficient.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use cnvx_lp::{LinExpr, VarId};
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
    /// # use cnvx_lp::LinExpr;
    /// let expr = LinExpr::constant(5.0); // 5
    /// ```
    pub fn constant(c: f64) -> Self {
        Self { terms: vec![], constant: c }
    }

    /// Creates a `<=` constraint from this linear expression.
    pub fn leq(self, rhs: f64) -> LinearConstraint {
        LinearConstraint::leq(self, rhs)
    }

    /// Creates a `>=` constraint from this linear expression.
    pub fn geq(self, rhs: f64) -> LinearConstraint {
        LinearConstraint::geq(self, rhs)
    }

    /// Creates a `==` constraint from this linear expression.
    pub fn eq(self, rhs: f64) -> LinearConstraint {
        LinearConstraint::eq(self, rhs)
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

/// LinExpr += f64
impl AddAssign<f64> for LinExpr {
    fn add_assign(&mut self, rhs: f64) {
        self.constant += rhs;
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

/// -LinExpr
impl Neg for LinExpr {
    type Output = LinExpr;

    fn neg(mut self) -> LinExpr {
        for term in &mut self.terms {
            term.coeff = -term.coeff;
        }
        self.constant = -self.constant;
        self
    }
}

/// LinExpr - LinExpr
impl Sub for LinExpr {
    type Output = LinExpr;

    fn sub(self, rhs: LinExpr) -> LinExpr {
        let mut terms = self.terms;
        for term in rhs.terms {
            terms.push(LinTerm { var: term.var, coeff: -term.coeff });
        }
        LinExpr { terms, constant: self.constant - rhs.constant }
    }
}

/// LinExpr - VarId
impl Sub<VarId> for LinExpr {
    type Output = LinExpr;

    fn sub(mut self, rhs: VarId) -> LinExpr {
        self.terms.push(LinTerm { var: rhs, coeff: -1.0 });
        self
    }
}

/// VarId - LinExpr
impl Sub<LinExpr> for VarId {
    type Output = LinExpr;

    fn sub(self, rhs: LinExpr) -> LinExpr {
        let mut expr = -rhs;
        expr.terms.push(LinTerm { var: self, coeff: 1.0 });
        expr
    }
}

/// LinExpr - f64
impl Sub<f64> for LinExpr {
    type Output = LinExpr;

    fn sub(mut self, rhs: f64) -> LinExpr {
        self.constant -= rhs;
        self
    }
}

/// f64 - LinExpr
impl Sub<LinExpr> for f64 {
    type Output = LinExpr;

    fn sub(self, rhs: LinExpr) -> LinExpr {
        let mut expr = -rhs;
        expr.constant += self;
        expr
    }
}

/// LinExpr -= LinExpr
impl SubAssign for LinExpr {
    fn sub_assign(&mut self, rhs: LinExpr) {
        for term in rhs.terms {
            self.terms.push(LinTerm { var: term.var, coeff: -term.coeff });
        }
        self.constant -= rhs.constant;
    }
}

/// LinExpr -= VarId
impl SubAssign<VarId> for LinExpr {
    fn sub_assign(&mut self, rhs: VarId) {
        self.terms.push(LinTerm { var: rhs, coeff: -1.0 });
    }
}

/// LinExpr -= f64
impl SubAssign<f64> for LinExpr {
    fn sub_assign(&mut self, rhs: f64) {
        self.constant -= rhs;
    }
}

/// LinExpr * f64
impl Mul<f64> for LinExpr {
    type Output = LinExpr;

    fn mul(mut self, rhs: f64) -> LinExpr {
        for term in &mut self.terms {
            term.coeff *= rhs;
        }
        self.constant *= rhs;
        self
    }
}

/// f64 * LinExpr
impl Mul<LinExpr> for f64 {
    type Output = LinExpr;

    fn mul(self, rhs: LinExpr) -> LinExpr {
        rhs * self
    }
}

/// LinExpr *= f64
impl MulAssign<f64> for LinExpr {
    fn mul_assign(&mut self, rhs: f64) {
        for term in &mut self.terms {
            term.coeff *= rhs;
        }
        self.constant *= rhs;
    }
}

/// LinExpr / f64
impl Div<f64> for LinExpr {
    type Output = LinExpr;

    fn div(mut self, rhs: f64) -> LinExpr {
        for term in &mut self.terms {
            term.coeff /= rhs;
        }
        self.constant /= rhs;
        self
    }
}

/// LinExpr /= f64
impl DivAssign<f64> for LinExpr {
    fn div_assign(&mut self, rhs: f64) {
        for term in &mut self.terms {
            term.coeff /= rhs;
        }
        self.constant /= rhs;
    }
}

/// Allows converting a single variable into a linear expression with
/// coefficient 1.0.
impl From<VarId> for LinExpr {
    fn from(var: VarId) -> Self {
        LinExpr::new(var, 1.0)
    }
}

/// Allows converting a constant into a linear expression.
impl From<f64> for LinExpr {
    fn from(c: f64) -> Self {
        LinExpr::constant(c)
    }
}
