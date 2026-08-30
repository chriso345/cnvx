/// The search strategy a solver runs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum MilpMethod {
    /// Branch-and-bound: solve the LP relaxation at each node, branch on
    /// the most-fractional integer variable when the relaxation isn't
    /// already integral, and prune nodes that can't beat the current
    /// incumbent.
    BranchAndBound,

    /// Branch-and-bound specialized for models where every variable is
    /// `VarKind::Binary`. Currently just a wrapped around of
    /// [`MilpMethod::BranchAndBound`].
    Binary,

    /// Branch-and-bound augmented with cutting planes (e.g. Gomory cuts)
    /// added at each node to tighten the LP relaxation before branching,
    /// shrinking the search tree at the cost of extra per-node work.
    BranchAndCut,

    /// Branch-and-bound where each node's LP relaxation is solved by
    /// column generation (pricing out variables) instead of directly.
    BranchAndPrice,
}
