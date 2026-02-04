use crate::LinExpr;

#[derive(Copy, Clone, Debug)]
pub enum Sense {
    Minimize,
    Maximize,
}

#[derive(Clone, Debug)]
pub struct Objective {
    pub sense: Sense,
    pub expr: LinExpr,
    pub name: Option<String>,
    pub priority: Option<u32>, // For multi-objective optimization
}

// Builder for ergonomic API
pub struct ObjectiveBuilder {
    objective: Objective,
}

impl ObjectiveBuilder {
    pub fn priority(mut self, p: u32) -> Self {
        self.objective.priority = Some(p);
        self
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Objective {
        self.objective.name = Some(name.into());
        self.objective
    }
}

impl Objective {
    pub fn minimize(expr: LinExpr) -> ObjectiveBuilder {
        ObjectiveBuilder {
            objective: Objective {
                sense: Sense::Minimize,
                expr,
                name: None,
                priority: None,
            },
        }
    }

    pub fn maximize(expr: LinExpr) -> ObjectiveBuilder {
        ObjectiveBuilder {
            objective: Objective {
                sense: Sense::Maximize,
                expr,
                name: None,
                priority: None,
            },
        }
    }
}
