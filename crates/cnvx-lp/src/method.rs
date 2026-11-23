/// The native LP algorithm a solver runs.
///
/// DualSimplex and InteriorPoint are not yet implemented, and
/// solving with either returns `Err(CnvxError::Numerical(..))`. Use
/// [`LpMethod::PrimalSimplex`] (or [`crate::LpSolver::default`], which
/// dispatches to it) for now.
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum LpMethod {
    /// Two-phase primal simplex over bounded variables.
    PrimalSimplex,
    /// Dual simplex, warm-start friendly.
    DualSimplex,
    /// Interior point method.
    InteriorPoint,
}
