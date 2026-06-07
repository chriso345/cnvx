# cnvx-python

**cnvx-python** provides Python bindings for the [cnvx](https://github.com/chriso345/cnvx) optimization library, built with [PyO3](https://pyo3.rs) and [maturin](https://maturin.rs).

> [!WARNING]
> Python bindings are currently in early development and may not yet cover all features of the Rust library. The API is subject to change as development progresses.

---

## Installation

Requires [Rust](https://rustup.rs), [uv](https://docs.astral.sh/uv/), and optionally [just](https://github.com/casey/just/).

`cnvx` is currently only available via source installation. Clone the repository and run the following commands to build and install the Python package from this directory:

With `just`:

```bash
just init
just build
```

Without `just`:

```bash
uv sync
uv run maturin develop
```

---

## Usage

The following example demonstrates how to define and solve a simple linear programming problem using **cnvx-python**:

```python
import cnvx.lp as lp

model = lp.Model()

gas = model.add_var(name="Gas", lb=0.0, ub=200.0)
coal = model.add_var(name="Coal", lb=0.0, ub=180.0)
wind = model.add_var(name="Wind", lb=0.0, ub=120.0)

# Total generation must meet demand
model.add_constraint((gas.expr().__add__(coal.expr()).__add__(wind.expr())).eq(300.0))

# Gas emissions <= 0.5 * coal
model.add_constraint(gas.leq(coal.__mul__(0.5)))

# At least 150 MW from thermal
model.add_constraint((gas.expr().__add__(coal.expr())).geq(150.0))

# Minimize cost: gas=$50, coal=$80, wind=$0
cost = gas.__mul__(50.0).__add__(coal.__mul__(80.0))
model.minimize(cost, name="TotalCost")

solution = model.solve()

print(f"Optimal cost:     ${solution.objective_value:.1f}")
print(f"Gas generation:   {solution.value(gas):.1f} MW")
print(f"Coal generation:  {solution.value(coal):.1f} MW")
print(f"Wind generation:  {solution.value(wind):.1f} MW")
```

Other examples are available in the [examples](examples) directory.

---

## License

Licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
