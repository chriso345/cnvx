//! # CNVX Math
//!
//! Linear algebra utilities for LP solvers and numerical algorithms.
//! Provides native matrix types and traits used in simplex computations and
//! other numerical routines without relying on external heavy dependencies.
//!
//! # Modules
//!
//! - [`matrix`]: Defines [`DenseMatrix`] and the [`Matrix`] trait for linear algebra operations.

pub mod matrix;

pub use matrix::{DenseMatrix, Matrix, SparseMatrix};
