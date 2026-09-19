//! Random number generation and probability distributions.
//!
//! See [`Rng`] for the underlying generator and [`Distribution`] for the
//! common sampling interface implemented by [`Uniform`], [`Normal`],
//! [`Exponential`], [`Bernoulli`], [`Poisson`], and [`Categorical`].
//! [`MultivariateNormal`] samples a [`crate::Vector`] rather than a scalar
//! and so is exposed separately.

mod distribution;
mod rng;

pub use distribution::{
    Bernoulli, Categorical, Distribution, Exponential, MultivariateNormal, Normal,
    Poisson, Uniform,
};
pub use rng::Rng;
