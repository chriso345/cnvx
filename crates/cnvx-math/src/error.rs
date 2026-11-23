//! Error type shared by all fallible `cnvx-math` operations.

/// Errors produced by [`crate::Vector`], [`crate::Matrix`], and
/// [`crate::SparseMatrix`] operations.
///
/// Dimension checks (e.g. adding a 2x3 and a 3x2 matrix) return
/// `Err` rather than panicking; low-level arithmetic operator overloads
/// (`Vector`'s `Add`/`Sub`/etc.) panic instead, since those are only ever
/// called with statically-known-compatible shapes internally.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MathError {
    /// Two operands had incompatible shapes.
    #[error("dimension mismatch: {0}")]
    DimensionMismatch(String),

    /// The matrix is (numerically) singular and `solve` cannot proceed.
    #[error("singular matrix")]
    Singular,

    /// A symmetric matrix failed to factor as symmetric positive definite
    /// during a Cholesky-based solve.
    #[error("matrix is not symmetric positive definite")]
    NotPositiveDefinite,

    /// A rectangular system did not have full column rank, so the
    /// least-squares solve could not produce a unique solution.
    #[error("matrix is rank deficient")]
    RankDeficient,

    /// The underlying native solver rejected one of its arguments.
    #[error("invalid argument to numerical routine: {0}")]
    InvalidArgument(String),

    /// The requested system shape/kind isn't supported by `solve`.
    #[error("unsupported system: {0}")]
    Unsupported(String),

    /// An iterative solver (`solve_cg`/`solve_gmres`) or an
    /// iterative eigensolver (`smallest_eigenpairs`/`largest_eigenpairs`)
    /// exhausted its iteration budget without reaching the requested
    /// tolerance.
    #[error("iterative method did not converge within {0} iterations")]
    NotConverged(u32),

    /// A [`crate::random::Distribution`] was constructed with an
    /// out-of-domain parameter (e.g. a negative standard deviation).
    #[error("invalid parameter for distribution: {0}")]
    InvalidDistributionParameter(String),
}
