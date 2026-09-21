use cnvx_core::ObjectiveId;

/// A multi-objective solving method.
#[derive(Clone, Debug)]
pub enum MopMethod {
    /// Solve every objective in descending order of the priority given
    /// to [`cnvx_core::Model::add_objective`] (higher first).
    Lexicographic,
    /// Optimize `primary` directly, subject to every other objective in
    /// `limits` being no worse than its paired bound.
    EpsilonConstraint {
        /// The objective to optimize.
        primary: ObjectiveId,
        /// `(objective, bound)` pairs for every other objective to
        /// constrain.
        limits: Vec<(ObjectiveId, f64)>,
    },
    /// Optimize a single weighted sum of every objective, each
    /// normalized to a minimization sense first.
    WeightedSum {
        /// One weight per objective, in `Model::objectives()` order.
        weights: Vec<f64>,
    },
    /// The exact efficient (Pareto) front of a model with exactly two
    /// objectives and no non-continuous variables.
    Biobjective,
}
