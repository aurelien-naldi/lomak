use crate::func::expr::*;
use crate::func::paths::Paths;

impl Expr {
    /// Compute the prime implicants of a Boolean expression.
    pub fn prime_implicants(&self) -> Paths {
        let mut paths = Paths::new();
        self.nnf()
            .as_ref()
            .unwrap_or(self)
            ._prime_implicants(&mut paths);
        paths
    }

    /// Look for prime implicants, dissolving dlinks as we go
    fn _prime_implicants(&self, paths: &mut Paths) {
        if paths.len() < 1 {
            return;
        }

        match self {
            Expr::TRUE => (),
            Expr::FALSE => paths.clear(),
            Expr::ATOM(u) => paths.extend(*u, false),
            Expr::NATOM(u) => paths.extend(*u, true),
            Expr::OPER(Operator::OR, c) => {
                let n = c.len();
                if n < 1 {
                    // An empty disjunction is false
                    paths.clear();
                    return;
                }

                let mut source = paths.clone();
                c.data[0]._prime_implicants(paths);
                for i in 1..n {
                    source.substract(paths);
                    let mut next = source.clone();
                    c.data[i]._prime_implicants(&mut next);
                    paths.merge_raw(&next);
                }
            }
            Expr::OPER(Operator::AND, c) => {
                for ch in c.data.iter() {
                    ch._prime_implicants(paths);
                }
            }
            _ => panic!("Input formula is not in NNF: {}", self),
        };
    }
}
