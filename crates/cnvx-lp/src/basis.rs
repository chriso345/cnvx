use cnvx_core::Var;

/// The status of one variable in a simplex basis.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VarStatus {
    /// The variable is basic (its value is determined by the current
    /// basis, not pinned to a bound).
    Basic,
    /// The variable is nonbasic, pinned at its lower bound.
    AtLower,
    /// The variable is nonbasic, pinned at its upper bound.
    AtUpper,
}

/// A snapshot of which variables were basic (and which bound the
/// nonbasic ones sat at) at the end of a solve.
///
/// Returned by [`crate::LpSolution::basis`] and accepted by
/// [`crate::LpSolver::warm_start`] to seed a subsequent, related solve
/// (e.g. after a small change to the model) without starting from
/// scratch.
#[derive(Clone, Debug)]
pub struct Basis {
    statuses: Vec<(Var, VarStatus)>,
}

impl Basis {
    pub(crate) fn new(statuses: Vec<(Var, VarStatus)>) -> Self {
        Basis { statuses }
    }

    /// Iterates over every variable's status in this basis.
    pub fn statuses(&self) -> impl Iterator<Item = (Var, VarStatus)> + '_ {
        self.statuses.iter().copied()
    }

    /// The status of a single variable, if it appears in this basis.
    pub fn status(&self, var: Var) -> Option<VarStatus> {
        self.statuses.iter().find(|(v, _)| *v == var).map(|(_, s)| *s)
    }
}
