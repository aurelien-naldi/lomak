use crate::func::expr::Expr;
use crate::func::implicant::Implicants;
use crate::func::pattern::Pattern;
use crate::func::state::State;
use crate::model::{GroupedVariables, QModel};
use bit_set::BitSet;
use std::collections::HashMap;

/// Identify the smallest fixed pattern containing a given state.
///
/// A target state outside of this pattern can never be reached.
///
/// A fixed pattern overlapping with this pattern is reachable
/// in the buffered and most-permissive semantics.
pub fn enclosing_trapspace(model: &QModel, initial: State) -> Pattern {
    let rules = pick_rules(model, |v| initial.contains(v));
    let variables = model.variables().copied().collect::<Vec<_>>();
    extend(&variables, &rules, &initial, &BitSet::new())
}

/// Check if a target state is reachable using the most-permissive semantics.
///
/// This will test if the target is included in the enclosing stable pattern
/// and attempt to revert the variables which need to go back to their initial state.
/// If any variable can not be reverted, it will freeze it and retry
pub fn most_permissive_reach(model: &QModel, initial: State, target: State) -> bool {
    let rules = pick_rules(model, |v| initial.contains(v));
    let rev_rules = pick_rules(model, |v| !initial.contains(v));
    let variables = model.variables().copied().collect::<Vec<usize>>();

    // Compute the enclosing trapspace
    let mut frozen = BitSet::new();
    loop {
        let enclosing = extend(&variables, &rules, &initial, &frozen);
        if !enclosing.contains_state(&target) {
            return false;
        }

        // Check if updated variables which should return to their initial state can be reverted
        // if it fails, extend the frozen set and retry
        let mut failed = false;
        for v in &variables {
            let v = *v;
            if !enclosing.is_fixed(v)
                && initial.contains(v) == target.contains(v)
                && !rev_rules.get(&v).unwrap().eval_in_pattern(&enclosing)
            {
                failed = true;
                frozen.insert(v);
            }
        }

        if !failed {
            return true;
        }

        // If we get here, the frozen set has been extended and we need to retry
    }
}

fn pick_rule(expr: &Expr, pos: bool) -> Implicants {
    if pos {
        expr.not().prime_implicants()
    } else {
        expr.prime_implicants()
    }
}

fn pick_rules<F: Fn(usize) -> bool>(model: &QModel, pos: F) -> HashMap<usize, Implicants> {
    model
        .variables()
        .map(|vid| (*vid, pick_rule(&model.get_var_rule(*vid), pos(*vid))))
        .collect()
}

fn extend(
    variables: &[usize],
    rules: &HashMap<usize, Implicants>,
    initial: &State,
    frozen: &BitSet,
) -> Pattern {
    // Compute the enclosing trapspace by extending the initial state as much as possible
    let mut enclosing = Pattern::from_state(&initial, variables.iter());
    let mut changed = true;
    while changed {
        changed = false;
        for uid in variables {
            if frozen.contains(*uid) || !enclosing.is_fixed(*uid) {
                continue;
            }
            // Test if this variable can be extended
            if rules.get(uid).unwrap().eval_in_pattern(&enclosing) {
                enclosing.release(*uid);
                changed = true;
            }
        }
    }
    enclosing
}
