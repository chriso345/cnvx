use cnvx_math::Vector;

#[test]
fn zeros_and_from_slice() {
    let z = Vector::zeros(3);
    assert_eq!(z.len(), 3);
    assert_eq!(z.to_vec(), vec![0.0, 0.0, 0.0]);

    let v = Vector::from_slice(&[1.0, 2.0, 3.0]);
    assert_eq!(v.len(), 3);
    assert_eq!(v[1], 2.0);
}

#[test]
fn deref_gives_slice_methods() {
    let v = Vector::from_slice(&[3.0, 1.0, 2.0]);
    assert_eq!(v.iter().sum::<f64>(), 6.0);
    assert_eq!(v.to_vec(), vec![3.0, 1.0, 2.0]);
}

#[test]
fn dot_and_norms() {
    let a = Vector::from_slice(&[1.0, 2.0, 3.0]);
    let b = Vector::from_slice(&[4.0, 5.0, 6.0]);
    assert_eq!(a.dot(&b), 32.0);

    let v = Vector::from_slice(&[3.0, -4.0, 0.0]);
    assert_eq!(v.norm(), 5.0);
    assert_eq!(v.norm_inf(), 4.0);
}

#[test]
fn arithmetic_operators() {
    let a = Vector::from_slice(&[1.0, 2.0, 3.0]);
    let b = Vector::from_slice(&[4.0, 5.0, 6.0]);

    assert_eq!((&a + &b).to_vec(), vec![5.0, 7.0, 9.0]);
    assert_eq!((&b - &a).to_vec(), vec![3.0, 3.0, 3.0]);
    assert_eq!((-a.clone()).to_vec(), vec![-1.0, -2.0, -3.0]);
    assert_eq!((&a * 2.0).to_vec(), vec![2.0, 4.0, 6.0]);
    assert_eq!((2.0 * a.clone()).to_vec(), vec![2.0, 4.0, 6.0]);
}

#[test]
fn from_conversions() {
    let v: Vector = vec![1.0, 2.0].into();
    assert_eq!(v.len(), 2);

    let slice: &[f64] = &[7.0, 8.0, 9.0];
    let v2: Vector = slice.into();
    assert_eq!(v2.to_vec(), vec![7.0, 8.0, 9.0]);

    let v3: Vector = (0..4).map(|i| i as f64).collect();
    assert_eq!(v3.to_vec(), vec![0.0, 1.0, 2.0, 3.0]);
}

#[test]
#[should_panic]
fn dot_mismatch_panics() {
    let a = Vector::from_slice(&[1.0, 2.0]);
    let b = Vector::from_slice(&[1.0, 2.0, 3.0]);
    a.dot(&b);
}

#[test]
#[should_panic]
fn add_mismatch_panics() {
    let a = Vector::from_slice(&[1.0, 2.0]);
    let b = Vector::from_slice(&[1.0, 2.0, 3.0]);
    let _ = &a + &b;
}
