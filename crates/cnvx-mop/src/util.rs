use cnvx_core::{CnvxError, Expression, Model, ObjectiveId, Sense, Solve, VarKind};

use crate::solution::{MopPointSolution, MopSolution, ParetoPoint};
use crate::solver::MopSolver;

pub(crate) fn is_pure_lp(model: &Model) -> Result<bool, CnvxError> {
    for v in model.vars() {
        if model.kind(v)? != VarKind::Continuous {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn evaluate_expr(
    expr: &Expression,
    value_of: impl Fn(cnvx_core::Var) -> f64,
) -> f64 {
    expr.constant_term() + expr.terms().map(|(v, c)| c * value_of(v)).sum::<f64>()
}

fn normalize_to_min(sense: Sense, expr: &Expression) -> Expression {
    match sense {
        Sense::Minimize => expr.clone(),
        Sense::Maximize => -expr.clone(),
    }
}

pub(crate) fn weighted_sum_expr(
    model: &Model,
    weights: &[(ObjectiveId, f64)],
) -> Result<Expression, CnvxError> {
    let mut total = Expression::zero();
    for &(id, weight) in weights {
        let objective = model.objective_at(id)?;
        total = total + weight * normalize_to_min(objective.sense, &objective.expr);
    }
    Ok(total)
}

pub(crate) fn all_objective_values(
    model: &Model,
    point_solution: &MopPointSolution,
) -> Vec<(ObjectiveId, f64)> {
    model
        .objectives()
        .map(|(id, objective)| {
            (id, evaluate_expr(&objective.expr, |v| point_solution.value(v)))
        })
        .collect()
}

pub(crate) fn solve_single_point(
    working: &Model,
    original: &Model,
    mop: &MopSolver,
) -> Result<MopSolution, CnvxError> {
    let (status, point_solution) = if is_pure_lp(working)? {
        let solution = working.solve(&mop.lp_solver())?;
        (solution.status, MopPointSolution::Lp(solution))
    } else {
        let solution = working.solve(&mop.milp_solver())?;
        (solution.status, MopPointSolution::Milp(solution))
    };

    let objective_values = all_objective_values(original, &point_solution);
    Ok(MopSolution::Single {
        status,
        point: ParetoPoint::new(objective_values, point_solution),
    })
}

pub(crate) fn dedup_points(points: &mut Vec<ParetoPoint>, tolerance: f64) {
    points.dedup_by(|a, b| {
        a.objective_values()
            .zip(b.objective_values())
            .all(|((_, value_a), (_, value_b))| (value_a - value_b).abs() < tolerance)
    });
}

pub(crate) fn filter_nondominated(model: &Model, points: &mut Vec<ParetoPoint>) {
    let senses: Vec<(ObjectiveId, Sense)> = model
        .objectives()
        .map(|(id, objective)| (id, objective.sense))
        .collect();

    let dominates = |a: &ParetoPoint, b: &ParetoPoint| -> bool {
        let mut at_least_as_good_everywhere = true;
        let mut strictly_better_somewhere = false;
        for &(id, sense) in &senses {
            let value_a = a.objective_value(id);
            let value_b = b.objective_value(id);
            let (better, worse) = match sense {
                Sense::Minimize => (value_a < value_b, value_a > value_b),
                Sense::Maximize => (value_a > value_b, value_a < value_b),
            };
            if worse {
                at_least_as_good_everywhere = false;
            }
            if better {
                strictly_better_somewhere = true;
            }
        }
        at_least_as_good_everywhere && strictly_better_somewhere
    };

    let keep: Vec<bool> = (0..points.len())
        .map(|i| !(0..points.len()).any(|j| j != i && dominates(&points[j], &points[i])))
        .collect();
    let mut i = 0;
    points.retain(|_| {
        let keep_this = keep[i];
        i += 1;
        keep_this
    });
}
