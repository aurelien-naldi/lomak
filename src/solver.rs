use crate::func::expr::Expr;

pub mod clingo;

pub fn get_solver() -> clingo::ClingoProblem {
    clingo::ClingoProblem::new()
}

pub trait Solver {

    fn add_constraint(&mut self, e: &Expr);

    fn solve(&self);
}

