"""
Simple power generation example using cnvx python bindings.

See ../../examples/lp/power.rs for the original Rust version.
"""

from cnvx import __version__, lp

print(f"cnvx version: {__version__}\n")

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
