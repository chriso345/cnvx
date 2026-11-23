use std::collections::HashMap;
use std::fmt::Write as _;

use cnvx_core::{Bounds, CnvxError, ConstraintKind, Expression, Model, Sense, Var};

use super::var_names;

/// Renders `model` as free-format MPS text.
pub(super) fn render(model: &Model) -> String {
    let names = var_names(model);
    let mut out = String::new();
    writeln!(out, "NAME          {}", model.name()).ok();

    writeln!(out, "ROWS").ok();
    writeln!(out, " N  obj").ok();
    for (i, (_, constraint)) in model.constraints().enumerate() {
        let label = constraint
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("c{i}"));
        let row_type = match constraint.kind() {
            ConstraintKind::Leq(_) => "L",
            ConstraintKind::Geq(_) => "G",
            ConstraintKind::Eq(_) => "E",
            ConstraintKind::Ranged { .. } => "L", /* upper bound; the range comes via
                                                   * RANGES below */
        };
        writeln!(out, " {row_type}  {label}").ok();
    }

    writeln!(out, "COLUMNS").ok();
    let sense_sign = match model.sense() {
        Sense::Minimize => 1.0,
        Sense::Maximize => -1.0,
    };
    let mut objective_coeffs: HashMap<Var, f64> = HashMap::new();
    for (v, c) in model.objective().terms() {
        *objective_coeffs.entry(v).or_insert(0.0) += c;
    }
    for v in model.vars() {
        let name = &names[&v];
        if let Some(&coeff) = objective_coeffs.get(&v)
            && coeff != 0.0
        {
            writeln!(out, "    {name}  obj  {}", sense_sign * coeff).ok();
        }
        for (i, (_, constraint)) in model.constraints().enumerate() {
            let label = constraint
                .name()
                .map(str::to_string)
                .unwrap_or_else(|| format!("c{i}"));
            let coeff: f64 = constraint
                .expr()
                .terms()
                .filter(|(cv, _)| *cv == v)
                .map(|(_, c)| c)
                .sum();
            if coeff != 0.0 {
                writeln!(out, "    {name}  {label}  {coeff}").ok();
            }
        }
    }

    writeln!(out, "RHS").ok();
    for (i, (_, constraint)) in model.constraints().enumerate() {
        let label = constraint
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("c{i}"));
        let rhs = match constraint.kind() {
            ConstraintKind::Leq(r) | ConstraintKind::Geq(r) | ConstraintKind::Eq(r) => r,
            ConstraintKind::Ranged { upper, .. } => upper,
        };
        writeln!(out, "    RHS  {label}  {rhs}").ok();
    }

    let ranged: Vec<(String, f64)> = model
        .constraints()
        .enumerate()
        .filter_map(|(i, (_, c))| match c.kind() {
            ConstraintKind::Ranged { lower, upper } => {
                let label =
                    c.name().map(str::to_string).unwrap_or_else(|| format!("c{i}"));
                Some((label, upper - lower))
            }
            _ => None,
        })
        .collect();
    if !ranged.is_empty() {
        writeln!(out, "RANGES").ok();
        for (label, range) in ranged {
            writeln!(out, "    RNG  {label}  {range}").ok();
        }
    }

    writeln!(out, "BOUNDS").ok();
    for v in model.vars() {
        let bounds = model.bounds(v).expect("handle from this model");
        let name = &names[&v];
        if bounds.lower == 0.0 && bounds.upper.is_infinite() {
            continue;
        }
        if bounds.lower.is_infinite() && bounds.upper.is_infinite() {
            writeln!(out, " FR BND  {name}").ok();
            continue;
        }
        if bounds.lower == bounds.upper {
            writeln!(out, " FX BND  {name}  {}", bounds.lower).ok();
            continue;
        }
        if bounds.lower != 0.0 && bounds.lower.is_finite() {
            writeln!(out, " LO BND  {name}  {}", bounds.lower).ok();
        }
        if bounds.lower.is_infinite() {
            writeln!(out, " MI BND  {name}").ok();
        }
        if bounds.upper.is_finite() {
            writeln!(out, " UP BND  {name}  {}", bounds.upper).ok();
        }
    }

    writeln!(out, "ENDATA").ok();
    out
}

#[derive(Clone, Copy)]
enum Section {
    None,
    ObjSense,
    Rows,
    Columns,
    Rhs,
    Ranges,
    Bounds,
    Skip,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowKind {
    Leq,
    Geq,
    Eq,
}

#[derive(Default)]
struct ColumnData {
    integer: bool,
    obj_coeff: f64,
    row_coeffs: HashMap<String, f64>,
}

struct BoundState {
    lower: f64,
    upper: f64,
    lower_explicit: bool,
    integer: bool,
}

impl Default for BoundState {
    fn default() -> Self {
        // MPS's own defaults: nonnegative, no explicit upper bound.
        BoundState {
            lower: 0.0,
            upper: f64::INFINITY,
            lower_explicit: false,
            integer: false,
        }
    }
}

pub(super) fn parse(contents: &str) -> Result<Model, CnvxError> {
    let mut sense = Sense::Minimize;
    let mut model_name = "model".to_string();

    let mut objective_row: Option<String> = None;
    let mut free_rows: Vec<String> = Vec::new(); // extra `N` rows: recognized, otherwise ignored
    let mut row_kinds: HashMap<String, RowKind> = HashMap::new();
    let mut row_order: Vec<String> = Vec::new(); // non-objective, non-free rows, in ROWS order

    let mut column_order: Vec<String> = Vec::new();
    let mut columns: HashMap<String, ColumnData> = HashMap::new();
    let mut in_integer_marker = false;

    let mut rhs: HashMap<String, f64> = HashMap::new();
    let mut objective_constant = 0.0_f64;
    let mut ranges: HashMap<String, f64> = HashMap::new();
    let mut bounds: HashMap<String, BoundState> = HashMap::new();

    let mut section = Section::None;

    for raw_line in contents.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }
        if raw_line.trim_start().starts_with('*') {
            continue; // comment line
        }

        let is_header = !raw_line.starts_with(' ') && !raw_line.starts_with('\t');
        if is_header {
            let mut tokens = raw_line.split_whitespace();
            let keyword = tokens.next().unwrap_or("");
            section = match keyword {
                "NAME" => {
                    if let Some(name) = tokens.next() {
                        model_name = name.to_string();
                    }
                    Section::None
                }
                "OBJSENSE" => Section::ObjSense,
                "ROWS" => Section::Rows,
                "COLUMNS" => Section::Columns,
                "RHS" => Section::Rhs,
                "RANGES" => Section::Ranges,
                "BOUNDS" => Section::Bounds,
                "ENDATA" => break,
                _ => Section::Skip,
            };
            continue;
        }

        let tokens: Vec<&str> = raw_line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match section {
            Section::None | Section::Skip => {}

            Section::ObjSense => {
                if tokens[0].eq_ignore_ascii_case("max")
                    || tokens[0].eq_ignore_ascii_case("maximize")
                {
                    sense = Sense::Maximize;
                } else if tokens[0].eq_ignore_ascii_case("min")
                    || tokens[0].eq_ignore_ascii_case("minimize")
                {
                    sense = Sense::Minimize;
                }
            }

            Section::Rows => {
                if tokens.len() < 2 {
                    return Err(CnvxError::Numerical(format!(
                        "malformed ROWS line: {raw_line:?}"
                    )));
                }
                let name = tokens[1].to_string();
                match tokens[0].to_ascii_uppercase().as_str() {
                    "N" => {
                        if objective_row.is_none() {
                            objective_row = Some(name);
                        } else {
                            free_rows.push(name);
                        }
                    }
                    "L" => {
                        row_kinds.insert(name.clone(), RowKind::Leq);
                        row_order.push(name);
                    }
                    "G" => {
                        row_kinds.insert(name.clone(), RowKind::Geq);
                        row_order.push(name);
                    }
                    "E" => {
                        row_kinds.insert(name.clone(), RowKind::Eq);
                        row_order.push(name);
                    }
                    other => {
                        return Err(CnvxError::Numerical(format!(
                            "unknown row type {other:?} in ROWS section: {raw_line:?}"
                        )));
                    }
                }
            }

            Section::Columns => {
                // `<name> 'MARKER' 'INTORG'|'INTEND'`: toggles whether
                // subsequently-declared columns are integer, until the
                // matching end marker.
                if tokens.get(1) == Some(&"'MARKER'") {
                    if tokens.contains(&"'INTORG'") {
                        in_integer_marker = true;
                    } else if tokens.contains(&"'INTEND'") {
                        in_integer_marker = false;
                    }
                    continue;
                }
                if tokens.len() < 3 || tokens.len().is_multiple_of(2) {
                    return Err(CnvxError::Numerical(format!(
                        "malformed COLUMNS line: {raw_line:?}"
                    )));
                }
                let column_name = tokens[0].to_string();
                if !columns.contains_key(&column_name) {
                    column_order.push(column_name.clone());
                }
                let data = columns.entry(column_name).or_default();
                data.integer = data.integer || in_integer_marker;

                for pair in tokens[1..].as_chunks::<2>().0 {
                    let row_name = pair[0];
                    let value: f64 = pair[1].parse().map_err(|_| {
                        CnvxError::Numerical(format!(
                            "bad coefficient {:?}: {raw_line:?}",
                            pair[1]
                        ))
                    })?;
                    if Some(row_name) == objective_row.as_deref() {
                        data.obj_coeff += value;
                    } else if free_rows.iter().any(|r| r == row_name) {
                        // A non-objective free (`N`) row: deliberately dropped.
                    } else if row_kinds.contains_key(row_name) {
                        *data.row_coeffs.entry(row_name.to_string()).or_insert(0.0) +=
                            value;
                    } else {
                        return Err(CnvxError::Numerical(format!(
                            "COLUMNS line references undeclared row {row_name:?}: {raw_line:?}"
                        )));
                    }
                }
            }

            Section::Rhs => {
                if tokens.len() < 3 || tokens.len().is_multiple_of(2) {
                    return Err(CnvxError::Numerical(format!(
                        "malformed RHS line: {raw_line:?}"
                    )));
                }
                for pair in tokens[1..].as_chunks::<2>().0 {
                    let row_name = pair[0];
                    let value: f64 = pair[1].parse().map_err(|_| {
                        CnvxError::Numerical(format!(
                            "bad RHS value {:?}: {raw_line:?}",
                            pair[1]
                        ))
                    })?;
                    if Some(row_name) == objective_row.as_deref() {
                        objective_constant = -value;
                    } else {
                        rhs.insert(row_name.to_string(), value);
                    }
                }
            }

            Section::Ranges => {
                if tokens.len() < 3 || tokens.len().is_multiple_of(2) {
                    return Err(CnvxError::Numerical(format!(
                        "malformed RANGES line: {raw_line:?}"
                    )));
                }
                for pair in tokens[1..].as_chunks::<2>().0 {
                    let value: f64 = pair[1].parse().map_err(|_| {
                        CnvxError::Numerical(format!(
                            "bad RANGES value {:?}: {raw_line:?}",
                            pair[1]
                        ))
                    })?;
                    ranges.insert(pair[0].to_string(), value);
                }
            }

            Section::Bounds => {
                if tokens.len() < 3 {
                    return Err(CnvxError::Numerical(format!(
                        "malformed BOUNDS line: {raw_line:?}"
                    )));
                }
                let bound_type = tokens[0].to_ascii_uppercase();
                let column_name = tokens[2];
                let value: Option<f64> = tokens
                    .get(3)
                    .copied()
                    .map(|v| {
                        v.parse::<f64>().map_err(|_| {
                            CnvxError::Numerical(format!(
                                "bad bound value {v:?}: {raw_line:?}"
                            ))
                        })
                    })
                    .transpose()?;
                let state = bounds.entry(column_name.to_string()).or_default();
                apply_bound(state, &bound_type, value, raw_line)?;
            }
        }
    }

    if objective_row.is_none() {
        return Err(CnvxError::Numerical(
            "MPS file has no objective (\"N\") row in its ROWS section".to_string(),
        ));
    }

    let mut model = Model::new(model_name);
    let mut vars: HashMap<String, Var> = HashMap::with_capacity(column_order.len());
    for name in &column_order {
        let state = bounds.remove(name).unwrap_or_default();
        let is_integer = columns[name].integer || state.integer;
        let column_bounds = Bounds::new(state.lower, state.upper);
        let var = if is_integer {
            model.add_integer(column_bounds, name)
        } else {
            model.add_named_var(column_bounds, name)
        };
        vars.insert(name.clone(), var);
    }

    let objective_expr = column_order.iter().fold(Expression::zero(), |acc, name| {
        let coeff = columns[name].obj_coeff;
        if coeff == 0.0 { acc } else { acc + coeff * vars[name] }
    }) + objective_constant;
    model.set_objective(sense, objective_expr)?;

    for row_name in &row_order {
        let kind = row_kinds[row_name];
        let rhs_value = *rhs.get(row_name).unwrap_or(&0.0); // MPS default: 0 if unspecified
        let expr =
            column_order.iter().fold(Expression::zero(), |acc, name| {
                match columns[name].row_coeffs.get(row_name) {
                    Some(&c) if c != 0.0 => acc + c * vars[name],
                    _ => acc,
                }
            });
        let constraint = match (kind, ranges.get(row_name)) {
            (RowKind::Leq, Some(&range)) => {
                expr.between(rhs_value - range.abs(), rhs_value)
            }
            (RowKind::Geq, Some(&range)) => {
                expr.between(rhs_value, rhs_value + range.abs())
            }
            (RowKind::Eq, Some(&range)) if range >= 0.0 => {
                expr.between(rhs_value, rhs_value + range)
            }
            (RowKind::Eq, Some(&range)) => expr.between(rhs_value + range, rhs_value),
            (RowKind::Leq, None) => expr.leq(rhs_value),
            (RowKind::Geq, None) => expr.geq(rhs_value),
            (RowKind::Eq, None) => expr.eq(rhs_value),
        };
        model.add_named_constraint(constraint, row_name)?;
    }

    Ok(model)
}

/// Applies one `BOUNDS` line to `state`.
fn apply_bound(
    state: &mut BoundState,
    bound_type: &str,
    value: Option<f64>,
    line: &str,
) -> Result<(), CnvxError> {
    match bound_type {
        "UP" => {
            let v = require_value(value, line)?;
            state.upper = v;
            if v < 0.0 && !state.lower_explicit {
                state.lower = f64::NEG_INFINITY;
            }
        }
        "LO" => {
            state.lower = require_value(value, line)?;
            state.lower_explicit = true;
        }
        "FX" => {
            let v = require_value(value, line)?;
            state.lower = v;
            state.upper = v;
            state.lower_explicit = true;
        }
        "FR" => {
            state.lower = f64::NEG_INFINITY;
            state.upper = f64::INFINITY;
            state.lower_explicit = true;
        }
        "MI" => {
            state.lower = f64::NEG_INFINITY;
            state.lower_explicit = true;
        }
        "PL" => {
            state.upper = f64::INFINITY;
        }
        "BV" => {
            state.lower = 0.0;
            state.upper = 1.0;
            state.lower_explicit = true;
            state.integer = true;
        }
        "UI" => {
            state.upper = require_value(value, line)?;
            state.integer = true;
        }
        "LI" => {
            state.lower = require_value(value, line)?;
            state.lower_explicit = true;
            state.integer = true;
        }
        other => {
            return Err(CnvxError::Numerical(format!(
                "unsupported bound type {other:?}: {line:?}"
            )));
        }
    }
    Ok(())
}

fn require_value(value: Option<f64>, line: &str) -> Result<f64, CnvxError> {
    value.ok_or_else(|| {
        CnvxError::Numerical(format!("bound line is missing its value: {line:?}"))
    })
}
