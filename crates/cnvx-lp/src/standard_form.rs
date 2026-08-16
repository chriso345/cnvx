use std::collections::HashMap;

use cnvx_core::{Bounds, CnvxError, ConstraintKind, Expression, Model, Sense, Var};
use cnvx_math::{Matrix, Vector};

/// How a bounded quantity (an original variable, or a constraint's
/// implicit comparison slack) maps onto plain nonnegative simplex
/// columns.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Column {
    /// The quantity is pinned to a constant; it gets no simplex column.
    Fixed(f64),
    /// `quantity = shift + sign * y[col]`, with `y[col] >= 0`.
    Single { shift: f64, sign: f64, col: usize },
    /// `quantity = y[pos] - y[neg]`, both `>= 0` (a free quantity).
    Split { pos: usize, neg: usize },
}

/// A standard-form LP built from a [`Model`].
pub(crate) struct StandardForm {
    /// Number of simplex columns.
    pub num_cols: usize,
    /// Dense equality rows.
    pub rows: Matrix,
    /// Right-hand side, one per row. Always `>= 0`.
    pub rhs: Vector,
    /// `+1.0` if a row was left as constructed, `-1.0` if it was negated
    /// during construction to make its `rhs >= 0`.
    pub row_sign: Vec<f64>,
    /// The column index of each row's dedicated artificial variable, in
    /// the same order as `rows`.
    pub artificial_cols: Vec<usize>,
    /// Number of rows built directly from `model.constraints()`, in that
    /// same order (`rows[0..num_primary_rows]` correspond 1:1 to
    /// `model.cons()`). Rows at or beyond this index are synthetic
    /// variable-upper-bound rows with no corresponding [`cnvx_core::Con`].
    pub num_primary_rows: usize,
    /// The real (Phase 2) objective, always in minimize form: negated
    /// already if `model.sense()` is [`Sense::Maximize`].
    pub objective: Vector,
    /// The constant term of the real objective, in the same
    /// always-minimize form as `objective`.
    pub objective_constant: f64,
    /// `+1.0` for [`Sense::Minimize`], `-1.0` for [`Sense::Maximize`]. The
    /// model's true objective value is `sense_sign * internal_min_value`.
    pub sense_sign: f64,
    /// How each of `model`'s variables maps onto `y` columns.
    pub var_columns: HashMap<Var, Column>,
}

/// How a bound pair `[lower, upper]` reduces to nonnegative columns.
enum Case {
    Fixed(f64),
    ShiftLower { shift: f64, extra_upper: Option<f64> },
    ShiftUpper { shift: f64 },
    Free,
}

fn classify(bounds: Bounds) -> Result<Case, CnvxError> {
    let (lower, upper) = (bounds.lower, bounds.upper);
    if lower > upper {
        return Err(CnvxError::InvalidBounds { lower, upper });
    }
    if lower.is_finite() && upper.is_finite() && lower == upper {
        Ok(Case::Fixed(lower))
    } else if lower.is_finite() {
        let extra_upper = if upper.is_finite() { Some(upper - lower) } else { None };
        Ok(Case::ShiftLower { shift: lower, extra_upper })
    } else if upper.is_finite() {
        Ok(Case::ShiftUpper { shift: upper })
    } else {
        Ok(Case::Free)
    }
}

/// Substitutes every variable in `expr` via `var_columns`, coalescing
/// duplicate variables, and returns `(sparse column -> coefficient,
/// constant term)`.
fn substitute(
    expr: &Expression,
    var_columns: &HashMap<Var, Column>,
) -> (HashMap<usize, f64>, f64) {
    let mut coalesced: HashMap<Var, f64> = HashMap::new();
    for (v, c) in expr.terms() {
        *coalesced.entry(v).or_insert(0.0) += c;
    }

    let mut cols: HashMap<usize, f64> = HashMap::new();
    let mut constant = expr.constant_term();
    for (v, coeff) in coalesced {
        match var_columns[&v] {
            Column::Fixed(value) => constant += coeff * value,
            Column::Single { shift, sign, col } => {
                constant += coeff * shift;
                *cols.entry(col).or_insert(0.0) += coeff * sign;
            }
            Column::Split { pos, neg } => {
                *cols.entry(pos).or_insert(0.0) += coeff;
                *cols.entry(neg).or_insert(0.0) -= coeff;
            }
        }
    }
    (cols, constant)
}

pub(crate) fn build(model: &Model) -> Result<StandardForm, CnvxError> {
    let mut num_cols = 0usize;
    let mut pending_bound_rows: Vec<(usize, f64)> = Vec::new();

    let mut var_columns: HashMap<Var, Column> = HashMap::with_capacity(model.num_vars());
    for v in model.vars() {
        let column = match classify(model.bounds(v)?)? {
            Case::Fixed(value) => Column::Fixed(value),
            Case::ShiftLower { shift, extra_upper } => {
                let col = num_cols;
                num_cols += 1;
                if let Some(limit) = extra_upper {
                    pending_bound_rows.push((col, limit));
                }
                Column::Single { shift, sign: 1.0, col }
            }
            Case::ShiftUpper { shift } => {
                let col = num_cols;
                num_cols += 1;
                Column::Single { shift, sign: -1.0, col }
            }
            Case::Free => {
                let pos = num_cols;
                let neg = num_cols + 1;
                num_cols += 2;
                Column::Split { pos, neg }
            }
        };
        var_columns.insert(v, column);
    }

    let (obj_cols, obj_constant) = substitute(model.objective(), &var_columns);
    let sense_sign = match model.sense() {
        Sense::Minimize => 1.0,
        Sense::Maximize => -1.0,
    };

    let mut primary_rows: Vec<(HashMap<usize, f64>, f64)> =
        Vec::with_capacity(model.num_constraints());
    for (_, constraint) in model.constraints() {
        let (mut row_cols, expr_constant) = substitute(constraint.expr(), &var_columns);
        let (aux_bounds, target_rhs) = match constraint.kind() {
            ConstraintKind::Leq(rhs) => (Bounds::new(0.0, f64::INFINITY), rhs),
            ConstraintKind::Geq(rhs) => (Bounds::new(f64::NEG_INFINITY, 0.0), rhs),
            ConstraintKind::Eq(rhs) => (Bounds::new(0.0, 0.0), rhs),
            ConstraintKind::Ranged { lower, upper } => {
                (Bounds::new(0.0, upper - lower), upper)
            }
        };

        let mut row_constant = expr_constant;
        match classify(aux_bounds)? {
            Case::Fixed(value) => row_constant += value,
            Case::ShiftLower { shift, extra_upper } => {
                let col = num_cols;
                num_cols += 1;
                if let Some(limit) = extra_upper {
                    pending_bound_rows.push((col, limit));
                }
                row_constant += shift;
                *row_cols.entry(col).or_insert(0.0) += 1.0;
            }
            Case::ShiftUpper { shift } => {
                let col = num_cols;
                num_cols += 1;
                row_constant += shift;
                *row_cols.entry(col).or_insert(0.0) += -1.0;
            }
            Case::Free => unreachable!(
                "a constraint's comparison slack is never free on both sides"
            ),
        }
        primary_rows.push((row_cols, target_rhs - row_constant));
    }
    let num_primary_rows = primary_rows.len();

    let mut all_rows = primary_rows;
    for (col, limit) in pending_bound_rows {
        let slack_col = num_cols;
        num_cols += 1;
        let mut row_cols = HashMap::with_capacity(2);
        row_cols.insert(col, 1.0);
        row_cols.insert(slack_col, 1.0);
        all_rows.push((row_cols, limit));
    }

    let num_rows = all_rows.len();
    let mut artificial_cols = Vec::with_capacity(num_rows);
    for _ in 0..num_rows {
        artificial_cols.push(num_cols);
        num_cols += 1;
    }

    let mut rows = Matrix::zeros(num_rows, num_cols);
    let mut rhs = vec![0.0; num_rows];
    let mut row_sign = vec![0.0; num_rows];
    for (i, (row_cols, row_rhs)) in all_rows.into_iter().enumerate() {
        let sign = if row_rhs < 0.0 { -1.0 } else { 1.0 };
        for (col, coeff) in row_cols {
            rows.set(i, col, sign * coeff);
        }
        rows.set(i, artificial_cols[i], 1.0);
        rhs[i] = sign * row_rhs;
        row_sign[i] = sign;
    }

    let mut objective = vec![0.0; num_cols];
    for (col, coeff) in obj_cols {
        objective[col] = sense_sign * coeff;
    }
    let objective_constant = sense_sign * obj_constant;

    Ok(StandardForm {
        num_cols,
        rows,
        rhs: Vector::from(rhs),
        row_sign,
        artificial_cols,
        num_primary_rows,
        objective: Vector::from(objective),
        objective_constant,
        sense_sign,
        var_columns,
    })
}
