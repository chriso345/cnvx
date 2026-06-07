use cnvx_lp::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_minimal_protocol::*;

initiate_protocol!();

// Request types

#[derive(Deserialize)]
struct VarDef {
    name: String,
    lb: Option<f64>,
    ub: Option<f64>,
}

#[derive(Deserialize)]
struct ConstraintDef {
    expr: HashMap<String, f64>,
    op: String,
    rhs: f64,
}

#[derive(Deserialize)]
struct ObjectiveDef {
    sense: String,
    expr: HashMap<String, f64>,
}

#[derive(Deserialize)]
struct SolveRequest {
    vars: Vec<VarDef>,
    constraints: Vec<ConstraintDef>,
    objective: ObjectiveDef,
}

// Response types

#[derive(Serialize)]
struct SolveResponse {
    status: String,
    objective: Option<f64>,
    values: HashMap<String, f64>,
    error: Option<String>,
}

impl SolveResponse {
    fn error(msg: impl Into<String>) -> Self {
        Self {
            status: "error".into(),
            objective: None,
            values: HashMap::new(),
            error: Some(msg.into()),
        }
    }
}

// Plugin entry point
#[wasm_func]
pub fn solve_lp(input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let req = serde_json::from_slice::<SolveRequest>(input)?;
    let response = run_solve(req);
    Ok(serde_json::to_vec(&response)?)
}

fn run_solve(req: SolveRequest) -> SolveResponse {
    let mut model = LpModel::new();
    let mut name_to_var: HashMap<String, VarId> = HashMap::new();

    for def in &req.vars {
        let b = model.add_var();
        let id = b.var;
        drop(b);
        let var = &mut model.vars[id.0];
        var.name = Some(def.name.clone());
        var.lb = def.lb;
        var.ub = def.ub;
        name_to_var.insert(def.name.clone(), id);
    }

    for def in &req.constraints {
        let expr = build_expr(&def.expr, &name_to_var);
        let constraint = match def.op.as_str() {
            "eq" => expr.eq(def.rhs),
            "leq" => expr.leq(def.rhs),
            "geq" => expr.geq(def.rhs),
            op => return SolveResponse::error(format!("unknown op: {op}")),
        };
        model += constraint;
    }

    let obj_expr = build_expr(&req.objective.expr, &name_to_var);
    let objective = match req.objective.sense.as_str() {
        "minimize" => Objective::minimize(obj_expr).name("objective"),
        "maximize" => Objective::maximize(obj_expr).name("objective"),
        s => return SolveResponse::error(format!("unknown sense: {s}")),
    };
    model.add_objective(objective);

    match LpSolver::new().solve(&model) {
        Ok(sol) => SolveResponse {
            status: "optimal".into(),
            objective: sol.objective_value,
            values: name_to_var
                .iter()
                .map(|(name, id)| (name.clone(), sol.value(*id)))
                .collect(),
            error: None,
        },
        Err(e) => SolveResponse {
            status: "error".into(),
            objective: None,
            values: HashMap::new(),
            error: Some(e.to_string()),
        },
    }
}

fn build_expr(terms: &HashMap<String, f64>, vars: &HashMap<String, VarId>) -> LinExpr {
    terms.iter().fold(LinExpr::constant(0.0), |acc, (name, &coeff)| {
        if let Some(&id) = vars.get(name) { acc + LinExpr::new(id, coeff) } else { acc }
    })
}
