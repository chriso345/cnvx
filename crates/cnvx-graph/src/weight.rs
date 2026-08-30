/// A weight type that can be combined along a path and summarized as a
/// single scalar.
pub trait GraphWeight: Clone {
    /// Combines two weights encountered in sequence along a path (e.g. two
    /// edges being merged by
    /// [`Graph::contract_edge`](crate::Graph::contract_edge) into one).
    fn accumulate(&self, other: &Self) -> Self;

    /// A scalar summary of this weight.
    fn measure(&self) -> f64;
}

impl GraphWeight for f64 {
    fn accumulate(&self, other: &Self) -> Self {
        self + other
    }

    fn measure(&self) -> f64 {
        *self
    }
}

impl GraphWeight for cnvx_math::Vector {
    fn accumulate(&self, other: &Self) -> Self {
        self + other
    }

    fn measure(&self) -> f64 {
        self.norm()
    }
}

/// Pareto dominance for a multi-criteria weight: `self` is no worse than
/// `other` in every criterion and strictly better in at least one, under
/// a "lower is better" convention.
pub trait Dominance {
    /// `true` if `self` Pareto-dominates `other`.
    fn dominates(&self, other: &Self) -> bool;
    /// `true` if `self` weakly dominates `other` (dominates, or equal in
    /// every criterion).
    fn dominates_or_equal(&self, other: &Self) -> bool;
}

impl Dominance for cnvx_math::Vector {
    fn dominates(&self, other: &Self) -> bool {
        cnvx_math::Vector::dominates(self, other)
    }

    fn dominates_or_equal(&self, other: &Self) -> bool {
        cnvx_math::Vector::dominates_or_equal(self, other)
    }
}

/// A caller-supplied projection from an edge's full data down to a single
/// `f64` cost. This is used for single criteria algorithms.
pub type WeightFn<'a, E> = Box<dyn Fn(&E) -> f64 + 'a>;

/// A caller-supplied projection from an edge's full data to a vector of
/// criteria values, used by [`multi_criteria`](crate::multi_criteria)
/// algorithms.
pub type CriteriaFn<'a, E> = Box<dyn Fn(&E) -> cnvx_math::Vector + 'a>;
