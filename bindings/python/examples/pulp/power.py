import pulp
from cnvx.pulp import CNVX

# 1. Create standard PuLP problem
prob = pulp.LpProblem("Power_Generation", pulp.LpMinimize)

# 2. Match variables & upper bounds from power.rs
gas = pulp.LpVariable("gas", lowBound=0, upBound=100)
coal = pulp.LpVariable("coal", lowBound=0, upBound=120)
wind = pulp.LpVariable("wind", lowBound=0, upBound=120)

# 3. Match costs from power.rs
prob += 100.0 * gas + 50.0 * coal + 5.0 * wind, "Total_Cost"

# 4. Total demand constraint
prob += gas + coal + wind >= 300.0, "Demand"

# 5. Solve using CNVX backend
print("Solving with CNVX...")
status = prob.solve(CNVX(msg=True))

# 6. Print Results
print(f"Status: {pulp.LpStatus[status]}")
print(f"Optimal cost: {pulp.value(prob.objective):.0f}")
print(f"Gas generation: {gas.varValue:.0f}")
print(f"Coal generation: {coal.varValue:.0f}")
print(f"Wind generation: {wind.varValue:.0f}")
