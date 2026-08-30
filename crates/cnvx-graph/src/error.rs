/// Errors produced by building, mutating, or solving against a
/// [`Graph`](crate::Graph).
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    /// A handle was used that does not exist in the graph.
    #[error("handle from a different Graph, or invalidated by a structural mutation")]
    ForeignHandle,
    /// An operation was attempted on a graph that is not allowed for its type
    /// (e.g. adding an edge to a directed graph when the edge is undirected).
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// A negative-weight cycle was detected in a graph where it is not allowed.
    #[error("negative-weight cycle present")]
    NegativeCycle,
    /// A lower-level [`cnvx_math::MathError`] propagated up unchanged
    #[error(transparent)]
    Math(#[from] cnvx_math::MathError),
    /// A shared [`cnvx_core::CnvxError`] propagated up unchanged.
    #[error(transparent)]
    Core(#[from] cnvx_core::CnvxError),
}

/// A `Result` alias using [`GraphError`].
pub type Result<T> = std::result::Result<T, GraphError>;
