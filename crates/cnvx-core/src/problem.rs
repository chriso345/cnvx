//! # Problem Trait
//!
//! Defines the [`Problem`] trait, the core abstraction for every optimization
//! problem in CNVX, along with [`ProblemKind`] tags and the [`ProblemMut`]
//! extension for solvers that mutate the problem (e.g. cut-adding).
//!
//! ## Design Principles
//!
//! `Problem` is deliberately minimal and dyn-safe.  It captures only the
//! structural metadata that every optimization problem must expose: what *kind*
//! of problem it is, how many variables and constraints it has, and whether it
//! has a defined objective.  Solvers that need richer access downcast via
//! [`as_any`](Problem::as_any) to the concrete type (e.g. [`Model`](crate::Model)).

use std::any::Any;

/// A short, lowercase tag that uniquely identifies a *kind* of optimization problem.
///
/// Tags are plain `&'static str` values so they can be used in `match` arms,
/// printed without allocation, and compared with `==` without hashing.
///
/// # Examples
///
/// ```rust
/// use cnvx_core::problem::ProblemKind;
///
/// const LP: ProblemKind = "lp";
/// const MIP: ProblemKind = "mip";
/// ```
pub type ProblemKind = &'static str;

/// The central abstraction for all optimization problems.
///
/// Every concrete problem type (e.g. [`Model`](crate::Model) for LP/MIP) must
/// implement this trait.  The trait is deliberately minimal and dyn-safe, so
/// `&dyn Problem` and `Box<dyn Problem>` work without knowing the concrete type.
///
/// ## Obtaining Richer Access
///
/// When a solver needs domain-specific information beyond what `Problem` exposes,
/// it downcasts via [`as_any`](Self::as_any):
///
/// ```rust,ignore
/// let model = problem
///     .as_any()
///     .downcast_ref::<Model>()
///     .expect("expected an LP/MIP Model");
/// ```
///
/// Sub-crates may also define intermediate domain traits (e.g. `LpProblem`) and
/// require `problem` to implement those, keeping downcasting to a minimum.
///
/// ## Dyn Safety
///
/// All methods are dyn-safe. There are no generic parameters and the only
/// `where Self: Sized` restriction is on the blanket helpers intentionally
/// excluded from the vtable.
///
/// # Examples
///
/// ```rust
/// use cnvx_core::{Model, problem::Problem};
///
/// let model = Model::new();
/// let p: &dyn Problem = &model;
///
/// assert_eq!(p.kind(), "lp");
/// assert_eq!(p.num_vars(), 0);
/// println!("{}", p.describe());
/// ```
pub trait Problem: Any + Send + Sync {
    /// Returns the [`ProblemKind`] tag identifying the class of this problem.
    ///
    /// Used by solver implementations in [`Solver::supports`](crate::solver::Solver::supports)
    /// to declare compatibility, and in diagnostics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cnvx_core::{Model, problem::Problem};
    /// let m = Model::new();
    /// assert_eq!((&m as &dyn Problem).kind(), "lp");
    /// ```
    fn kind(&self) -> ProblemKind;

    /// Returns the number of decision variables in the problem.
    fn num_vars(&self) -> usize;

    /// Returns the number of constraints in the problem.
    fn num_constraints(&self) -> usize;

    /// Returns `true` if the problem has a defined objective function.
    fn has_objective(&self) -> bool;

    /// Exposes the concrete type for downcasting.
    ///
    /// ```rust,ignore
    /// let lp = problem.as_any().downcast_ref::<Model>()
    ///     .expect("expected an LP Model");
    /// ```
    fn as_any(&self) -> &dyn Any;

    /// Returns a human-readable summary of the problem for diagnostics.
    ///
    /// The default implementation is sufficient for most types; override for
    /// richer output (e.g. to include the objective name or constraint names).
    fn describe(&self) -> String {
        format!(
            "{} problem ({} vars, {} constraints, objective: {})",
            self.kind(),
            self.num_vars(),
            self.num_constraints(),
            if self.has_objective() { "yes" } else { "no" },
        )
    }
}

/// Extension of [`Problem`] for solvers that need to mutate the problem.
///
/// This should be implemented alongside [`Problem`] when a solver needs write
/// acess (e.g. to add Benders cuts, perform pre-processing, or populate warm-start
/// data).
///
/// # Examples
///
/// ```rust,ignore
/// impl ProblemMut for MyModel {
///     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
/// }
/// ```
pub trait ProblemMut: Problem {
    /// Exposes the concrete type mutably for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
