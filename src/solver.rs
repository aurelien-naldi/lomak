use crate::func::expr::Expr;

pub mod clingo;

pub enum SolverMode {
    MAX,
    MIN,
    ALL,
}

pub fn get_solver(mode: SolverMode) -> clingo::ClingoProblem {
    clingo::ClingoProblem::new(mode)
}

pub trait Solver {
    fn add_constraint(&mut self, e: &Expr);

    fn solve(&self);
}
