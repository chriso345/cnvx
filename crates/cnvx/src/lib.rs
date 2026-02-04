pub use cnvx_core as core;
pub use cnvx_lp as lp;

pub mod prelude {
    pub use crate::core::*;
    pub use crate::lp::*;
}

pub mod solvers {
    pub use crate::lp::SimplexSolver;
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
