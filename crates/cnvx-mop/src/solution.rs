use std::ops::Index;

use cnvx_core::{Model, ObjectiveId, Sense, Status, Var};
use cnvx_lp::LpSolution;
use cnvx_milp::MilpSolution;

/// The single-model solution backing one [`ParetoPoint`]
#[derive(Clone, Debug)]
pub enum MopPointSolution {
    /// Solved by [`cnvx_lp::LpSolver`].
    Lp(LpSolution),
    /// Solved by [`cnvx_milp::MilpSolver`].
    Milp(MilpSolution),
}

impl MopPointSolution {
    /// The value of `v` in this solution.
    ///
    /// Returns `0.0` if `v` isn't part of the solved model.
    pub fn value(&self, v: Var) -> f64 {
        match self {
            MopPointSolution::Lp(solution) => solution.value(v),
            MopPointSolution::Milp(solution) => solution.value(v),
        }
    }
}

/// One point on an efficient (Pareto) front, or the single solution
/// returned by a scalarizing method
/// ([`crate::MopMethod::Lexicographic`],
/// [`crate::MopMethod::EpsilonConstraint`],
/// [`crate::MopMethod::WeightedSum`]).
#[derive(Clone, Debug)]
pub struct ParetoPoint {
    objective_values: Vec<(ObjectiveId, f64)>,
    /// The underlying single-objective solution this point came from.
    pub solution: MopPointSolution,
}

impl ParetoPoint {
    pub(crate) fn new(
        objective_values: Vec<(ObjectiveId, f64)>,
        solution: MopPointSolution,
    ) -> Self {
        ParetoPoint { objective_values, solution }
    }

    /// The value of objective `id` at this point.
    ///
    /// Returns `0.0` if `id` isn't one of the objectives this point was
    /// computed for.
    pub fn objective_value(&self, id: ObjectiveId) -> f64 {
        self.objective_values
            .iter()
            .find(|(oid, _)| *oid == id)
            .map(|(_, value)| *value)
            .unwrap_or(0.0)
    }

    /// Iterates over every `(objective, value)` pair at this point, in
    /// [`cnvx_core::Model::objectives`] order.
    pub fn objective_values(&self) -> impl Iterator<Item = (ObjectiveId, f64)> + '_ {
        self.objective_values.iter().copied()
    }

    /// The value of variable `v` in the underlying solution.
    ///
    /// Returns `0.0` if `v` isn't part of the solved model.
    pub fn value(&self, v: Var) -> f64 {
        self.solution.value(v)
    }
}

/// `point[v]` instead of `point.value(v)`.
///
/// # Panics
/// Panics if `v` isn't part of the solved model. Prefer
/// [`ParetoPoint::value`] if that's a possibility.
impl Index<Var> for ParetoPoint {
    type Output = f64;

    fn index(&self, v: Var) -> &f64 {
        match &self.solution {
            MopPointSolution::Lp(solution) => &solution[v],
            MopPointSolution::Milp(solution) => &solution[v],
        }
    }
}

/// `expr`'s value, normalized to a minimization sense.
fn normalized(value: f64, sense: Sense) -> f64 {
    match sense {
        Sense::Minimize => value,
        Sense::Maximize => -value,
    }
}

/// An approximation (or, for [`crate::MopMethod::Biobjective`], the
/// exact set) of the efficient (Pareto) points found for a
/// multi-objective model.
#[derive(Clone, Debug)]
pub struct ParetoFront {
    points: Vec<ParetoPoint>,
    senses: Vec<(ObjectiveId, Sense)>,
}

impl ParetoFront {
    pub(crate) fn new(model: &Model, points: Vec<ParetoPoint>) -> Self {
        let senses = model
            .objectives()
            .map(|(id, objective)| (id, objective.sense))
            .collect();
        ParetoFront { points, senses }
    }

    /// Every point on the front.
    pub fn points(&self) -> &[ParetoPoint] {
        &self.points
    }

    /// The number of points on the front.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether the front has no points (e.g. because the underlying
    /// model was infeasible).
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The point minimizing normalized Euclidean distance to the
    /// front's own ideal point.
    ///
    /// Returns `None` if the front has no points.
    pub fn knee_point(&self) -> Option<&ParetoPoint> {
        if self.points.is_empty() {
            return None;
        }

        let ideal: Vec<f64> = self
            .senses
            .iter()
            .map(|&(id, sense)| {
                self.points
                    .iter()
                    .map(|point| normalized(point.objective_value(id), sense))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect();
        let nadir: Vec<f64> = self
            .senses
            .iter()
            .map(|&(id, sense)| {
                self.points
                    .iter()
                    .map(|point| normalized(point.objective_value(id), sense))
                    .fold(f64::NEG_INFINITY, f64::max)
            })
            .collect();

        let distance_to_ideal = |point: &ParetoPoint| -> f64 {
            self.senses
                .iter()
                .zip(&ideal)
                .zip(&nadir)
                .map(|((&(id, sense), &ideal_value), &nadir_value)| {
                    let value = normalized(point.objective_value(id), sense);
                    let range = (nadir_value - ideal_value).abs().max(1e-12);
                    ((value - ideal_value) / range).powi(2)
                })
                .sum::<f64>()
                .sqrt()
        };

        self.points.iter().min_by(|a, b| {
            distance_to_ideal(a)
                .partial_cmp(&distance_to_ideal(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// The result of solving a [`cnvx_core::Model`] with
/// [`crate::MopSolver`].
#[derive(Clone, Debug)]
pub enum MopSolution {
    /// The result of a scalarizing method
    /// ([`crate::MopMethod::Lexicographic`],
    /// [`crate::MopMethod::EpsilonConstraint`],
    /// [`crate::MopMethod::WeightedSum`]): one point.
    Single {
        /// The outcome of the underlying solve(s).
        status: Status,
        /// The point found, meaningful only if `status` is
        /// [`Status::Optimal`].
        point: ParetoPoint,
    },
    /// The result of a front-tracing method
    /// ([`crate::MopMethod::Biobjective`],
    Front(ParetoFront),
}
