use crate::{Constraint, VarId};
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug)]
pub struct LinTerm {
    pub var: VarId,
    pub coeff: f64,
}

#[derive(Clone, Debug, Default)]
pub struct LinExpr {
    pub terms: Vec<LinTerm>,
    pub constant: f64,
}

impl LinExpr {
    pub fn new(var: VarId, coeff: f64) -> Self {
        Self { terms: vec![LinTerm { var, coeff }], constant: 0.0 }
    }

    pub fn constant(c: f64) -> Self {
        Self { terms: vec![], constant: c }
    }
}

// LinExpr + LinExpr
impl Add for LinExpr {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut terms = self.terms;
        terms.extend(rhs.terms);
        LinExpr { terms, constant: self.constant + rhs.constant }
    }
}

// LinExpr + VarId
impl Add<VarId> for LinExpr {
    type Output = LinExpr;

    fn add(mut self, rhs: VarId) -> LinExpr {
        self.terms.push(LinTerm { var: rhs, coeff: 1.0 });
        self
    }
}

// VarId + LinExpr
impl Add<LinExpr> for VarId {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut terms = vec![LinTerm { var: self, coeff: 1.0 }];
        terms.extend(rhs.terms);
        LinExpr { terms, constant: rhs.constant }
    }
}

// VarId + VarId
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

// LinExpr += LinExpr
impl AddAssign for LinExpr {
    fn add_assign(&mut self, rhs: LinExpr) {
        self.terms.extend(rhs.terms);
        self.constant += rhs.constant;
    }
}

// LinExpr += VarId
impl AddAssign<VarId> for LinExpr {
    fn add_assign(&mut self, rhs: VarId) {
        self.terms.push(LinTerm { var: rhs, coeff: 1.0 });
    }
}

// f64 + LinExpr
impl Add<LinExpr> for f64 {
    type Output = LinExpr;

    fn add(self, rhs: LinExpr) -> LinExpr {
        let mut expr = rhs.clone();
        expr.constant += self;
        expr
    }
}

// LinExpr + f64
impl Add<f64> for LinExpr {
    type Output = LinExpr;

    fn add(mut self, rhs: f64) -> LinExpr {
        self.constant += rhs;
        self
    }
}

impl LinExpr {
    pub fn le(self, rhs: f64) -> Constraint {
        Constraint::leq(self, rhs)
    }

    pub fn ge(self, rhs: f64) -> Constraint {
        Constraint::geq(self, rhs)
    }
}

impl From<VarId> for LinExpr {
    fn from(var: VarId) -> Self {
        LinExpr::new(var, 1.0)
    }
}
