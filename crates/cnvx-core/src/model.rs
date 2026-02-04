use crate::*;
use std::ops::AddAssign;

#[derive(Debug, Default)]
pub struct Model {
    pub(crate) vars: Vec<Var>,
    pub(crate) constraints: Vec<Constraint>,
    pub(crate) objective: Option<Objective>, // TODO: Replace with Vec<Objective> for multi-objective
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_var(&mut self) -> VarBuilder<'_> {
        let id = VarId(self.vars.len());
        self.vars.push(Var { id, lb: None, ub: None, is_integer: false });
        VarBuilder { model: self, var: id }
    }

    pub fn add_objective(&mut self, obj: Objective) {
        self.objective = Some(obj);
    }

    // Read-only access for solvers
    pub fn vars(&self) -> &[Var] {
        &self.vars
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn objective(&self) -> Option<&Objective> {
        self.objective.as_ref()
    }
}

// Allow `model += constraint`
impl AddAssign<Constraint> for Model {
    fn add_assign(&mut self, rhs: Constraint) {
        self.constraints.push(rhs);
    }
}
