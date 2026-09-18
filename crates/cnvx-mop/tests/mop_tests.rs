use cnvx_core::{CnvxError, Model, Sense, Solve, Status};
use cnvx_lp::LpSolver;
use cnvx_milp::MilpSolver;
use cnvx_mop::{MopSolution, MopSolver};

/// A model with a genuine trade-off: `x + y == 10`, `x, y >= 0`,
/// minimize `x` and minimize `y`. Every feasible point is efficient
/// (decreasing one coordinate strictly increases the other), so the
/// only two *extreme* efficient points are `(0, 10)` and `(10, 0)`.
fn split_model() -> (Model, cnvx_core::Var, cnvx_core::Var) {
    let mut model = Model::new("split");
    let x = model.add_var(0.0..);
    let y = model.add_var(0.0..);
    model.add_constraint((x + y).eq(10.0)).unwrap();
    model.set_objective(Sense::Minimize, x.into()).unwrap();
    model.add_objective(Sense::Minimize, y.into(), 0).unwrap();
    (model, x, y)
}

#[test]
fn weighted_sum_matches_manual_scalarization() {
    let (model, x, y) = split_model();

    let solution = model.solve(&MopSolver::weighted_sum(vec![0.3, 0.7])).unwrap();
    let MopSolution::Single { status, point } = solution else {
        panic!("WeightedSum should return MopSolution::Single");
    };
    assert_eq!(status, Status::Optimal);

    // Minimizing 0.3x + 0.7y subject to x + y == 10, x, y >= 0 puts all
    // mass on the cheaper (lower-weight) variable: x = 10, y = 0.
    assert!((point.value(x) - 10.0).abs() < 1e-6);
    assert!((point.value(y) - 0.0).abs() < 1e-6);
}

#[test]
fn lexicographic_respects_priority_order() {
    let mut model = Model::new("lexicographic");
    let x = model.add_var(0.0..=10.0);
    let y = model.add_var(0.0..=10.0);
    model.add_constraint((x + y).leq(10.0)).unwrap();

    // Higher priority is optimized first: minimizing x takes precedence
    // over minimizing -y (i.e. maximizing y).
    model.set_objective(Sense::Minimize, x.into()).unwrap();
    model.add_objective(Sense::Minimize, -y, 1).unwrap();

    let solution = model.solve(&MopSolver::lexicographic()).unwrap();
    let MopSolution::Single { status, point } = solution else {
        panic!("Lexicographic should return MopSolution::Single");
    };
    assert_eq!(status, Status::Optimal);

    // Tier 1 (priority 1, minimize -y i.e. maximize y) is solved first:
    // x can be anything in [0, 10] without affecting it directly, but
    // the constraint x + y <= 10 means y is maximized by x = 0, y = 10.
    // Tier 0 (minimize x) then picks the smallest x consistent with
    // that fixed y = 10, which is x = 0.
    assert!((point.value(x) - 0.0).abs() < 1e-5);
    assert!((point.value(y) - 10.0).abs() < 1e-5);
}

#[test]
fn lexicographic_tied_priorities_combine_into_one_tier() {
    // Two objectives at the same (default) priority behave like an
    // equally weighted sum in one tier.
    let (model, x, y) = split_model();
    let solution = model.solve(&MopSolver::lexicographic()).unwrap();
    let MopSolution::Single { status, point } = solution else {
        panic!("Lexicographic should return MopSolution::Single");
    };
    assert_eq!(status, Status::Optimal);
    // Both objectives have priority 0 (the default), so this is
    // equivalent to minimizing x + y, which is constant (== 10) over
    // the whole feasible line -- any feasible point is optimal.
    assert!((point.value(x) + point.value(y) - 10.0).abs() < 1e-6);
}

#[test]
fn epsilon_constraint_bounds_the_secondary_objective() {
    let (model, x, y) = split_model();
    let ids: Vec<_> = model.objectives().map(|(id, _)| id).collect();
    let (obj_x, obj_y) = (ids[0], ids[1]);

    // Minimize x, subject to y <= 4 (obj_y minimizes, so its bound is
    // an upper bound).
    let solver = MopSolver::epsilon_constraint(obj_x, vec![(obj_y, 4.0)]);
    let solution = model.solve(&solver).unwrap();
    let MopSolution::Single { status, point } = solution else {
        panic!("EpsilonConstraint should return MopSolution::Single");
    };
    assert_eq!(status, Status::Optimal);

    // y <= 4 and x + y == 10 forces x >= 6; minimizing x makes the
    // constraint tight: y = 4, x = 6.
    assert!((point.value(y) - 4.0).abs() < 1e-6);
    assert!((point.value(x) - 6.0).abs() < 1e-6);
}

#[test]
fn biobjective_finds_both_extreme_points() {
    let (model, x, y) = split_model();
    let solution = model.solve(&MopSolver::biobjective()).unwrap();
    let MopSolution::Front(front) = solution else {
        panic!("Biobjective should return MopSolution::Front");
    };

    assert_eq!(
        front.len(),
        2,
        "expected exactly the two extreme points of the split model"
    );

    let has_point = |want_x: f64, want_y: f64| {
        front.points().iter().any(|p| {
            (p.value(x) - want_x).abs() < 1e-5 && (p.value(y) - want_y).abs() < 1e-5
        })
    };
    assert!(has_point(0.0, 10.0), "front should contain (x=0, y=10)");
    assert!(has_point(10.0, 0.0), "front should contain (x=10, y=0)");
}

#[test]
fn biobjective_rejects_wrong_objective_count() {
    let mut model = Model::new("single_objective");
    let x = model.add_var(0.0..);
    model.set_objective(Sense::Minimize, x.into()).unwrap();

    let err = model.solve(&MopSolver::biobjective()).unwrap_err();
    assert!(matches!(err, CnvxError::InvalidArgument(_)));
}

#[test]
fn biobjective_rejects_non_continuous_models() {
    let mut model = Model::new("integer_biobjective");
    let x = model.add_integer(0.0..=10.0, "x");
    let y = model.add_var(0.0..=10.0);
    model.add_constraint((x + y).leq(10.0)).unwrap();
    model.set_objective(Sense::Minimize, x.into()).unwrap();
    model.add_objective(Sense::Minimize, y.into(), 0).unwrap();

    let err = model.solve(&MopSolver::biobjective()).unwrap_err();
    assert!(matches!(err, CnvxError::InvalidArgument(_)));
}

#[test]
fn lp_and_milp_reject_multi_objective_models() {
    let (model, _, _) = split_model();

    let lp_err = model.solve(&LpSolver::default()).unwrap_err();
    assert!(matches!(lp_err, CnvxError::InvalidArgument(_)));

    let milp_err = model.solve(&MilpSolver::default()).unwrap_err();
    assert!(matches!(milp_err, CnvxError::InvalidArgument(_)));
}

#[test]
fn add_objective_rejects_foreign_handles() {
    let mut model_a = Model::new("a");
    let mut model_b = Model::new("b");
    let x_b = model_b.add_var(0.0..);

    let err = model_a.add_objective(Sense::Minimize, x_b.into(), 0).unwrap_err();
    assert!(matches!(err, CnvxError::ForeignHandle));
}

#[test]
fn objective_at_rejects_foreign_ids() {
    let mut model_a = Model::new("a");
    let x_a = model_a.add_var(0.0..);
    model_a.set_objective(Sense::Minimize, x_a.into()).unwrap();
    let id_a = model_a.objectives().next().unwrap().0;

    let model_b = Model::new("b");
    let err = model_b.objective_at(id_a).unwrap_err();
    assert!(matches!(err, CnvxError::ForeignHandle));
}

#[test]
fn with_objective_keeps_original_var_handles_valid() {
    let mut model = Model::new("original");
    let x = model.add_var(0.0..=5.0);
    model.set_objective(Sense::Minimize, x.into()).unwrap();
    let cost = model.add_objective(Sense::Maximize, 2.0 * x, 0).unwrap();

    let cost_expr = model.objective_at(cost).unwrap().expr;
    let single = model.with_objective(Sense::Maximize, cost_expr).unwrap();
    assert_eq!(single.num_objectives(), 1);

    // `x` was minted by `model`, but `single` shares its generation, so
    // it can be used directly on `single`'s solution.
    let solution = single.solve(&LpSolver::default()).unwrap();
    assert!((solution.value(x) - 5.0).abs() < 1e-9);
}

#[test]
fn num_objectives_counts_primary_and_additional() {
    let mut model = Model::new("counting");
    let x = model.add_var(0.0..);
    model.set_objective(Sense::Minimize, x.into()).unwrap();
    assert_eq!(model.num_objectives(), 1);
    assert_eq!(model.num_additional_objectives(), 0);

    model.add_objective(Sense::Maximize, x.into(), 0).unwrap();
    assert_eq!(model.num_objectives(), 2);
    assert_eq!(model.num_additional_objectives(), 1);
}
