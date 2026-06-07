#let cnvx = plugin("cnvx.wasm")

#let solve-lp(request) = {
  let bytes = cnvx.solve_lp(bytes(json.encode(request)))
  json(bytes)
}

// Define the power grid problem
#let result = solve-lp((
  vars: (
    (name: "Gas", lb: 0.0, ub: 200.0),
    (name: "Coal", lb: 0.0, ub: 180.0),
    (name: "Wind", lb: 0.0, ub: 120.0),
  ),
  constraints: (
    (expr: (Gas: 1.0, Coal: 1.0, Wind: 1.0), op: "eq", rhs: 300.0),
    (expr: (Gas: 1.0, Coal: -0.5), op: "leq", rhs: 0.0),
    (expr: (Gas: 1.0, Coal: 1.0), op: "geq", rhs: 150.0),
  ),
  objective: (
    sense: "minimize",
    expr: (Gas: 50.0, Coal: 80.0),
  ),
))

= Power Grid Dispatch

#table(
  columns: 2,
  [*Variable*], [*Value*],
  [Gas], [#result.values.Gas MW],
  [Coal], [#result.values.Coal MW],
  [Wind], [#result.values.Wind MW],
  [*Total cost*], [*\$#result.objective*],
)
