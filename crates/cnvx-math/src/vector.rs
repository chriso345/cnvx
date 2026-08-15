//! [`Vector`]: a thin, owned wrapper around `Vec<f64>`.

use std::iter::FromIterator;
use std::ops::{Add, Deref, Index, Mul, Neg, Sub};

/// A dense vector.
///
/// `Vector` is a thin wrapper around `Vec<f64>`, not a generic
/// linear-algebra type: it exists so [`crate::Matrix`] and
/// [`crate::SparseMatrix`] have a single, consistent right-hand-side /
/// result type across the crate.
///
/// Arithmetic operators (`+`, `-`, unary `-`, `* f64`) and elementwise
/// combination panic on a dimension mismatch rather than returning a
/// `Result`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector {
    data: Vec<f64>,
}

impl Vector {
    /// Creates a vector of `n` zeros.
    pub fn zeros(n: usize) -> Self {
        Self { data: vec![0.0; n] }
    }

    /// Creates a vector by copying `data`.
    pub fn from_slice(data: &[f64]) -> Self {
        Self { data: data.to_vec() }
    }

    /// Number of elements in the vector.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the vector has no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Dot product with another vector.
    ///
    /// # Panics
    /// Panics if `self.len() != other.len()`.
    pub fn dot(&self, other: &Vector) -> f64 {
        assert_eq!(self.len(), other.len(), "Vector::dot: dimension mismatch");
        self.data.iter().zip(other.data.iter()).map(|(a, b)| a * b).sum()
    }

    /// Euclidean (L2) norm.
    pub fn norm(&self) -> f64 {
        self.data.iter().map(|&x| x * x).sum::<f64>().sqrt()
    }

    /// Infinity norm: the maximum absolute element.
    pub fn norm_inf(&self) -> f64 {
        self.data.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()))
    }
}

impl Deref for Vector {
    type Target = [f64];

    fn deref(&self) -> &[f64] {
        &self.data
    }
}

impl Index<usize> for Vector {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        &self.data[i]
    }
}

impl From<Vec<f64>> for Vector {
    fn from(data: Vec<f64>) -> Self {
        Self { data }
    }
}

impl From<&[f64]> for Vector {
    fn from(data: &[f64]) -> Self {
        Self { data: data.to_vec() }
    }
}

impl FromIterator<f64> for Vector {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self { data: iter.into_iter().collect() }
    }
}

impl Add for &Vector {
    type Output = Vector;

    fn add(self, rhs: &Vector) -> Vector {
        assert_eq!(self.len(), rhs.len(), "Vector addition: dimension mismatch");
        self.data.iter().zip(&rhs.data).map(|(a, b)| a + b).collect()
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        &self + &rhs
    }
}

impl Sub for &Vector {
    type Output = Vector;

    fn sub(self, rhs: &Vector) -> Vector {
        assert_eq!(self.len(), rhs.len(), "Vector subtraction: dimension mismatch");
        self.data.iter().zip(&rhs.data).map(|(a, b)| a - b).collect()
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        &self - &rhs
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        self.data.iter().map(|&x| -x).collect()
    }
}

impl Neg for &Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        self.data.iter().map(|&x| -x).collect()
    }
}

impl Mul<f64> for &Vector {
    type Output = Vector;

    fn mul(self, scalar: f64) -> Vector {
        self.data.iter().map(|&x| x * scalar).collect()
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, scalar: f64) -> Vector {
        &self * scalar
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        rhs * self
    }
}

impl Mul<&Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: &Vector) -> Vector {
        rhs * self
    }
}
