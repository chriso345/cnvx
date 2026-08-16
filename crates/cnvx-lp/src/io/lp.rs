use std::collections::HashMap;
use std::fmt::Write as _;

use cnvx_core::{
    Bounds, CnvxError, Constraint, ConstraintKind, Expression, Model, Sense, Var, VarKind,
};

use super::var_names;

pub(super) fn render(model: &Model) -> String {
    let names = var_names(model);
    let mut out = String::new();
    writeln!(out, "\\ {}", model.name()).ok();
    writeln!(
        out,
        "{}",
        match model.sense() {
            Sense::Minimize => "Minimize",
            Sense::Maximize => "Maximize",
        }
    )
    .ok();
    writeln!(out, " obj: {}", format_expr_terms(model.objective(), &names)).ok();

    writeln!(out, "Subject To").ok();
    for (i, (_, constraint)) in model.constraints().enumerate() {
        let label = constraint
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("c{i}"));
        writeln!(out, " {label}: {}", format_constraint(constraint, &names)).ok();
    }

    writeln!(out, "Bounds").ok();
    for v in model.vars() {
        let bounds = model.bounds(v).expect("handle from this model");
        let name = &names[&v];
        if bounds.lower == 0.0 && bounds.upper.is_infinite() {
            continue; // the LP format default; no line needed
        }
        if bounds.lower.is_infinite() && bounds.upper.is_infinite() {
            writeln!(out, " free {name}").ok();
        } else if bounds.lower == bounds.upper {
            writeln!(out, " {name} = {}", bounds.lower).ok();
        } else {
            let lower = if bounds.lower.is_infinite() {
                "-1e30".to_string()
            } else {
                bounds.lower.to_string()
            };
            let upper = if bounds.upper.is_infinite() {
                "1e30".to_string()
            } else {
                bounds.upper.to_string()
            };
            writeln!(out, " {lower} <= {name} <= {upper}").ok();
        }
    }

    let integers: Vec<&str> = model
        .vars()
        .filter(|&v| matches!(model.kind(v), Ok(VarKind::Integer)))
        .map(|v| names[&v].as_str())
        .collect();
    if !integers.is_empty() {
        writeln!(out, "General").ok();
        writeln!(out, " {}", integers.join(" ")).ok();
    }
    let binaries: Vec<&str> = model
        .vars()
        .filter(|&v| matches!(model.kind(v), Ok(VarKind::Binary)))
        .map(|v| names[&v].as_str())
        .collect();
    if !binaries.is_empty() {
        writeln!(out, "Binary").ok();
        writeln!(out, " {}", binaries.join(" ")).ok();
    }

    writeln!(out, "End").ok();
    out
}

/// Formats a linear expression as `c1 name1 + c2 name2 ... [+ constant]`,
/// coalescing duplicate variables and skipping zero coefficients.
fn format_expr_terms(expr: &Expression, names: &HashMap<Var, String>) -> String {
    let mut coalesced: HashMap<Var, f64> = HashMap::new();
    for (v, c) in expr.terms() {
        *coalesced.entry(v).or_insert(0.0) += c;
    }
    let mut terms: Vec<(&str, f64)> = coalesced
        .iter()
        .filter(|&(_, &c)| c != 0.0)
        .map(|(v, &c)| (names[v].as_str(), c))
        .collect();
    terms.sort_by(|a, b| a.0.cmp(b.0));

    let mut out = String::new();
    for (i, (name, coeff)) in terms.iter().enumerate() {
        if i > 0 {
            out.push_str(if *coeff < 0.0 { " - " } else { " + " });
        } else if *coeff < 0.0 {
            out.push('-');
        }
        let magnitude = coeff.abs();
        if magnitude != 1.0 {
            write!(out, "{magnitude} ").ok();
        }
        out.push_str(name);
    }
    let constant = expr.constant_term();
    if constant != 0.0 || out.is_empty() {
        if !out.is_empty() {
            out.push_str(if constant < 0.0 { " - " } else { " + " });
            write!(out, "{}", constant.abs()).ok();
        } else {
            write!(out, "{constant}").ok();
        }
    }
    out
}

fn format_constraint(constraint: &Constraint, names: &HashMap<Var, String>) -> String {
    let body = format_expr_terms(constraint.expr(), names);
    match constraint.kind() {
        ConstraintKind::Leq(rhs) => format!("{body} <= {rhs}"),
        ConstraintKind::Geq(rhs) => format!("{body} >= {rhs}"),
        ConstraintKind::Eq(rhs) => format!("{body} = {rhs}"),
        ConstraintKind::Ranged { lower, upper } => {
            format!("{lower} <= {body} <= {upper}")
        }
    }
}

pub(super) fn parse(contents: &str) -> Result<Model, CnvxError> {
    let model_name = contents
        .lines()
        .next()
        .and_then(|l| l.trim().strip_prefix('\\'))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "model".to_string());

    let mut lines = contents
        .lines()
        .map(|line| line.split('\\').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty());

    let sense = match lines.next() {
        Some(s)
            if s.eq_ignore_ascii_case("minimize") || s.eq_ignore_ascii_case("min") =>
        {
            Sense::Minimize
        }
        Some(s)
            if s.eq_ignore_ascii_case("maximize") || s.eq_ignore_ascii_case("max") =>
        {
            Sense::Maximize
        }
        other => {
            return Err(CnvxError::Numerical(format!(
                "expected \"Minimize\" or \"Maximize\" as the first line, got {other:?}"
            )));
        }
    };

    let objective_line = lines.next().ok_or_else(|| {
        CnvxError::Numerical(
            "expected an objective line after Minimize/Maximize".to_string(),
        )
    })?;
    let objective_body = objective_line
        .split_once(':')
        .map(|(_, body)| body)
        .unwrap_or(objective_line);

    let mut order: Vec<String> = Vec::new();
    let mut var_bounds: HashMap<String, Bounds> = HashMap::new();
    let declare = |name: &str,
                   order: &mut Vec<String>,
                   var_bounds: &mut HashMap<String, Bounds>| {
        if !var_bounds.contains_key(name) {
            order.push(name.to_string());
            var_bounds.insert(name.to_string(), Bounds::new(0.0, f64::INFINITY));
        }
    };

    let (objective_terms, objective_constant) = parse_expr(objective_body)?;
    for (name, _) in &objective_terms {
        declare(name, &mut order, &mut var_bounds);
    }

    let mut section = "subject to".to_string();
    let mut constraint_lines: Vec<String> = Vec::new();
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if [
            "subject to",
            "st",
            "s.t.",
            "bounds",
            "general",
            "integer",
            "binary",
            "binaries",
            "end",
        ]
        .contains(&lower.as_str())
        {
            section = lower;
            continue;
        }
        match section.as_str() {
            "subject to" | "st" | "s.t." => constraint_lines.push(line.to_string()),
            "bounds" => parse_bound_line(line, &mut order, &mut var_bounds)?,
            "end" => {}
            _ => {} // General/Binary: parsed for variable names only, kind not restored
        }
    }

    for line in &constraint_lines {
        let body = line.split_once(':').map(|(_, b)| b).unwrap_or(line.as_str());
        let (terms, _) = parse_expr(constraint_expr_text(body))?;
        for (name, _) in &terms {
            declare(name, &mut order, &mut var_bounds);
        }
    }

    let mut model = Model::new(model_name);
    let mut vars: HashMap<String, Var> = HashMap::with_capacity(order.len());
    for name in &order {
        let bounds = var_bounds[name];
        vars.insert(name.clone(), model.add_named_var(bounds, name));
    }

    let objective_expr = build_expr(&objective_terms, objective_constant, &vars)?;
    model.set_objective(sense, objective_expr)?;

    for (i, line) in constraint_lines.iter().enumerate() {
        let (label, body) = match line.split_once(':') {
            Some((label, body)) => (label.trim().to_string(), body),
            None => (format!("c{i}"), line.as_str()),
        };
        let constraint = parse_constraint(body, &vars)?;
        model.add_named_constraint(constraint, &label)?;
    }

    Ok(model)
}

fn parse_expr(input: &str) -> Result<(Vec<(String, f64)>, f64), CnvxError> {
    let spaced = input.replace('-', " - ").replace('+', " + ");
    let tokens: Vec<&str> = spaced.split_whitespace().collect();

    let mut terms = Vec::new();
    let mut constant = 0.0;
    let mut sign = 1.0;
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            token => {
                if let Ok(value) = token.parse::<f64>() {
                    // Either a bare constant, or a coefficient followed by a name.
                    if i + 1 < tokens.len()
                        && tokens[i + 1] != "+"
                        && tokens[i + 1] != "-"
                    {
                        terms.push((tokens[i + 1].to_string(), sign * value));
                        i += 1;
                    } else {
                        constant += sign * value;
                    }
                } else {
                    terms.push((token.to_string(), sign));
                }
                sign = 1.0;
            }
        }
        i += 1;
    }
    Ok((terms, constant))
}

fn build_expr(
    terms: &[(String, f64)],
    constant: f64,
    vars: &HashMap<String, Var>,
) -> Result<Expression, CnvxError> {
    let mut expr = Expression::zero() + constant;
    for (name, coeff) in terms {
        let v = *vars.get(name).ok_or_else(|| {
            CnvxError::Numerical(format!("undeclared variable {name:?} in expression"))
        })?;
        expr = expr + *coeff * v;
    }
    Ok(expr)
}

fn constraint_expr_text(body: &str) -> &str {
    if let Some((lo, rest)) = body.split_once("<=")
        && let Some((mid, hi)) = rest.split_once("<=")
        && lo.trim().parse::<f64>().is_ok()
        && hi.trim().parse::<f64>().is_ok()
    {
        return mid;
    }
    for op in ["<=", ">=", "="] {
        if let Some((lhs, _)) = body.split_once(op) {
            return lhs;
        }
    }
    body
}

fn parse_constraint(
    body: &str,
    vars: &HashMap<String, Var>,
) -> Result<Constraint, CnvxError> {
    // Ranged: `lo <= expr <= hi`.
    if let Some((lo, rest)) = body.split_once("<=")
        && let Some((mid, hi)) = rest.split_once("<=")
        && let (Ok(lo), Ok(hi)) = (lo.trim().parse::<f64>(), hi.trim().parse::<f64>())
    {
        let (terms, constant) = parse_expr(mid)?;
        let expr = build_expr(&terms, constant, vars)?;
        return Ok(expr.between(lo, hi));
    }
    for (op, build) in [
        ("<=", Expression::leq as fn(Expression, f64) -> Constraint),
        (">=", Expression::geq),
        ("=", Expression::eq),
    ] {
        if let Some((lhs, rhs)) = body.split_once(op) {
            let rhs = rhs.trim().parse::<f64>().map_err(|_| {
                CnvxError::Numerical(format!(
                    "expected a numeric right-hand side, got {:?}",
                    rhs.trim()
                ))
            })?;
            let (terms, constant) = parse_expr(lhs)?;
            let expr = build_expr(&terms, constant, vars)?;
            return Ok(build(expr, rhs));
        }
    }
    Err(CnvxError::Numerical(format!("could not parse constraint: {body:?}")))
}

fn parse_bound_line(
    line: &str,
    order: &mut Vec<String>,
    var_bounds: &mut HashMap<String, Bounds>,
) -> Result<(), CnvxError> {
    let declare = |name: &str,
                   order: &mut Vec<String>,
                   var_bounds: &mut HashMap<String, Bounds>| {
        if !var_bounds.contains_key(name) {
            order.push(name.to_string());
            var_bounds.insert(name.to_string(), Bounds::new(0.0, f64::INFINITY));
        }
    };

    let lower_line = line.to_ascii_lowercase();
    if lower_line.starts_with("free ") {
        let name = line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        declare(name, order, var_bounds);
        var_bounds.insert(name.to_string(), Bounds::from(..));
        return Ok(());
    }

    if line.matches("<=").count() == 2 {
        let parts: Vec<&str> = line.splitn(3, "<=").collect();
        let lo: f64 = parts[0]
            .trim()
            .parse()
            .map_err(|_| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        let name = parts[1].trim();
        let hi: f64 = parts[2]
            .trim()
            .parse()
            .map_err(|_| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        declare(name, order, var_bounds);
        var_bounds.insert(name.to_string(), Bounds::new(lo, hi));
        return Ok(());
    }
    if let Some((name, rhs)) = line.split_once(">=") {
        let name = name.trim();
        let lo: f64 = rhs
            .trim()
            .parse()
            .map_err(|_| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        declare(name, order, var_bounds);
        let upper = var_bounds[name].upper;
        var_bounds.insert(name.to_string(), Bounds::new(lo, upper));
        return Ok(());
    }
    if let Some((name, rhs)) = line.split_once("<=") {
        let name = name.trim();
        let hi: f64 = rhs
            .trim()
            .parse()
            .map_err(|_| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        declare(name, order, var_bounds);
        let lower = var_bounds[name].lower;
        var_bounds.insert(name.to_string(), Bounds::new(lower, hi));
        return Ok(());
    }
    if let Some((name, rhs)) = line.split_once('=') {
        let name = name.trim();
        let value: f64 = rhs
            .trim()
            .parse()
            .map_err(|_| CnvxError::Numerical(format!("bad bound line: {line:?}")))?;
        declare(name, order, var_bounds);
        var_bounds.insert(name.to_string(), Bounds::fixed(value));
        return Ok(());
    }
    Err(CnvxError::Numerical(format!("could not parse bound line: {line:?}")))
}
