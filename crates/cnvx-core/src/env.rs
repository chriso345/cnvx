/// ```rust
/// # use cnvx_core::Env;
/// let env = Env::with_params(Params { time_limit: Some(30.0), threads: Some(8), ..Default::default() });
/// ```
pub struct Env {
    pub params: Params,
}

impl Env {
    pub fn new() -> Self {
        Env { params: Params::default() }
    }

    pub fn with_params(params: Params) -> Self {
        Env { params }
    }
}

/// One parameter bag, shared vocabulary across solvers.
#[derive(Clone, Debug)]
pub struct Params {
    pub time_limit: Option<f64>,
    pub mip_gap: f64,
    pub feasibility_tol: f64,
    pub optimality_tol: f64,
    pub threads: Option<u32>,
    pub output: bool,
    pub presolve: bool,
}

impl Default for Params {
    /* Gurobi-like sane defaults */
    fn default() -> Self {
        Params {
            time_limit: None,
            mip_gap: 1e-4,
            feasibility_tol: 1e-6,
            optimality_tol: 1e-6,
            threads: None,
            output: true,
            presolve: true,
        }
    }
}
