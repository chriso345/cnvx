use cnvx_lp::*;
use pyo3::prelude::*;

#[derive(FromPyObject)]
pub enum Operand<'a> {
    Expr(PyRef<'a, LinExprPy>),
    Var(PyRef<'a, Var>),
    Float(f64),
}

impl Operand<'_> {
    fn into_expr(self) -> LinExpr {
        match self {
            Operand::Expr(e) => e.inner.clone(),
            Operand::Var(v) => LinExpr::from(v.inner),
            Operand::Float(f) => LinExpr::constant(f),
        }
    }
}

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

    pub fn add_var(
        &mut self,
        name: Option<&str>,
        lb: Option<f64>,
        ub: Option<f64>,
    ) -> Var {
        let mut b = self.inner.add_var();
        if let Some(n) = name {
            b = b.name(n);
        }
        if let Some(l) = lb {
            b = b.lower_bound(l);
        }
        if let Some(u) = ub {
            b = b.upper_bound(u);
        }

        Var { inner: b.finish() }
    }

    pub fn minimize(&mut self, expr: &LinExprPy, name: Option<&str>) {
        let b = Objective::minimize(expr.inner.clone());
        let obj = if let Some(n) = name { b.name(n) } else { b.name("objective") };
        self.inner.add_objective(obj);
    }

    pub fn maximize(&mut self, expr: &LinExprPy, name: Option<&str>) {
        let b = Objective::maximize(expr.inner.clone());
        let obj = if let Some(n) = name { b.name(n) } else { b.name("objective") };
        self.inner.add_objective(obj);
    }

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
    pub fn eq(&self, rhs: Operand) -> ConstraintPy {
        self.__eq__(rhs)
    }
    pub fn leq(&self, rhs: Operand) -> ConstraintPy {
        self.__le__(rhs)
    }
    pub fn geq(&self, rhs: Operand) -> ConstraintPy {
        self.__ge__(rhs)
    }

    pub fn __eq__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: self.inner.clone().eq(f) },
            _ => {
                let mut diff = self.inner.clone() - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.eq(bound) }
            }
        }
    }

    pub fn __le__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: self.inner.clone().leq(f) },
            _ => {
                let mut diff = self.inner.clone() - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.leq(bound) }
            }
        }
    }

    pub fn __ge__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: self.inner.clone().geq(f) },
            _ => {
                let mut diff = self.inner.clone() - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.geq(bound) }
            }
        }
    }

    pub fn __add__(&self, rhs: Operand) -> LinExprPy {
        match rhs {
            Operand::Float(f) => {
                let mut e = self.inner.clone();
                e.constant += f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: self.inner.clone() + rhs.into_expr() },
        }
    }
    pub fn __radd__(&self, lhs: Operand) -> LinExprPy {
        self.__add__(lhs)
    }

    pub fn __sub__(&self, rhs: Operand) -> LinExprPy {
        match rhs {
            Operand::Float(f) => {
                let mut e = self.inner.clone();
                e.constant -= f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: self.inner.clone() - rhs.into_expr() },
        }
    }
    pub fn __rsub__(&self, lhs: Operand) -> LinExprPy {
        match lhs {
            Operand::Float(f) => {
                let mut e = -self.inner.clone();
                e.constant += f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: lhs.into_expr() - self.inner.clone() },
        }
    }

    pub fn __mul__(&self, rhs: f64) -> LinExprPy {
        LinExprPy { inner: self.inner.clone() * rhs }
    }
    pub fn __rmul__(&self, lhs: f64) -> LinExprPy {
        LinExprPy { inner: lhs * self.inner.clone() }
    }
    pub fn __truediv__(&self, rhs: f64) -> LinExprPy {
        LinExprPy { inner: self.inner.clone() / rhs }
    }
    pub fn __neg__(&self) -> LinExprPy {
        LinExprPy { inner: -self.inner.clone() }
    }
}

#[pymethods]
impl Var {
    pub fn expr(&self) -> LinExprPy {
        LinExprPy { inner: LinExpr::from(self.inner) }
    }

    pub fn eq(&self, rhs: Operand) -> ConstraintPy {
        self.__eq__(rhs)
    }
    pub fn leq(&self, rhs: Operand) -> ConstraintPy {
        self.__le__(rhs)
    }
    pub fn geq(&self, rhs: Operand) -> ConstraintPy {
        self.__ge__(rhs)
    }

    pub fn __eq__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: LinExpr::from(self.inner).eq(f) },
            _ => {
                let mut diff = LinExpr::from(self.inner) - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.eq(bound) }
            }
        }
    }

    pub fn __le__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: LinExpr::from(self.inner).leq(f) },
            _ => {
                let mut diff = LinExpr::from(self.inner) - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.leq(bound) }
            }
        }
    }

    pub fn __ge__(&self, rhs: Operand) -> ConstraintPy {
        match rhs {
            Operand::Float(f) => ConstraintPy { inner: LinExpr::from(self.inner).geq(f) },
            _ => {
                let mut diff = LinExpr::from(self.inner) - rhs.into_expr();
                let bound = -diff.constant;
                diff.constant = 0.0;
                ConstraintPy { inner: diff.geq(bound) }
            }
        }
    }

    pub fn __add__(&self, rhs: Operand) -> LinExprPy {
        match rhs {
            Operand::Float(f) => {
                let mut e = LinExpr::from(self.inner);
                e.constant += f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: LinExpr::from(self.inner) + rhs.into_expr() },
        }
    }
    pub fn __radd__(&self, lhs: Operand) -> LinExprPy {
        self.__add__(lhs)
    }

    pub fn __sub__(&self, rhs: Operand) -> LinExprPy {
        match rhs {
            Operand::Float(f) => {
                let mut e = LinExpr::from(self.inner);
                e.constant -= f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: LinExpr::from(self.inner) - rhs.into_expr() },
        }
    }
    pub fn __rsub__(&self, lhs: Operand) -> LinExprPy {
        match lhs {
            Operand::Float(f) => {
                let mut e = -LinExpr::from(self.inner);
                e.constant += f;
                LinExprPy { inner: e }
            }
            _ => LinExprPy { inner: lhs.into_expr() - LinExpr::from(self.inner) },
        }
    }

    pub fn __mul__(&self, rhs: f64) -> LinExprPy {
        LinExprPy { inner: self.inner * rhs }
    }
    pub fn __rmul__(&self, lhs: f64) -> LinExprPy {
        LinExprPy { inner: lhs * self.inner }
    }
    pub fn __truediv__(&self, rhs: f64) -> LinExprPy {
        LinExprPy { inner: self.inner / rhs }
    }
    pub fn __neg__(&self) -> LinExprPy {
        LinExprPy { inner: -self.inner }
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
