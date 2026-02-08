# cnvx

**cnvx** is a Rust library for modeling and solving optimization problems. It provides a clean and flexible API with an emphasis on clarity and performance.

Additionally, **cnvx** offers a command-line interface (CLI) for solving models directly from files in various formats, making it easy to experiment without writing Rust code.

---

## Features

* Define decision variables with bounds, types (continuous, binary, integer), and custom constraints.
* Add linear or convex objectives with expressive operators.
* Modular API with prelude imports for ergonomic usage.
* Lightweight and extendable for integrating new solvers and algorithms.
* CLI support for solving optimization models from standard formats like **GMPL**, or **MPS**.

### Cargo Features

**cnvx** can be customized using Cargo features:

* `lp` – enable linear programming solvers (**default**)
* `mop` – enable multi-objective problem support (planned)
* `sat` – enable satisfiability problem support (planned)
* `nlp` – enable non-linear problem support (planned)

These features allow the use of specific solvers and functionalities while keeping the core library lightweight.

---

## Installation

**cnvx** can be used either as a library in your Rust projects or via its command-line interface (CLI).

```bash
# Library usage
cargo add cnvx --features "lp"

# CLI usage
cargo install cnvx-cli
```

---

## Usage

The following example demonstrates how to define and solve a simple linear programming problem using **cnvx**:

```rust
use cnvx::prelude::*;

fn main() -> Result<(), SolveError> {
  // Create a new optimization model
  let mut model = Model::new();

  // Variable definitions
  let x = model.add_var().finish();
  let y = model.add_var().finish();

  // Objective function
  model.add_objective(Objective::maximize(3.0 * x + 2.0 * y).name("profit"));

  // Constraints
  model += (x + y).eq(4.0);
  model += (2.0 * x + 3.0 * y).eq(9.0);

  // Solve the model using the Simplex solver
  let solver = SimplexSolver::default();
  let sol = solver.solve(&model)?;

  println!(
    "Solution: x = {}, y = {}, objective = {}",
    sol.value(x),
    sol.value(y),
    sol.objective_value.unwrap_or_default()
  );

  Ok(())
}
```

---

## License

Licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
