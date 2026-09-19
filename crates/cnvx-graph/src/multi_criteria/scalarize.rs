use cnvx_math::Vector;

/// Builds a [`WeightFn`](crate::weight::WeightFn)-compatible closure from
/// a multi-criteria accessor and a fixed weight vector: the returned
/// closure computes `criteria(edge).dot(weights)`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::multi_criteria::scalarize_weighted_sum;
/// use cnvx_graph::shortest_path::Dijkstra;
/// use cnvx_math::Vector;
///
/// struct Road {
///     distance: f64,
///     time: f64,
/// }
///
/// let mut g = Graph::<(), Road>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// g.add_edge(a, b, Road { distance: 10.0, time: 2.0 }).unwrap();
///
/// // Minimize `0.7 * distance + 0.3 * time`.
/// let weight = scalarize_weighted_sum(
///     |r: &Road| Vector::from_slice(&[r.distance, r.time]),
///     Vector::from_slice(&[0.7, 0.3]),
/// );
/// let result = (&g).solve(&Dijkstra::new(a).weight(weight)).unwrap();
/// assert_eq!(result.distance_to(b), Some(7.6));
/// ```
pub fn scalarize_weighted_sum<'a, E>(
    criteria: impl Fn(&E) -> Vector + 'a,
    weights: Vector,
) -> impl Fn(&E) -> f64 + 'a {
    move |e: &E| criteria(e).dot(&weights)
}
