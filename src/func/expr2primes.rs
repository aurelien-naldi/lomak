use crate::func::expr::*;
use crate::func::implicant::Implicants;

impl Expr {
    /// Compute the prime implicants of a Boolean expression.
    pub fn prime_implicants(&self) -> Implicants {
        let mut paths = Implicants::new();
        self._prime_implicants_pure(&mut paths);
        paths
    }

    /// Dispatch prime implicant construction to specialized functions depending on the negated status
    fn _prime_implicants(&self, paths: &mut Implicants, neg: bool) {
        if neg {
            self._prime_implicants_neg(paths);
        } else {
            self._prime_implicants_pure(paths);
        }
    }

    /// Look for prime implicants, dissolving dlinks as we go, for non-negated expressions
    fn _prime_implicants_pure(&self, paths: &mut Implicants) {
        if paths.is_empty() {
            return;
        }

        match self {
            Expr::TRUE => (),
            Expr::FALSE => paths.clear(),
            Expr::ATOM(u) => paths.extend_literal(*u, true),
            Expr::NATOM(u) => paths.extend_literal(*u, false),
            Expr::OPER(Operator::OR, c) => Expr::_pi_or(c, paths, false),
            Expr::OPER(Operator::NOR, c) => Expr::_pi_and(c, paths, true),
            Expr::OPER(Operator::AND, c) => Expr::_pi_and(c, paths, false),
            Expr::OPER(Operator::NAND, c) => Expr::_pi_or(c, paths, true),
        };
    }

    /// Look for prime implicants, dissolving dlinks as we go, for negated expressions
    fn _prime_implicants_neg(&self, paths: &mut Implicants) {
        if paths.is_empty() {
            return;
        }

        match self {
            Expr::TRUE => paths.clear(),
            Expr::FALSE => (),
            Expr::ATOM(u) => paths.extend_literal(*u, false),
            Expr::NATOM(u) => paths.extend_literal(*u, true),
            Expr::OPER(Operator::OR, c) => Expr::_pi_and(c, paths, true),
            Expr::OPER(Operator::NOR, c) => Expr::_pi_or(c, paths, false),
            Expr::OPER(Operator::AND, c) => Expr::_pi_or(c, paths, true),
            Expr::OPER(Operator::NAND, c) => Expr::_pi_and(c, paths, false),
        };
    }

    fn _pi_and(c: &Children, paths: &mut Implicants, neg: bool) {
        for ch in c.data.iter() {
            ch._prime_implicants(paths, neg);
        }
    }

    fn _pi_or(c: &Children, paths: &mut Implicants, neg: bool) {
        let n = c.len();
        if n < 1 {
            // An empty disjunction is false
            Expr::FALSE._prime_implicants(paths, neg);
            return;
        }

        if n == 1 {
            c.data[0]._prime_implicants(paths, neg);
            return;
        }

        let mut source = paths.clone();
        c.data[0]._prime_implicants(paths, neg);
        for i in 1..n {
            source.substract(paths);
            let mut next = source.clone();
            c.data[i]._prime_implicants(&mut next, neg);
            paths.merge_raw(&next);
        }
    }
}
