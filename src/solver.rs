use std::fmt;

use crate::func::paths::LiteralSet;

mod clingo;

#[derive(Clone, Copy, PartialEq)]
pub enum SolverMode {
    MAX,
    MIN,
    ALL,
}

pub struct SolverSolution {
    number: u64,
    pattern: LiteralSet,
}

pub fn get_solver(mode: SolverMode) -> Box<dyn Solver> {
    Box::new(clingo::ClingoProblem::new(mode))
}

pub trait Solver {
    fn restrict(&mut self, p: &LiteralSet);

    fn add(&mut self, instruct: &str);

    fn solve<'a>(&'a mut self) -> Box<dyn SolverResults + 'a>;
}

pub trait SolverResults<'a>: Iterator<Item = SolverSolution> {
    fn set_halved(&mut self);
}

impl SolverSolution {
    pub fn filter(mut self, filter: &Option<Vec<usize>>) -> SolverSolution {
        if let Some(uids) = filter {
            self.pattern = self.pattern.filter_map(uids);
        }
        self
    }
}

impl fmt::Display for SolverSolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:4}: {}", self.number, self.pattern)
    }
}
