use crate::func::expr::Expr;
use crate::func::paths::LiteralSet;
use crate::func::paths::Paths;

pub struct PathsMerger {
    solvable: bool,
    req: LiteralSet,
    extra: Vec<Paths>,
}

impl Solver for PathsMerger {

    fn add_constraint(&mut self, e: &Expr) {
        if !self.solvable {
            return;
        }
        println!("Add cst: {}", e);

        let cur = e.prime_implicants();
        if cur.len() == 0 {
            self.solvable = false;
            println!("  ->  impossible");
            return;
        }

        if cur.len() == 1 {
//            self.req.intersect_with(&cur.paths[0]);
        }
    }

    fn solve(&self) {
        unimplemented!()
    }
}
