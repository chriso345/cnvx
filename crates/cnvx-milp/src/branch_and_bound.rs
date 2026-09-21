use std::collections::HashMap;

use cnvx_core::{
    Bounds, CnvxError, ConstraintKind, Expression, Model, Sense, Solve, Status, Var,
    VarKind, sum,
};
use cnvx_lp::LpSolver;

use crate::solution::MilpSolution;
use crate::solver::MilpSolver;

pub(crate) fn solve(
    model: &Model,
    solver: &MilpSolver,
) -> Result<MilpSolution, CnvxError> {
    let integer_vars: Vec<Var> = model
        .vars()
        .filter(|&v| matches!(model.kind(v), Ok(VarKind::Integer) | Ok(VarKind::Binary)))
        .collect();

    let sense = model.sense();
    let mut incumbent: Option<(f64, HashMap<Var, f64>)> = None;
    let mut root_bound: Option<f64> = None;
    let mut nodes_explored: u64 = 0;
    let mut stack: Vec<HashMap<Var, Bounds>> = vec![HashMap::new()];

    while let Some(overrides) = stack.pop() {
        nodes_explored += 1;
        if nodes_explored > solver.node_limit {
            return Ok(finish(
                Status::IterationLimit,
                incumbent,
                root_bound,
                nodes_explored,
            ));
        }

        let is_root = overrides.is_empty();
        let (relaxation, var_map) = build_relaxation(model, &overrides)?;
        let lp_solution = relaxation
            .solve(&LpSolver::primal_simplex().tolerance(solver.lp_tolerance))?;

        if is_root && lp_solution.status == Status::Unbounded {
            return Ok(MilpSolution {
                status: Status::Unbounded,
                objective: f64::NAN,
                bound: f64::NAN,
                nodes_explored,
                values: Vec::new(),
            });
        }
        if lp_solution.status != Status::Optimal {
            continue;
        }
        if is_root {
            root_bound = Some(lp_solution.objective);
        }

        if let Some((incumbent_objective, _)) = &incumbent
            && !improves(
                sense,
                lp_solution.objective,
                *incumbent_objective,
                solver.mip_gap,
            )
        {
            continue;
        }

        let mut branch_var: Option<(Var, f64, f64)> = None; // (var, value, |fractional part - 0.5|)
        for &v in &integer_vars {
            let value = lp_solution.value(var_map[&v]);
            let fraction = (value - value.round()).abs();
            if fraction > solver.integrality_tolerance {
                let distance_from_half = (fraction - 0.5).abs();
                if branch_var.is_none_or(|(_, _, best)| distance_from_half < best) {
                    branch_var = Some((v, value, distance_from_half));
                }
            }
        }

        match branch_var {
            None => {
                // Every integer-kind variable is already integral: a candidate
                // solution.
                let values: HashMap<Var, f64> = model
                    .vars()
                    .map(|v| {
                        let raw = lp_solution.value(var_map[&v]);
                        let value =
                            if integer_vars.contains(&v) { raw.round() } else { raw };
                        (v, value)
                    })
                    .collect();
                let is_new_best = incumbent.as_ref().is_none_or(|(objective, _)| {
                    is_strictly_better(sense, lp_solution.objective, *objective)
                });
                if is_new_best {
                    incumbent = Some((lp_solution.objective, values));
                }
            }
            Some((v, value, _)) => {
                let current = match overrides.get(&v) {
                    Some(&bounds) => bounds,
                    None => model.bounds(v)?,
                };
                let mut down = overrides.clone();
                down.insert(v, Bounds::new(current.lower, value.floor()));
                let mut up = overrides.clone();
                up.insert(v, Bounds::new(value.ceil(), current.upper));
                stack.push(down);
                stack.push(up);
            }
        }
    }

    let status = if incumbent.is_some() { Status::Optimal } else { Status::Infeasible };
    Ok(finish(status, incumbent, root_bound, nodes_explored))
}

fn finish(
    status: Status,
    incumbent: Option<(f64, HashMap<Var, f64>)>,
    root_bound: Option<f64>,
    nodes_explored: u64,
) -> MilpSolution {
    match incumbent {
        Some((objective, values)) => MilpSolution {
            status,
            objective,
            bound: root_bound.unwrap_or(objective),
            nodes_explored,
            values: values.into_iter().collect(),
        },
        None => MilpSolution {
            status,
            objective: f64::NAN,
            bound: root_bound.unwrap_or(f64::NAN),
            nodes_explored,
            values: Vec::new(),
        },
    }
}

/// Whether `candidate` could still beat `incumbent` by more than the
/// relative `mip_gap`, given `sense`. Used to decide whether a node's
/// relaxation bound is still worth branching on.
fn improves(sense: Sense, candidate: f64, incumbent: f64, mip_gap: f64) -> bool {
    let threshold = incumbent.abs().max(1.0) * mip_gap;
    match sense {
        Sense::Minimize => candidate < incumbent - threshold,
        Sense::Maximize => candidate > incumbent + threshold,
    }
}

/// Whether `candidate` is strictly better than `incumbent`, given
/// `sense`. Used to decide whether a newly found integral solution
/// replaces the current incumbent.
fn is_strictly_better(sense: Sense, candidate: f64, incumbent: f64) -> bool {
    match sense {
        Sense::Minimize => candidate < incumbent,
        Sense::Maximize => candidate > incumbent,
    }
}

/// Builds a fresh `Model` equivalent to `model`, except that every
/// variable's kind becomes continuous (the LP relaxation) and each
/// variable named in `overrides` gets those bounds instead of its
/// original ones. Returns the new model alongside a map from `model`'s
/// variable handles to the new model's.
fn build_relaxation(
    model: &Model,
    overrides: &HashMap<Var, Bounds>,
) -> Result<(Model, HashMap<Var, Var>), CnvxError> {
    let mut relaxation = Model::new(format!("{}_relaxation", model.name()));
    let mut var_map: HashMap<Var, Var> = HashMap::with_capacity(model.num_vars());

    for v in model.vars() {
        let bounds = match overrides.get(&v) {
            Some(&bounds) => bounds,
            None => model.bounds(v)?,
        };
        let name = model.var_name(v)?.unwrap_or("");
        var_map.insert(v, relaxation.add_named_var(bounds, name));
    }

    relaxation.set_objective(model.sense(), remap(model.objective(), &var_map))?;

    for (_, constraint) in model.constraints() {
        let expr = remap(constraint.expr(), &var_map);
        let remapped = match constraint.kind() {
            ConstraintKind::Leq(rhs) => expr.leq(rhs),
            ConstraintKind::Geq(rhs) => expr.geq(rhs),
            ConstraintKind::Eq(rhs) => expr.eq(rhs),
            ConstraintKind::Ranged { lower, upper } => expr.between(lower, upper),
        };
        let named = match constraint.name() {
            Some(name) => remapped.named(name),
            None => remapped,
        };
        relaxation.add_constraint(named)?;
    }

    Ok((relaxation, var_map))
}

/// Rewrites `expr` (which references `model`'s variables) in terms of the
/// corresponding variables in a rebuilt model, via `var_map`.
fn remap(expr: &Expression, var_map: &HashMap<Var, Var>) -> Expression {
    sum(expr.terms().map(|(v, c)| c * var_map[&v])) + expr.constant_term()
}
