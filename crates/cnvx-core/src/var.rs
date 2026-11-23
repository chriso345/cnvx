/// A decision variable handle.
///
/// `Var` is a cheap, `Copy` index into the [`Model`](crate::Model) that
/// created it, not a string lookup: Each handle is tagged internally with
/// the generation of the model that produced it, so passing a `Var` from one
/// `Model` into a different `Model` is caught as
/// `Err(CnvxError::ForeignHandle)`
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Var {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// A constraint handle, returned by
/// [`Model::add_constraint`](crate::Model::add_constraint).
///
/// Same handle discipline as [`Var`]: cheap, `Copy`, tagged with the
/// generation of the model that created it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Con {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// The kind of value a variable is allowed to take.
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum VarKind {
    /// May take any value within its bounds.
    Continuous,
    /// May take only integer values within its bounds.
    Integer,
    /// May take only the values `0` or `1`. Bounds are fixed to `[0, 1]`.
    Binary,
    /// Either `0`, or a continuous value within its bounds.
    SemiContinuous,
    /// Either `0`, or an integer value within its bounds.
    SemiInteger,
}
