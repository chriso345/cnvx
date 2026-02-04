use crate::VarId;

#[derive(Debug)]
pub struct Solution {
    pub values: Vec<f64>,
    pub objective_value: Option<f64>,
}

impl Solution {
    pub fn value(&self, var: VarId) -> f64 {
        self.values[var.0]
    }
}
