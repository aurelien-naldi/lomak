use crate::func::expr::Expr;
use crate::func::implicant::Implicants;
use crate::func::state::State;
use crate::model::{GroupedVariables, QModel};
use bit_set::BitSet;
use std::collections::HashMap;

pub struct Reachable {
    info: HashMap<usize, VariableInfo>,
    initial: State,
}

struct VariableInfo {
    pos_implicants: Implicants,
    neg_implicants: Implicants,
    regulators: BitSet,
}

impl VariableInfo {
    fn new(e: &Expr) -> Self {
        let pis = e.prime_implicants();
        let regulators = pis.get_regulators();
        VariableInfo {
            pos_implicants: pis,
            neg_implicants: e.not().prime_implicants(),
            regulators: regulators,
        }
    }
}

impl Reachable {
    pub fn new(model: &QModel, initial: State) -> Self {
        let variables = model.variables().clone();
        let functions: HashMap<usize, VariableInfo> = variables
            .map(|vid| (*vid, VariableInfo::new(&model.get_var_rule(*vid))))
            .collect();

        Reachable {
            info: functions,
            initial: initial,
        }
    }
}
