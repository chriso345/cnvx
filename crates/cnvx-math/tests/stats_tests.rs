use cnvx_math::Vector;
use cnvx_math::stats::{self, Histogram};

#[test]
fn quantiles_bracket_the_median() {
    let data = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    let q25 = stats::quantile(&data, 0.25);
    let q50 = stats::quantile(&data, 0.5);
    let q75 = stats::quantile(&data, 0.75);
    assert!(q25 < q50);
    assert!(q50 < q75);
    assert!((q50 - stats::median(&data)).abs() < 1e-12);
}

#[test]
fn correlation_matrix_diagonal_is_all_ones() {
    let obs = vec![
        Vector::from_slice(&[1.0, 2.0, 3.0]),
        Vector::from_slice(&[2.0, 1.0, 4.0]),
        Vector::from_slice(&[3.0, 5.0, 2.0]),
        Vector::from_slice(&[4.0, 2.0, 6.0]),
    ];
    let corr = stats::correlation_matrix(&obs);
    for i in 0..3 {
        assert!((corr.get(i, i) - 1.0).abs() < 1e-9);
    }
}

#[test]
fn histogram_edges_span_the_data_range() {
    let data = Vector::from_slice(&[-5.0, -2.0, 0.0, 3.0, 10.0]);
    let hist = Histogram::new(&data, 4);
    assert!((hist.edges[0] - (-5.0)).abs() < 1e-9);
    assert!((*hist.edges.last().unwrap() - 10.0).abs() < 1e-9);
    assert_eq!(hist.counts.iter().sum::<usize>(), 5);
}
