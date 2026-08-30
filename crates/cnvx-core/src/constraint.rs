use crate::Var;
use crate::expr::Expression;

/// The comparison a [`Constraint`] enforces on its linear part.
///
/// All bounds here have already had the originating [`Expression`]s'
/// constant terms folded in, so a `Constraint`'s [`Constraint::expr`] is
/// always a pure linear combination (constant term `0.0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstraintKind {
    /// `expr <= rhs`.
    Leq(f64),
    /// `expr >= rhs`.
    Geq(f64),
    /// `expr == rhs`.
    Eq(f64),
    /// `lower <= expr <= upper`.
    Ranged {
        /// The lower bound.
        lower: f64,

        /// The upper bound.
        upper: f64,
    },
}

impl ConstraintKind {
    /// Shifts every bound by `delta`.
    ///
    /// Used to move an expression's constant term onto the right-hand
    /// side during [`Constraint`] construction.
    fn shifted(self, delta: f64) -> ConstraintKind {
        match self {
            ConstraintKind::Leq(rhs) => ConstraintKind::Leq(rhs + delta),
            ConstraintKind::Geq(rhs) => ConstraintKind::Geq(rhs + delta),
            ConstraintKind::Eq(rhs) => ConstraintKind::Eq(rhs + delta),
            ConstraintKind::Ranged { lower, upper } => {
                ConstraintKind::Ranged { lower: lower + delta, upper: upper + delta }
            }
        }
    }
}

/// A linear [`Expression`] compared against one or two bounds, ready to be
/// added to a [`Model`](crate::Model) via
/// [`Model::add_constraint`](crate::Model::add_constraint).
///
/// Built from an `Expression` via [`Expression::leq`], [`Expression::geq`],
/// [`Expression::eq`], or [`Expression::between`].
#[derive(Clone, Debug)]
pub struct Constraint {
    expr: Expression,
    kind: ConstraintKind,
    name: Option<String>,
}

impl Constraint {
    fn new(expr: Expression, kind: ConstraintKind) -> Self {
        let (constant, linear) = expr.without_constant();

        Constraint {
            expr: linear,
            kind: kind.shifted(-constant),
            name: None,
        }
    }

    /// The linear part being compared.
    ///
    /// Its constant term is always `0.0`; any constant from the originating
    /// expression has already been moved into [`Constraint::kind`]'s bound(s).
    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    /// The comparison and its bound(s).
    pub fn kind(&self) -> ConstraintKind {
        self.kind
    }

    /// The constraint's name, if one was set via [`Constraint::named`].
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Attaches a human-readable name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::Model;
    /// let mut model = Model::new("example");
    /// let x = model.add_var(0.0..);
    /// let y = model.add_var(0.0..);
    /// model.add_constraint((x + y).leq(20.0).named("supply_limit")).unwrap();
    /// ```
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Expression {
    /// Builds `self <= rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression. Internally this is normalized as:
    ///
    /// `self - rhs <= 0`
    pub fn leq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Constraint::new(self - rhs.into(), ConstraintKind::Leq(0.0))
    }

    /// Builds `self >= rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression. Internally this is normalized as:
    ///
    /// `self - rhs >= 0`
    pub fn geq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Constraint::new(self - rhs.into(), ConstraintKind::Geq(0.0))
    }

    /// Builds `self == rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression. Internally this is normalized as:
    ///
    /// `self - rhs == 0`
    pub fn eq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Constraint::new(self - rhs.into(), ConstraintKind::Eq(0.0))
    }

    /// Builds `lower <= self <= upper`, a ranged constraint.
    ///
    /// This never panics, even if `lower > upper`; that is reported as
    /// `Err(CnvxError::InvalidBounds)` when the constraint is added to a
    /// [`Model`](crate::Model).
    pub fn between(self, lower: f64, upper: f64) -> Constraint {
        Constraint::new(self, ConstraintKind::Ranged { lower, upper })
    }
}

impl Var {
    /// Builds `self <= rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression.
    pub fn leq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Expression::from(self).leq(rhs)
    }

    /// Builds `self >= rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression.
    pub fn geq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Expression::from(self).geq(rhs)
    }

    /// Builds `self == rhs`.
    ///
    /// The right-hand side may be either a scalar or another linear
    /// expression.
    pub fn eq<Rhs>(self, rhs: Rhs) -> Constraint
    where
        Rhs: Into<Expression>,
    {
        Expression::from(self).eq(rhs)
    }

    /// Builds `lower <= self <= upper`.
    pub fn between(self, lower: f64, upper: f64) -> Constraint {
        Expression::from(self).between(lower, upper)
    }
}
