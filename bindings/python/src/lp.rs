use cnvx_lp::*;
use pyo3::prelude::*;

/// Wraps VarId so Python can hold a reference to a variable
#[pyclass]
pub struct Var {
    inner: VarId,
}

#[pyclass]
pub struct Model {
    inner: LpModel,
}

#[pymethods]
impl Model {
    #[new]
    pub fn new() -> Self {
        Self { inner: LpModel::new() }
    }

    /// model.add_var(name="Gas", lb=0.0, ub=200.0)
    pub fn add_var(
        &mut self,
        name: Option<&str>,
        lb: Option<f64>,
        ub: Option<f64>,
    ) -> Var {
        let b = self.inner.add_var();

        // VarBuilder consumes self on each call, so we apply via the model directly
        // after finish()
        let id = b.var;

        let var = &mut self.inner.vars[id.0];
        if let Some(n) = name {
            var.name = Some(n.to_string());
        }
        if let Some(l) = lb {
            var.lb = Some(l);
        }
        if let Some(u) = ub {
            var.ub = Some(u);
        }

        Var { inner: id }
    }

    /// model.minimize(expr, name="Cost")
    pub fn minimize(&mut self, expr: &LinExprPy, name: Option<&str>) {
        let b = Objective::minimize(expr.inner.clone());
        let obj = if let Some(n) = name { b.name(n) } else { b.name("objective") };
        self.inner.add_objective(obj);
    }

    /// model.maximize(expr, name="Profit")
    pub fn maximize(&mut self, expr: &LinExprPy, name: Option<&str>) {
        let b = Objective::maximize(expr.inner.clone());
        let obj = if let Some(n) = name { b.name(n) } else { b.name("objective") };
        self.inner.add_objective(obj);
    }

    /// model.add_constraint(expr.eq(300.0))
    pub fn add_constraint(&mut self, c: &ConstraintPy) {
        self.inner += c.inner.clone();
    }

    pub fn solve(&mut self) -> PyResult<Solution> {
        let mut solver = LpSolver::new();
        solver
            .solve(&self.inner)
            .map(|s| Solution { inner: s })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

#[pyclass]
pub struct LinExprPy {
    pub inner: LinExpr,
}

#[pymethods]
impl LinExprPy {
    /// expr.eq(rhs) - ConstraintPy
    pub fn eq(&self, rhs: f64) -> ConstraintPy {
        ConstraintPy { inner: self.inner.clone().eq(rhs) }
    }

    pub fn leq(&self, rhs: f64) -> ConstraintPy {
        ConstraintPy { inner: self.inner.clone().leq(rhs) }
    }

    pub fn geq(&self, rhs: f64) -> ConstraintPy {
        ConstraintPy { inner: self.inner.clone().geq(rhs) }
    }

    /// expr + expr  or  expr + var
    pub fn __add__(&self, other: &LinExprPy) -> LinExprPy {
        LinExprPy { inner: self.inner.clone() + other.inner.clone() }
    }

    /// expr * scalar
    pub fn __mul__(&self, rhs: f64) -> LinExprPy {
        let scaled: LinExpr = self
            .inner
            .terms
            .iter()
            .map(|t| LinExpr::new(t.var, t.coeff * rhs))
            .fold(LinExpr::constant(self.inner.constant * rhs), |acc, e| acc + e);
        LinExprPy { inner: scaled }
    }

    pub fn __rmul__(&self, lhs: f64) -> LinExprPy {
        self.__mul__(lhs)
    }
}

#[pymethods]
impl Var {
    /// var.expr() - LinExprPy  (used for arithmetic)
    pub fn expr(&self) -> LinExprPy {
        LinExprPy { inner: LinExpr::from(self.inner) }
    }

    pub fn leq(&self, other: &LinExprPy) -> ConstraintPy {
        ConstraintPy { inner: self.inner.leq(other.inner.clone()) }
    }

    pub fn geq(&self, other: &LinExprPy) -> ConstraintPy {
        ConstraintPy { inner: self.inner.geq(other.inner.clone()) }
    }

    pub fn eq(&self, other: &LinExprPy) -> ConstraintPy {
        ConstraintPy { inner: self.inner.eq(other.inner.clone()) }
    }

    /// Scalar multiply: var * 50.0
    pub fn __mul__(&self, rhs: f64) -> LinExprPy {
        LinExprPy { inner: self.inner * rhs }
    }

    pub fn __rmul__(&self, lhs: f64) -> LinExprPy {
        self.__mul__(lhs)
    }

    /// var + var  or  var + expr
    pub fn __add__(&self, other: &LinExprPy) -> LinExprPy {
        LinExprPy {
            inner: LinExpr::from(self.inner) + other.inner.clone(),
        }
    }

    pub fn __radd__(&self, other: &LinExprPy) -> LinExprPy {
        self.__add__(other)
    }
}

#[pyclass]
pub struct ConstraintPy {
    pub inner: LinearConstraint,
}

#[pyclass]
pub struct Solution {
    inner: LpSolution,
}

#[pymethods]
impl Solution {
    /// solution.value(var) - f64
    pub fn value(&self, var: &Var) -> f64 {
        self.inner.value(var.inner)
    }

    #[getter]
    pub fn objective_value(&self) -> Option<f64> {
        self.inner.objective_value
    }

    pub fn __repr__(&self) -> String {
        format!("Solution(objective={})", self.inner.objective_value.unwrap_or(0.0))
    }
}

pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();
    let m = PyModule::new(py, "lp")?;

    m.add_class::<Model>()?;
    m.add_class::<Var>()?;
    m.add_class::<LinExprPy>()?;
    m.add_class::<ConstraintPy>()?;
    m.add_class::<Solution>()?;

    parent.add_submodule(&m)?;
    py.import("sys")?.getattr("modules")?.set_item("cnvx.lp", &m)?;

    Ok(())
}
