use cnvx_core::{CnvxError, Expression, Model, ObjectiveId, Sense, Status};

use crate::solution::MopSolution;
use crate::solver::MopSolver;
use crate::util::{evaluate_expr, solve_single_point, weighted_sum_expr};

fn tier_objective(
    model: &Model,
    ids: &[ObjectiveId],
) -> Result<(Sense, Expression), CnvxError> {
    if let [id] = ids {
        let objective = model.objective_at(*id)?;
        return Ok((objective.sense, objective.expr));
    }

    let weights: Vec<(ObjectiveId, f64)> = ids.iter().map(|&id| (id, 1.0)).collect();
    Ok((Sense::Minimize, weighted_sum_expr(model, &weights)?))
}

/// Groups every objective on `model` into tiers by priority, highest
/// priority first (see [`crate::MopMethod::Lexicographic`]).
fn tiers_by_priority(model: &Model) -> Vec<(i32, Vec<ObjectiveId>)> {
    let mut tiers: Vec<(i32, Vec<ObjectiveId>)> = Vec::new();
    for (id, objective) in model.objectives() {
        match tiers.iter_mut().find(|(priority, _)| *priority == objective.priority) {
            Some((_, ids)) => ids.push(id),
            None => tiers.push((objective.priority, vec![id])),
        }
    }
    tiers.sort_by_key(|a| std::cmp::Reverse(a.0));
    tiers
}

/// Implements [`crate::MopMethod::Lexicographic`].
pub(crate) fn solve(model: &Model, mop: &MopSolver) -> Result<MopSolution, CnvxError> {
    let tiers = tiers_by_priority(model);

    let mut fixed_constraints: Vec<Expression> = Vec::new();
    let mut fixed_ranges: Vec<(f64, f64)> = Vec::new();
    let mut last: Option<MopSolution> = None;

    for (_, ids) in &tiers {
        let (sense, expr) = tier_objective(model, ids)?;

        let mut tier_model = model.with_objective(sense, expr.clone())?;
        for (fixed_expr, &(lower, upper)) in fixed_constraints.iter().zip(&fixed_ranges) {
            tier_model.add_constraint(fixed_expr.clone().between(lower, upper))?;
        }

        let solved = solve_single_point(&tier_model, model, mop)?;
        let MopSolution::Single { status, point } = &solved else {
            unreachable!("solve_single_point always returns MopSolution::Single")
        };

        if *status != Status::Optimal {
            return Ok(solved);
        }

        let tier_value = evaluate_expr(&expr, |v| point.value(v));
        let tolerance = mop.tolerance;
        fixed_constraints.push(expr);
        fixed_ranges.push((tier_value - tolerance, tier_value + tolerance));

        last = Some(solved);
    }

    Ok(last.expect(
        "a model always has at least the primary objective, so tiers is never empty",
    ))
}
