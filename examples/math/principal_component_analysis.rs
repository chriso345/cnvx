//! Principal Component Analysis
//!
//! Demonstrates PCA for dimensionality reduction on a synthetic dataset
//! and the classic Iris dataset (embedded).

use cnvx::math::random::{Distribution, Normal, Rng};
use cnvx::prelude::*;

#[rustfmt::skip]
fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== PCA on Synthetic Correlated Data ===\n");

    // Generate 3D data with strong correlation along first two dimensions
    let mut rng = Rng::new(42);
    let n = 500;
    let normal = Normal::new(0.0, 1.0).unwrap();

    // True latent variables
    let z1: Vector = (0..n).map(|_| normal.sample(&mut rng) * 3.0).collect(); // Large variance
    let z2: Vector = (0..n).map(|_| normal.sample(&mut rng) * 1.0).collect(); // Medium variance
    let z3: Vector = (0..n).map(|_| normal.sample(&mut rng) * 0.3).collect(); // Small variance

    // Mix them: x = A * z
    let mut a = Matrix::zeros(3, 3);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.8);
    a.set(0, 2, 0.2);
    a.set(1, 0, 0.8);
    a.set(1, 1, 1.0);
    a.set(1, 2, 0.1);
    a.set(2, 0, 0.3);
    a.set(2, 1, 0.2);
    a.set(2, 2, 1.0);

    let mut x = Matrix::zeros(3, n);
    for i in 0..n {
        x.set(0, i, a.get(0, 0) * z1[i] + a.get(0, 1) * z2[i] + a.get(0, 2) * z3[i]);
        x.set(1, i, a.get(1, 0) * z1[i] + a.get(1, 1) * z2[i] + a.get(1, 2) * z3[i]);
        x.set(2, i, a.get(2, 0) * z1[i] + a.get(2, 1) * z2[i] + a.get(2, 2) * z3[i]);
    }

    // Center the data (now 3 x n) -> transpose to n x 3 for row-wise data
    let mut data = Matrix::zeros(n, 3);
    for i in 0..n {
        for j in 0..3 {
            data.set(i, j, x.get(j, i));
        }
    }

    // Compute mean manually (no mean_axis method)
    let mut mean = Vector::zeros(3);
    for j in 0..3 {
        let mut sum = 0.0;
        for i in 0..n {
            sum += data.get(i, j);
        }
        mean[j] = sum / n as f64;
    }

    for i in 0..n {
        for j in 0..3 {
            data.set(i, j, data.get(i, j) - mean[j]);
        }
    }

    // Compute covariance: data^T * data / (n-1)
    let data_t = data.transpose();
    let cov = data_t.mul(&data)?.mul_scalar(1.0 / (n - 1) as f64);

    println!("Covariance matrix:");
    for i in 0..3 {
        println!("  [{:.3}, {:.3}, {:.3}]", cov.get(i, 0), cov.get(i, 1), cov.get(i, 2));
    }

    // Eigendecomposition
    let eigen = cov.eigen_symmetric()?;
    println!("\nEigenvalues (variance explained): {:?}", eigen.values);
    println!("Total variance: {:.3}", eigen.values.iter().sum::<f64>());

    // Proportion of variance
    let total: f64 = eigen.values.iter().sum();
    for (i, &val) in eigen.values.iter().enumerate() {
        println!("  PC{}: {:.1}% ({:.3})", i + 1, val / total * 100.0, val);
    }

    // Project to 2D (first 2 PCs)
    let mut pc2 = Matrix::zeros(3, 2);
    for i in 0..3 {
        pc2.set(i, 0, eigen.vectors.get(i, 0));
        pc2.set(i, 1, eigen.vectors.get(i, 1));
    }
    let projected = data.mul(&pc2)?;

    println!("\nProjected data (first 5 points):");
    for i in 0..5.min(n) {
        println!("  [{:.3}, {:.3}]", projected.get(i, 0), projected.get(i, 1));
    }

    // Reconstruction error from 2D
    let pc2_t = pc2.transpose();
    let reconstructed = projected.mul(&pc2_t)?;
    let mut mse = 0.0;
    for i in 0..n {
        for j in 0..3 {
            let diff = reconstructed.get(i, j) - data.get(i, j);
            mse += diff * diff;
        }
    }
    mse /= (n * 3) as f64;
    println!("\nReconstruction MSE (2D -> 3D): {mse:.6}");

    // ===== Iris-like dataset =====
    println!("\n=== PCA on Iris-like Data ===");
    // Simulated Iris: 150 samples, 4 features, 3 classes
    let iris_data = generate_iris_like();
    let (n_samples, n_features) = (iris_data.rows(), iris_data.cols());

    // Center
    let mut mean = Vector::zeros(n_features);
    for j in 0..n_features {
        let mut sum = 0.0;
        for i in 0..n_samples {
            sum += iris_data.get(i, j);
        }
        mean[j] = sum / n_samples as f64;
    }
    let mut centered = iris_data.clone();
    for i in 0..n_samples {
        for j in 0..n_features {
            centered.set(i, j, centered.get(i, j) - mean[j]);
        }
    }

    // Covariance and eigendecomp
    let centered_t = centered.transpose();
    let cov = centered_t.mul(&centered)?.mul_scalar(1.0 / (n_samples - 1) as f64);
    let eigen = cov.eigen_symmetric()?;

    println!("Eigenvalues: {:?}", eigen.values);
    let total: f64 = eigen.values.iter().sum();
    for (i, &val) in eigen.values.iter().enumerate() {
        println!("  PC{}: {:.1}%", i + 1, val / total * 100.0);
    }

    // 2D projection
    let mut pc2 = Matrix::zeros(n_features, 2);
    for i in 0..n_features {
        pc2.set(i, 0, eigen.vectors.get(i, 0));
        pc2.set(i, 1, eigen.vectors.get(i, 1));
    }
    let projected = centered.mul(&pc2)?;

    println!("\n2D projection (first 3 samples per class):");
    for class in 0..3 {
        for i in (class * 50)..(class * 50 + 3) {
            println!(
                "  Class {class}: [{:.2}, {:.2}]",
                projected.get(i, 0),
                projected.get(i, 1)
            );
        }
    }

    // Expected output:
    //
    // === PCA on Synthetic Correlated Data ===
    //
    // Covariance matrix:
    //   [9.239, 7.683, 2.752]
    //   [7.683, 6.510, 2.274]
    //   [2.752, 2.274, 0.890]
    //
    // Eigenvalues (variance explained): Vector { data: [16.494089188679922, 0.09620401279751634, 0.04810276705892071] }
    // Total variance: 16.638
    //   PC1: 99.1% (16.494)
    //   PC2: 0.6% (0.096)
    //   PC3: 0.3% (0.048)
    //
    // Projected data (first 5 points):
    //   [-1.100, 0.232]
    //   [5.292, 0.037]
    //   [0.934, -0.264]
    //   [5.532, 0.344]
    //   [-5.927, -0.057]
    //
    // Reconstruction MSE (2D -> 3D): 0.016002
    //
    // === PCA on Iris-like Data ===
    // Eigenvalues: Vector { data: [4.129951572829542, 0.3257843247841365, 0.13110150651733898, 0.06661411218569556] }
    //   PC1: 88.8%
    //   PC2: 7.0%
    //   PC3: 2.8%
    //   PC4: 1.4%
    //
    // 2D projection (first 3 samples per class):
    //   Class 0: [-2.48, -0.24]
    //   Class 0: [-2.81, 0.45]
    //   Class 0: [-2.55, -0.00]
    //   Class 1: [0.52, -1.50]
    //   Class 1: [0.67, 0.36]
    //   Class 1: [0.79, 0.42]
    //   Class 2: [2.61, -0.22]
    //   Class 2: [1.87, -0.24]
    //   Class 2: [1.47, 1.11]

    Ok(())
}

fn generate_iris_like() -> Matrix {
    let mut rng = Rng::new(123);
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut data = Matrix::zeros(150, 4);

    // Class 0 (setosa-like): small petals, large sepals
    for i in 0..50 {
        data.set(i, 0, 5.0 + 0.35 * normal.sample(&mut rng)); // sepal length
        data.set(i, 1, 3.4 + 0.38 * normal.sample(&mut rng)); // sepal width
        data.set(i, 2, 1.5 + 0.17 * normal.sample(&mut rng)); // petal length
        data.set(i, 3, 0.2 + 0.1 * normal.sample(&mut rng)); // petal width
    }

    // Class 1 (versicolor-like): medium
    for i in 50..100 {
        data.set(i, 0, 5.9 + 0.5 * normal.sample(&mut rng));
        data.set(i, 1, 2.8 + 0.3 * normal.sample(&mut rng));
        data.set(i, 2, 4.3 + 0.5 * normal.sample(&mut rng));
        data.set(i, 3, 1.3 + 0.2 * normal.sample(&mut rng));
    }

    // Class 2 (virginica-like): large
    for i in 100..150 {
        data.set(i, 0, 6.6 + 0.6 * normal.sample(&mut rng));
        data.set(i, 1, 3.0 + 0.3 * normal.sample(&mut rng));
        data.set(i, 2, 5.6 + 0.5 * normal.sample(&mut rng));
        data.set(i, 3, 2.0 + 0.3 * normal.sample(&mut rng));
    }

    data
}
