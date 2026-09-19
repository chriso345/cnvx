use std::ops::{Add, Mul, Neg, Sub};
use std::rc::Rc;

use crate::var::Var;

/// A linear combination of variables: `c0 + c1*x1 + c2*x2 + ...`.
///
/// Built via operator overloading (`x + y`, `2.0 * x`, `expr - 3.0`, and so
/// on) rather than a macro or a string-based expression parser. `Expression`
/// is cheap to clone: the term list is reference-counted, so passing one
/// around or building several related expressions from a shared prefix does
/// not repeatedly copy the underlying coefficients.
///
/// The same variable may appear more than once in [`Expression::terms`] if
/// it was added to the expression more than once (e.g. `x + x`); code that
/// consumes an `Expression` (such as
/// [`Model::add_constraint`](crate::Model::add_constraint)) coalesces duplicate
/// variables into a single coefficient.
#[derive(Clone, Debug)]
pub struct Expression {
    constant: f64,
    terms: Rc<[(Var, f64)]>,
}

impl Expression {
    /// The zero expression: no terms, constant `0.0`.
    pub fn zero() -> Self {
        Self { constant: 0.0, terms: Rc::from([]) }
    }

    /// The constant term (`c0`).
    pub fn constant_term(&self) -> f64 {
        self.constant
    }

    /// Iterates over the `(variable, coefficient)` pairs in insertion order.
    pub fn terms(&self) -> impl Iterator<Item = (Var, f64)> + '_ {
        self.terms.iter().copied()
    }

    fn from_constant(constant: f64) -> Self {
        Self { constant, terms: Rc::from([]) }
    }

    fn from_term(var: Var, coeff: f64) -> Self {
        Self { constant: 0.0, terms: Rc::from([(var, coeff)]) }
    }

    /// Splits into `(constant, expression with that constant removed)`.
    ///
    /// Used by [`crate::Constraint`] construction to move a comparison's
    /// constant term onto the right-hand side (e.g. `(x + 3.0).leq(10.0)`
    /// becomes the linear part `x` compared against `7.0`).
    pub(crate) fn without_constant(&self) -> (f64, Expression) {
        (self.constant, Expression { constant: 0.0, terms: Rc::clone(&self.terms) })
    }

    /// `self + other * other_sign`, used to implement both `Add` and `Sub`.
    fn combine(&self, other: &Expression, other_sign: f64) -> Expression {
        let mut terms = Vec::with_capacity(self.terms.len() + other.terms.len());
        terms.extend_from_slice(&self.terms);
        terms.extend(other.terms.iter().map(|&(v, c)| (v, c * other_sign)));
        Expression {
            constant: self.constant + other.constant * other_sign,
            terms: terms.into(),
        }
    }

    /// `self * factor`, used to implement both `Mul<f64>` and `Neg`.
    fn scaled(&self, factor: f64) -> Expression {
        Expression {
            constant: self.constant * factor,
            terms: self.terms.iter().map(|&(v, c)| (v, c * factor)).collect(),
        }
    }
}

impl From<Var> for Expression {
    fn from(v: Var) -> Self {
        Expression::from_term(v, 1.0)
    }
}

impl From<f64> for Expression {
    fn from(c: f64) -> Self {
        Expression::from_constant(c)
    }
}

// ---- Addition: Expression/Var/f64 in any combination ----

impl Add<Expression> for Expression {
    type Output = Expression;

    fn add(self, rhs: Expression) -> Expression {
        self.combine(&rhs, 1.0)
    }
}

impl Add<f64> for Expression {
    type Output = Expression;

    fn add(self, rhs: f64) -> Expression {
        self.combine(&Expression::from_constant(rhs), 1.0)
    }
}

impl Add<Expression> for f64 {
    type Output = Expression;

    fn add(self, rhs: Expression) -> Expression {
        rhs + self
    }
}

impl Add<Var> for Expression {
    type Output = Expression;

    fn add(self, rhs: Var) -> Expression {
        self.combine(&Expression::from(rhs), 1.0)
    }
}

impl Add<Expression> for Var {
    type Output = Expression;

    fn add(self, rhs: Expression) -> Expression {
        Expression::from(self) + rhs
    }
}

impl Add<Var> for Var {
    type Output = Expression;

    fn add(self, rhs: Var) -> Expression {
        Expression::from(self) + rhs
    }
}

impl Add<f64> for Var {
    type Output = Expression;

    fn add(self, rhs: f64) -> Expression {
        Expression::from(self) + rhs
    }
}

impl Add<Var> for f64 {
    type Output = Expression;

    fn add(self, rhs: Var) -> Expression {
        Expression::from(rhs) + self
    }
}

// ---- Subtraction: Expression/Var/f64 in any combination ----

impl Sub<Expression> for Expression {
    type Output = Expression;

    fn sub(self, rhs: Expression) -> Expression {
        self.combine(&rhs, -1.0)
    }
}

impl Sub<f64> for Expression {
    type Output = Expression;

    fn sub(self, rhs: f64) -> Expression {
        self.combine(&Expression::from_constant(rhs), -1.0)
    }
}

impl Sub<Expression> for f64 {
    type Output = Expression;

    fn sub(self, rhs: Expression) -> Expression {
        Expression::from_constant(self).combine(&rhs, -1.0)
    }
}

impl Sub<Var> for Expression {
    type Output = Expression;

    fn sub(self, rhs: Var) -> Expression {
        self.combine(&Expression::from(rhs), -1.0)
    }
}

impl Sub<Expression> for Var {
    type Output = Expression;

    fn sub(self, rhs: Expression) -> Expression {
        Expression::from(self).combine(&rhs, -1.0)
    }
}

impl Sub<Var> for Var {
    type Output = Expression;

    fn sub(self, rhs: Var) -> Expression {
        Expression::from(self).combine(&Expression::from(rhs), -1.0)
    }
}

impl Sub<f64> for Var {
    type Output = Expression;

    fn sub(self, rhs: f64) -> Expression {
        Expression::from(self).combine(&Expression::from_constant(rhs), -1.0)
    }
}

impl Sub<Var> for f64 {
    type Output = Expression;

    fn sub(self, rhs: Var) -> Expression {
        Expression::from_constant(self).combine(&Expression::from(rhs), -1.0)
    }
}

// ---- Negation ----

impl Neg for Expression {
    type Output = Expression;

    fn neg(self) -> Expression {
        self.scaled(-1.0)
    }
}

impl Neg for Var {
    type Output = Expression;

    fn neg(self) -> Expression {
        Expression::from(self).scaled(-1.0)
    }
}

// ---- Scalar multiplication ----

impl Mul<f64> for Var {
    type Output = Expression;

    fn mul(self, rhs: f64) -> Expression {
        Expression::from_term(self, rhs)
    }
}

impl Mul<Var> for f64 {
    type Output = Expression;

    fn mul(self, rhs: Var) -> Expression {
        rhs * self
    }
}

impl Mul<f64> for Expression {
    type Output = Expression;

    fn mul(self, rhs: f64) -> Expression {
        self.scaled(rhs)
    }
}

impl Mul<Expression> for f64 {
    type Output = Expression;

    fn mul(self, rhs: Expression) -> Expression {
        rhs * self
    }
}

// ---- Sum ----

impl std::iter::Sum<Var> for Expression {
    fn sum<I: Iterator<Item = Var>>(iter: I) -> Self {
        iter.fold(Expression::zero(), |acc, v| acc + v)
    }
}

impl std::iter::Sum<Expression> for Expression {
    fn sum<I: Iterator<Item = Expression>>(iter: I) -> Self {
        iter.fold(Expression::zero(), |acc, e| acc + e)
    }
}

/// Sums an iterator of [`Expression`]s (or anything convertible into one,
/// such as `Var` or `f64`).
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Model, sum};
/// let mut model = Model::new("example");
/// let vars = model.add_vars(3, 0.0..);
/// let coeffs = [1.0, 2.0, 3.0];
/// let cost = sum(vars.iter().zip(&coeffs).map(|(&v, &c)| c * v));
/// ```
pub fn sum<E: Into<Expression>, I: IntoIterator<Item = E>>(items: I) -> Expression {
    items
        .into_iter()
        .fold(Expression::zero(), |acc, item| acc + item.into())
}
