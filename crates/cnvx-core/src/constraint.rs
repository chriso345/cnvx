use crate::LinExpr;

#[derive(Copy, Clone, Debug)]
pub enum Cmp {
    Le,
    Ge,
    Eq,
    Leq,
    Geq,
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub expr: LinExpr,
    pub rhs: f64,
    pub cmp: Cmp,
}

impl Constraint {
    pub fn leq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Leq }
    }

    pub fn geq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Geq }
    }

    pub fn eq(lhs: LinExpr, rhs: f64) -> Self {
        Self { expr: lhs, rhs, cmp: Cmp::Eq }
    }
}
