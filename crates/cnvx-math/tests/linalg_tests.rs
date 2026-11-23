use cnvx_math::Matrix;

fn spd_3x3() -> Matrix {
    let mut m = Matrix::zeros(3, 3);
    m.set(0, 0, 4.0);
    m.set(0, 1, 1.0);
    m.set(0, 2, 0.0);
    m.set(1, 0, 1.0);
    m.set(1, 1, 3.0);
    m.set(1, 2, 1.0);
    m.set(2, 0, 0.0);
    m.set(2, 1, 1.0);
    m.set(2, 2, 2.0);
    m
}

fn general_3x3() -> Matrix {
    let mut m = Matrix::zeros(3, 3);
    m.set(0, 0, 2.0);
    m.set(0, 1, -1.0);
    m.set(0, 2, 0.0);
    m.set(1, 0, -1.0);
    m.set(1, 1, 2.0);
    m.set(1, 2, -1.0);
    m.set(2, 0, 3.0);
    m.set(2, 1, 1.0);
    m.set(2, 2, 4.0);
    m
}

#[test]
fn lu_reconstructs_original_matrix() {
    let a = general_3x3();
    let lu = a.lu().unwrap();
    let reconstructed = lu.l.mul(&lu.u).unwrap();
    // reconstructed == P * a, so compare against a permuted by lu.p.
    for (i, &pi) in lu.p.iter().take(3).enumerate() {
        for j in 0..3 {
            assert!((reconstructed.get(i, j) - a.get(pi, j)).abs() < 1e-9);
        }
    }
}

#[test]
fn qr_reconstructs_original_and_q_is_orthonormal() {
    let a = general_3x3();
    let qr = a.qr().unwrap();
    let reconstructed = qr.q.mul(&qr.r).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!((reconstructed.get(i, j) - a.get(i, j)).abs() < 1e-9);
        }
    }
    let qtq = qr.q.transpose().mul(&qr.q).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!((qtq.get(i, j) - expected).abs() < 1e-9);
        }
    }
}

#[test]
fn cholesky_reconstructs_spd_matrix() {
    let a = spd_3x3();
    let chol = a.cholesky().unwrap();
    let reconstructed = chol.l.mul(&chol.l.transpose()).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!((reconstructed.get(i, j) - a.get(i, j)).abs() < 1e-9);
        }
    }
}

#[test]
fn cholesky_rejects_non_positive_definite() {
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 1.0); // Not PD: eigenvalues are -1 and 3.
    assert!(a.cholesky().is_err());
}

#[test]
fn eigen_symmetric_reconstructs_matrix() {
    let a = spd_3x3();
    let eig = a.eigen_symmetric().unwrap();
    // A V = V diag(values)
    for col in 0..3 {
        let v = eig.vectors.get_col(col);
        let av = a.mul_vec(&v);
        for i in 0..3 {
            assert!((av[i] - eig.values[col] * v[i]).abs() < 1e-8);
        }
    }
    // All eigenvalues of a diagonally-dominant SPD matrix are positive.
    for &val in eig.values.iter() {
        assert!(val > 0.0);
    }
}

#[test]
fn eigen_general_finds_known_real_eigenvalues() {
    // Upper triangular matrix: eigenvalues are exactly the diagonal.
    let mut a = Matrix::zeros(3, 3);
    a.set(0, 0, 2.0);
    a.set(0, 1, 5.0);
    a.set(1, 1, 3.0);
    a.set(1, 2, 7.0);
    a.set(2, 2, 4.0);
    let eig = a.eigen_general().unwrap();
    let mut values: Vec<f64> = eig.values.iter().copied().collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((values[0] - 2.0).abs() < 1e-6);
    assert!((values[1] - 3.0).abs() < 1e-6);
    assert!((values[2] - 4.0).abs() < 1e-6);
}

#[test]
fn eigen_general_finds_complex_pair() {
    // A 2x2 rotation-like matrix [[0, -1], [1, 0]] has eigenvalues +-i.
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 1, -1.0);
    a.set(1, 0, 1.0);
    let eig = a.eigen_general().unwrap();
    assert!(eig.imag_values.is_some());
    let imag = eig.imag_values.unwrap();
    assert!((imag[0].abs() - 1.0).abs() < 1e-6);
    assert!((eig.values[0]).abs() < 1e-6);
}

#[test]
fn svd_reconstructs_matrix() {
    let a = general_3x3();
    let svd = a.svd().unwrap();
    let mut s_mat = Matrix::zeros(3, 3);
    for i in 0..3 {
        s_mat.set(i, i, svd.s[i]);
    }
    let reconstructed = svd.u.mul(&s_mat).unwrap().mul(&svd.vt).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            assert!((reconstructed.get(i, j) - a.get(i, j)).abs() < 1e-8);
        }
    }
    // Singular values are sorted descending.
    assert!(svd.s[0] >= svd.s[1]);
    assert!(svd.s[1] >= svd.s[2]);
}

#[test]
fn determinant_matches_known_value() {
    let a = general_3x3();
    // det([[2,-1,0],[-1,2,-1],[3,1,4]]) computed by cofactor expansion:
    // 2*(2*4 - (-1)*1) - (-1)*(-1*4 - (-1)*3) + 0
    // = 2*(8+1) + 1*(-4+3) = 18 - 1 = 17
    let det = a.determinant().unwrap();
    assert!((det - 17.0).abs() < 1e-8, "det = {det}");
}

#[test]
fn determinant_of_singular_matrix_is_zero() {
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 4.0); // row 2 = 2 * row 1
    let det = a.determinant().unwrap();
    assert!(det.abs() < 1e-9, "det = {det}");
}

#[test]
fn rank_and_condition_number_of_identity() {
    let a = Matrix::identity(4);
    assert_eq!(a.rank(1e-10).unwrap(), 4);
    let cond = a.condition_number().unwrap();
    assert!((cond - 1.0).abs() < 1e-9);
}

#[test]
fn rank_detects_deficiency() {
    let mut a = Matrix::zeros(3, 3);
    a.set(0, 0, 1.0);
    a.set(1, 1, 1.0);
    // Row/col 2 is entirely zero => rank 2.
    assert_eq!(a.rank(1e-9).unwrap(), 2);
}

#[test]
fn pseudo_inverse_of_invertible_matrix_matches_inverse_behavior() {
    let a = spd_3x3();
    let pinv = a.pseudo_inverse().unwrap();
    let product = a.mul(&pinv).unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!((product.get(i, j) - expected).abs() < 1e-8);
        }
    }
}

#[test]
fn least_squares_fits_overdetermined_system() {
    // Fit y = a*x + b to noisy-free points on a known line: y = 2x + 1.
    let mut a = Matrix::zeros(4, 2);
    let xs = [0.0, 1.0, 2.0, 3.0];
    for (i, &x) in xs.iter().enumerate() {
        a.set(i, 0, x);
        a.set(i, 1, 1.0);
    }
    let y = cnvx_math::Vector::from_slice(&[1.0, 3.0, 5.0, 7.0]);
    let coeffs = a.least_squares(&y).unwrap();
    assert!((coeffs[0] - 2.0).abs() < 1e-8);
    assert!((coeffs[1] - 1.0).abs() < 1e-8);
}

#[test]
fn matrix_ergonomics() {
    let a = spd_3x3();
    assert!((a.trace() - 9.0).abs() < 1e-12);
    assert!(a.is_symmetric(1e-12));
    assert!(a.frobenius_norm() > 0.0);
    assert_eq!(a.row(0).len(), 3);
    assert_eq!(a.col(0).len(), 3);
    assert_eq!(a.diag().len(), 3);
    let scaled = a.scale(2.0);
    assert!((scaled.get(0, 0) - 8.0).abs() < 1e-12);
}
