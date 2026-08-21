use std::iter::FromIterator;
use std::ops::{Add, Deref, Index, IndexMut, Mul, Neg, Sub};

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
    crate::elementwise::elementwise_fn_set!();

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

    /// Creates a vector of `n` ones.
    pub fn ones(n: usize) -> Self {
        Self { data: vec![1.0; n] }
    }

    /// Creates a vector of `n` evenly spaced values from `start` to `end`
    /// inclusive.
    ///
    /// Returns an empty vector if `n == 0`, and `[start]` if `n == 1`
    /// (matching NumPy's `linspace` conventions).
    pub fn linspace(start: f64, end: f64, n: usize) -> Self {
        if n == 0 {
            return Self::zeros(0);
        }
        if n == 1 {
            return Self { data: vec![start] };
        }
        let step = (end - start) / (n - 1) as f64;
        (0..n).map(|i| start + step * i as f64).collect()
    }

    /// The general `p`-norm: `(sum(|x_i|^p))^(1/p)`.
    ///
    /// `p = 2.0` matches [`Vector::norm`]; `p = f64::INFINITY` matches
    /// [`Vector::norm_inf`].
    pub fn norm_p(&self, p: f64) -> f64 {
        if p.is_infinite() {
            return self.norm_inf();
        }
        self.data.iter().map(|x| x.abs().powf(p)).sum::<f64>().powf(1.0 / p)
    }

    /// Sum of all elements.
    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }

    /// Arithmetic mean of all elements. Thin wrapper over
    /// [`crate::stats::mean`].
    ///
    /// # Panics
    /// Panics if the vector is empty.
    pub fn mean(&self) -> f64 {
        crate::stats::mean(self)
    }

    /// The largest element.
    ///
    /// # Panics
    /// Panics if the vector is empty.
    pub fn max(&self) -> f64 {
        self.data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    /// The smallest element.
    ///
    /// # Panics
    /// Panics if the vector is empty.
    pub fn min(&self) -> f64 {
        self.data.iter().copied().fold(f64::INFINITY, f64::min)
    }

    /// The index of the largest element. Ties return the first occurrence.
    ///
    /// # Panics
    /// Panics if the vector is empty.
    pub fn argmax(&self) -> usize {
        assert!(!self.data.is_empty(), "Vector::argmax: empty vector");
        let mut best = 0;
        for i in 1..self.data.len() {
            if self.data[i] > self.data[best] {
                best = i;
            }
        }
        best
    }

    /// The index of the smallest element. Ties return the first occurrence.
    ///
    /// # Panics
    /// Panics if the vector is empty.
    pub fn argmin(&self) -> usize {
        assert!(!self.data.is_empty(), "Vector::argmin: empty vector");
        let mut best = 0;
        for i in 1..self.data.len() {
            if self.data[i] < self.data[best] {
                best = i;
            }
        }
        best
    }

    /// Elementwise Pareto dominance: `self` is no worse than `other` in
    /// every component, and strictly better in at least one.
    ///
    /// This is the routine multi-criteria/multi-objective search (e.g.
    /// Pareto-optimal shortest paths, Pareto fronts in evolutionary search)
    /// is built on: lower is assumed better in every component, matching
    /// the usual "minimize every objective" convention.
    ///
    /// # Panics
    /// Panics if `self.len() != other.len()`.
    ///
    /// # Examples
    /// ```
    /// use cnvx_math::Vector;
    ///
    /// let a = Vector::from_slice(&[1.0, 2.0]);
    /// let b = Vector::from_slice(&[1.0, 3.0]);
    /// assert!(a.dominates(&b));
    /// assert!(!b.dominates(&a));
    /// ```
    pub fn dominates(&self, other: &Vector) -> bool {
        assert_eq!(self.len(), other.len(), "Vector::dominates: dimension mismatch");
        let mut strictly_better = false;
        for (a, b) in self.data.iter().zip(&other.data) {
            if a > b {
                return false;
            }
            if a < b {
                strictly_better = true;
            }
        }
        strictly_better
    }

    /// Like [`Vector::dominates`], but also returns `true` when `self` and
    /// `other` are equal in every component (weak dominance).
    ///
    /// # Panics
    /// Panics if `self.len() != other.len()`.
    pub fn dominates_or_equal(&self, other: &Vector) -> bool {
        assert_eq!(
            self.len(),
            other.len(),
            "Vector::dominates_or_equal: dimension mismatch"
        );
        self.data.iter().zip(&other.data).all(|(a, b)| a <= b)
    }

    /// Epsilon-tolerant elementwise equality: `true` if every pair of
    /// corresponding elements differs by no more than `tol`.
    ///
    /// Useful for dominance/correctness checks where exact floating-point
    /// equality is too strict (e.g. after arithmetic that accumulates
    /// rounding error).
    ///
    /// # Panics
    /// Panics if `self.len() != other.len()`.
    pub fn approx_eq(&self, other: &Vector, tol: f64) -> bool {
        assert_eq!(self.len(), other.len(), "Vector::approx_eq: dimension mismatch");
        self.data.iter().zip(&other.data).all(|(a, b)| (a - b).abs() <= tol)
    }

    /// Applies `f` to every element, producing a new vector.
    pub fn map(&self, f: impl Fn(f64) -> f64) -> Vector {
        self.data.iter().map(|&x| f(x)).collect()
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

impl IndexMut<usize> for Vector {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        &mut self.data[i]
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
