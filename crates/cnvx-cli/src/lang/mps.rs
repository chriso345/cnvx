use super::LanguageParser;
use cnvx_core::{LinExpr, Model, Objective, VarId};
use std::collections::HashMap;

#[derive(Default)]
pub struct MPSLanguage;

impl MPSLanguage {
    pub fn new() -> Self {
        Self {}
    }
}

impl LanguageParser for MPSLanguage {
    fn parse(&self, src: &str) -> Result<Model, String> {
        let mut model = Model::new();
        let mut section = "";

        let mut rows: HashMap<String, char> = HashMap::new();
        let mut col_exprs: HashMap<String, LinExpr> = HashMap::new();
        let mut rhs_map: HashMap<String, f64> = HashMap::new();
        let mut var_map: HashMap<String, VarId> = HashMap::new();

        for raw in src.lines() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if line.eq_ignore_ascii_case("ROWS") {
                section = "ROWS";
                continue;
            } else if line.eq_ignore_ascii_case("COLUMNS") {
                section = "COLUMNS";
                continue;
            } else if line.eq_ignore_ascii_case("RHS") {
                section = "RHS";
                continue;
            } else if line.eq_ignore_ascii_case("BOUNDS") {
                section = "BOUNDS";
                continue;
            } else if line.eq_ignore_ascii_case("ENDATA") {
                break;
            }

            match section {
                "ROWS" => {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if parts[0].ends_with('.') && parts.len() >= 3 {
                            let rtype = parts[1].chars().next().unwrap_or(' ');
                            let name = parts[2].to_string();
                            rows.insert(name, rtype);
                        } else {
                            let rtype = parts[0].chars().next().unwrap_or(' ');
                            let name = parts[1].to_string();
                            rows.insert(name, rtype);
                        }
                    }
                }
                "COLUMNS" => {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    let mut idx = 0;
                    if parts[0].ends_with('.') && parts.len() >= 2 {
                        idx = 1;
                    }
                    if parts.len() <= idx {
                        continue;
                    }
                    let col = parts[idx].to_string();
                    let varid = *var_map
                        .entry(col.clone())
                        .or_insert_with(|| model.add_var().finish());
                    let mut i = idx + 1;
                    while i + 1 < parts.len() {
                        let row = parts[i].to_string();
                        let val = parts[i + 1].parse::<f64>().map_err(|_| {
                            format!("invalid number in COLUMNS: {}", parts[i + 1])
                        })?;
                        let entry = col_exprs
                            .entry(row.clone())
                            .or_insert(LinExpr::constant(0.0));
                        *entry += LinExpr::new(varid, val);
                        i += 2;
                    }
                }
                "RHS" => {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() < 3 {
                        continue;
                    }
                    let mut idx = 0;
                    if parts[0].ends_with('.') && parts.len() >= 2 {
                        idx = 1;
                    }
                    let mut i = idx + 1;
                    while i + 1 < parts.len() {
                        let row = parts[i].to_string();
                        let val = parts[i + 1].parse::<f64>().map_err(|_| {
                            format!("invalid number in RHS: {}", parts[i + 1])
                        })?;
                        rhs_map.insert(row, val);
                        i += 2;
                    }
                }
                "BOUNDS" => {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() < 3 {
                        continue;
                    }
                    let mut idx = 0;
                    if parts[0].ends_with('.') && parts.len() >= 2 {
                        idx = 1;
                    }
                    let btype = parts[idx];
                    if parts.len() <= idx + 2 {
                        continue;
                    }
                    let varname = parts[idx + 2].to_string();
                    let varid = *var_map
                        .entry(varname.clone())
                        .or_insert_with(|| model.add_var().finish());
                    match btype {
                        "UP" => {
                            if parts.len() >= idx + 4 {
                                if let Ok(v) = parts[idx + 3].parse::<f64>() {
                                    model.vars[varid.0].ub = Some(v);
                                }
                            }
                        }
                        "LO" => {
                            if parts.len() >= idx + 4 {
                                if let Ok(v) = parts[idx + 3].parse::<f64>() {
                                    model.vars[varid.0].lb = Some(v);
                                }
                            }
                        }
                        "FR" => {
                            model.vars[varid.0].lb = None;
                            model.vars[varid.0].ub = None;
                        }
                        "MI" => {
                            model.vars[varid.0].lb = None;
                        }
                        "BV" => {
                            model.vars[varid.0].is_integer = true;
                            model.vars[varid.0].lb = Some(0.0);
                            model.vars[varid.0].ub = Some(1.0);
                        }
                        "FX" => {
                            if parts.len() >= idx + 4 {
                                if let Ok(v) = parts[idx + 3].parse::<f64>() {
                                    model.vars[varid.0].lb = Some(v);
                                    model.vars[varid.0].ub = Some(v);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        for (rname, rtype) in &rows {
            let expr = col_exprs.get(rname).cloned().unwrap_or(LinExpr::constant(0.0));
            let rhs = *rhs_map.get(rname).unwrap_or(&0.0);
            match *rtype {
                'N' => {
                    model.add_objective(Objective::minimize(expr).name("Z"));
                }
                'L' => {
                    model += expr.leq(rhs);
                }
                'G' => {
                    model += expr.geq(rhs);
                }
                'E' => {
                    model += expr.eq(rhs);
                }
                _ => {}
            }
        }

        Ok(model)
    }
}
