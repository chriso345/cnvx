import pulp
import cnvx.lp


class CNVX(pulp.LpSolver):
    name = "CNVX_LP"

    def __init__(self, timeLimit=None, msg=True, **kwargs):
        super().__init__(timeLimit=timeLimit, msg=msg, **kwargs)

    def available(self):
        try:
            import cnvx.lp  # noqa: F401

            return True
        except ImportError:
            return False

    def actualSolve(self, lp: pulp.LpProblem, **kwargs):
        model = cnvx.lp.Model()
        var_map = {}

        for v_name, p_var in lp.variablesDict().items():
            lb = (
                p_var.lowBound
                if p_var.lowBound is not None and p_var.lowBound > -float("inf")
                else None
            )
            ub = (
                p_var.upBound
                if p_var.upBound is not None and p_var.upBound < float("inf")
                else None
            )

            if p_var.cat != pulp.LpContinuous and self.msg:
                print(
                    f"Warning: CNVX is an LP solver. Variable '{v_name}' will be relaxed to continuous."
                )

            var_map[v_name] = model.add_var(name=v_name, lb=lb, ub=ub)

        def build_expr_vars(pulp_expr):
            c_expr = None
            for p_var, coeff in pulp_expr.items():
                term = var_map[p_var.name] * float(coeff)
                if c_expr is None:
                    c_expr = term
                else:
                    c_expr += term
            return c_expr

        c_obj = build_expr_vars(lp.objective)
        if c_obj is not None:
            if lp.objective.constant:
                c_obj += float(lp.objective.constant)

            if lp.sense == pulp.LpMinimize:
                model.minimize(c_obj, name=lp.name)
            else:
                model.maximize(c_obj, name=lp.name)

        for c_name, p_constr in lp.constraints.items():
            c_expr = build_expr_vars(p_constr)
            if c_expr is None:
                continue

            rhs = -float(p_constr.constant)

            if p_constr.sense == pulp.LpConstraintEQ:
                model.add_constraint(c_expr == rhs)
            elif p_constr.sense == pulp.LpConstraintLE:
                model.add_constraint(c_expr <= rhs)
            elif p_constr.sense == pulp.LpConstraintGE:
                model.add_constraint(c_expr >= rhs)
            else:
                raise ValueError(f"Unknown constraint sense: {p_constr.sense}")

        try:
            solution = model.solve()

            if solution.objective_value is None:
                lp.status = pulp.LpStatusInfeasible
            else:
                lp.status = pulp.LpStatusOptimal
                for v_name, p_var in lp.variablesDict().items():
                    p_var.varValue = solution.value(var_map[v_name])

        except Exception as e:
            if self.msg:
                print(f"CNVX Solver Error: {e}")
            lp.status = pulp.LpStatusNotSolved

        return lp.status
