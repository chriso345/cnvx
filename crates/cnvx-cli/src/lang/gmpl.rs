use super::LanguageParser;
use cnvx_core::{LinExpr, Model, Objective, VarId};

#[derive(Default)]
pub struct GMPLLanguage;

impl GMPLLanguage {
    pub fn new() -> Self {
        Self {}
    }
}

impl LanguageParser for GMPLLanguage {
    fn parse(&self, src: &str) -> Result<Model, String> {
        let mut model = Model::new();
        let mut vars: Vec<VarId> = vec![];

        for line in src.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("var ") {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let v = model.add_var().finish();
                    vars.push(v);
                }
            } else if line.to_lowercase().starts_with("maximize") {
                let after_colon = line.split(':').nth(1).unwrap_or("").trim();
                let expr = parse_expression(after_colon, &vars)?;
                model.add_objective(Objective::maximize(expr).name("Z"));
            } else if line.to_lowercase().starts_with("subject to") {
                let rest = line.split(':').nth(1).unwrap_or("").trim();
                let (lhs, rhs, cmp) = parse_constraint(rest, &vars)?;
                match cmp {
                    "<=" => model += lhs.leq(rhs),
                    ">=" => model += lhs.geq(rhs),
                    "=" => model += lhs.eq(rhs),
                    _ => return Err(format!("unknown constraint type '{}'", cmp)),
                }
            }
        }

        Ok(model)
    }
}

fn parse_expression(expr: &str, vars: &[VarId]) -> Result<LinExpr, String> {
    let mut le = LinExpr::constant(0.0);

    for tok in expr.split('+') {
        let tok = tok.trim();
        if tok.is_empty() {
            continue;
        }

        let (coef, varname) = if tok.contains('*') {
            let parts: Vec<_> = tok.split('*').collect();
            let coef = parts[0]
                .parse::<f64>()
                .map_err(|_| format!("invalid coefficient '{}'", parts[0]))?;
            (coef, parts[1])
        } else if tok.starts_with('-') {
            (-1.0, &tok[1..])
        } else {
            (1.0, tok)
        };

        let varname = varname.trim().trim_end_matches(';');

        let idx = varname[1..]
            .parse::<usize>()
            .map_err(|_| format!("invalid variable '{}'", varname))?;
        if idx == 0 || idx > vars.len() {
            return Err(format!("unknown variable '{}'", varname));
        }
        le += coef * vars[idx - 1];
    }

    Ok(le)
}

fn parse_constraint<'a>(
    line: &'a str,
    vars: &[VarId],
) -> Result<(LinExpr, f64, &'a str), String> {
    let cmp: &'a str = if line.contains("<=") {
        "<="
    } else if line.contains(">=") {
        ">="
    } else if line.contains('=') {
        "="
    } else {
        return Err("invalid constraint".into());
    };
    let parts: Vec<&str> = line.split(cmp).collect();
    if parts.len() != 2 {
        return Err("invalid constraint format".into());
    }

    let lhs = parse_expression(parts[0].trim(), vars)?;
    let rhs = parts[1]
        .trim()
        .trim_end_matches(';') // <-- remove trailing semicolon
        .parse::<f64>()
        .map_err(|_| "invalid RHS".to_string())?;

    Ok((lhs, rhs, cmp))
}
