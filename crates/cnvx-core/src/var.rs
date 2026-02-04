use crate::expr::LinExpr;
use std::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

#[derive(Clone, Debug)]
pub struct Var {
    pub id: VarId,
    pub lb: Option<f64>,
    pub ub: Option<f64>,
    pub is_integer: bool,
}

// Builder handle for ergonomic API
pub struct VarBuilder<'a> {
    pub(crate) model: &'a mut crate::Model,
    pub(crate) var: VarId,
}

impl<'a> VarBuilder<'a> {
    /// Set lower bound
    pub fn lower_bound(self, lb: f64) -> Self {
        self.model.vars[self.var.0].lb = Some(lb);
        self
    }

    /// Set upper bound
    pub fn upper_bound(self, ub: f64) -> Self {
        self.model.vars[self.var.0].ub = Some(ub);
        self
    }

    /// Mark as integer variable
    pub fn integer(self) -> Self {
        self.model.vars[self.var.0].is_integer = true;
        self
    }

    /// Mark as binary variable (0..1 integer)
    pub fn binary(self) -> Self {
        let var = &mut self.model.vars[self.var.0];
        var.is_integer = true;
        var.lb = Some(0.0);
        var.ub = Some(1.0);
        self
    }

    /// Finish variable building and return VarId
    pub fn finish(self) -> VarId {
        self.var
    }
}

/// Allow converting VarBuilder into VarId
impl<'a> From<VarBuilder<'a>> for VarId {
    fn from(builder: VarBuilder<'a>) -> Self {
        builder.var
    }
}

// Allow VarId * f64 → LinExpr
impl Mul<f64> for VarId {
    type Output = LinExpr;

    fn mul(self, rhs: f64) -> LinExpr {
        LinExpr::new(self, rhs)
    }
}

// Allow f64 * VarId → LinExpr
impl Mul<VarId> for f64 {
    type Output = LinExpr;

    fn mul(self, rhs: VarId) -> LinExpr {
        rhs * self
    }
}
