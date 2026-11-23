use cnvx_core::{CnvxError, Model, ObjectiveId, Sense, Status};

use crate::solution::{MopSolution, ParetoFront, ParetoPoint};
use crate::solver::MopSolver;
use crate::util::{
    dedup_points, filter_nondominated, is_pure_lp, solve_single_point, weighted_sum_expr,
};

/// The value of objective `id` at `point`.
fn value_for(point: &ParetoPoint, id: ObjectiveId) -> f64 {
    point.objective_value(id)
}

/// Solves the weighted sum `lambda * f(id0) + (1 - lambda) * f(id1)` and
/// returns the resulting point, tracking how many solves have been
/// attempted so far against `mop`'s `max_iterations` budget.
fn solve_weighted(
    model: &Model,
    id0: ObjectiveId,
    id1: ObjectiveId,
    lambda: f64,
    mop: &MopSolver,
    solves: &mut u32,
) -> Result<ParetoPoint, CnvxError> {
    *solves += 1;
    if *solves > mop.max_iterations {
        return Err(CnvxError::Numerical(
            "MopMethod::Biobjective exceeded MopSolver::max_iterations weighted-sum solves \
             without terminating"
                .to_string(),
        ));
    }

    let expr = weighted_sum_expr(model, &[(id0, lambda), (id1, 1.0 - lambda)])?;
    let working = model.with_objective(Sense::Minimize, expr)?;
    match solve_single_point(&working, model, mop)? {
        MopSolution::Single { status: Status::Optimal, point } => Ok(point),
        MopSolution::Single { status, .. } => Err(CnvxError::SolveFailed(status)),
        MopSolution::Front(_) => {
            unreachable!("solve_single_point always returns MopSolution::Single")
        }
    }
}

/// Recursively finds every weighted-sum breakpoint strictly between the
/// two known supported points `p` (at weight `lambda_p`) and `q` (at
/// weight `lambda_q`), appending any new point found to `out`.
#[allow(clippy::too_many_arguments)]
fn bisect(
    model: &Model,
    id0: ObjectiveId,
    id1: ObjectiveId,
    lambda_p: f64,
    p: &ParetoPoint,
    lambda_q: f64,
    q: &ParetoPoint,
    mop: &MopSolver,
    solves: &mut u32,
    out: &mut Vec<ParetoPoint>,
) -> Result<(), CnvxError> {
    let (f1p, f2p) = (value_for(p, id0), value_for(p, id1));
    let (f1q, f2q) = (value_for(q, id0), value_for(q, id1));

    // The weight at which p and q's weighted objective values coincide:
    // lambda * f1p + (1 - lambda) * f2p == lambda * f1q + (1 - lambda) * f2q.
    let denominator = (f1p - f2p) - (f1q - f2q);
    if denominator.abs() < mop.tolerance {
        // p and q are tied at every weight along this segment already;
        // no interior breakpoint to find.
        return Ok(());
    }
    let lambda = (f2q - f2p) / denominator;
    if !(lambda_q + mop.tolerance < lambda && lambda < lambda_p - mop.tolerance) {
        // Not strictly between q and p (can happen from numerical
        // noise right at the endpoints); nothing more to find here.
        return Ok(());
    }

    let r = solve_weighted(model, id0, id1, lambda, mop, solves)?;
    let tied_value = lambda * f1p + (1.0 - lambda) * f2p;
    let r_value = lambda * value_for(&r, id0) + (1.0 - lambda) * value_for(&r, id1);

    if r_value >= tied_value - mop.tolerance {
        // No strictly better vertex at this weight: p and q are
        // adjacent breakpoints, connected by a single face.
        return Ok(());
    }

    out.push(r.clone());
    bisect(model, id0, id1, lambda_p, p, lambda, &r, mop, solves, out)?;
    bisect(model, id0, id1, lambda, &r, lambda_q, q, mop, solves, out)?;
    Ok(())
}

/// Implements [`crate::MopMethod::Biobjective`].
pub(crate) fn solve(model: &Model, mop: &MopSolver) -> Result<MopSolution, CnvxError> {
    if model.num_objectives() != 2 {
        return Err(CnvxError::InvalidArgument(format!(
            "MopMethod::Biobjective requires exactly 2 objectives, model has {}",
            model.num_objectives()
        )));
    }
    if !is_pure_lp(model)? {
        return Err(CnvxError::InvalidArgument(
            "MopMethod::Biobjective only supports models where every variable is \
             VarKind::Continuous"
                .to_string(),
        ));
    }

    let ids: Vec<ObjectiveId> = model.objectives().map(|(id, _)| id).collect();
    let (id0, id1) = (ids[0], ids[1]);

    let mut solves = 0u32;
    let left = solve_weighted(model, id0, id1, 1.0, mop, &mut solves)?;
    let right = solve_weighted(model, id0, id1, 0.0, mop, &mut solves)?;

    let mut points = vec![left.clone(), right.clone()];
    bisect(model, id0, id1, 1.0, &left, 0.0, &right, mop, &mut solves, &mut points)?;

    points.sort_by(|a, b| {
        value_for(a, id0)
            .partial_cmp(&value_for(b, id0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    dedup_points(&mut points, mop.tolerance);
    filter_nondominated(model, &mut points);

    Ok(MopSolution::Front(ParetoFront::new(model, points)))
}
