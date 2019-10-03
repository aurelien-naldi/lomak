use clingo::*;
use itertools::Itertools;
use regex::Regex;

use crate::func::paths::LiteralSet;
use crate::solver::SolverMode;
use std::num::ParseIntError;

use std::fmt;

lazy_static! {
    static ref RE_VAR: Regex = Regex::new(r"v[0-9]+").unwrap();
}

pub struct ClingoProblem {
    minsolutions: bool,
    ctl: Control,
}

pub struct ClingoResults<'a> {
    handle: Option<SolveHandle<'a>>,
    halved: bool,
}

pub struct ClingoResultModel {
    number: u64,
    model_type: ModelType,
    pattern: LiteralSet,
}

impl ClingoProblem {
    pub fn new(mode: SolverMode) -> Self {
        let mut args = vec!["-n", "0"];

        // Set the adapted clingo flags:
        //   To find terminal trapspaces: --enum-mode=domRec --heuristic=Domain --dom-mod=3,16
        //   To find minimal trapspaces:
        match mode {
            SolverMode::MAX => args.append(&mut vec![
                "--enum-mode=domRec",
                "--heuristic=Domain",
                "--dom-mod=3,16",
            ]),
            SolverMode::MIN => args.append(&mut vec![
                "--enum-mode=domRec",
                "--heuristic=Domain",
                "--dom-mod=5,16",
            ]),
            SolverMode::ALL => (),
        }

        ClingoProblem {
            minsolutions: false,
            ctl: Control::new(args.into_iter().map(|s| String::from(s)).collect())
                .expect("Failed creating Control."),
        }
    }

    pub fn minsolution(mut self, b: bool) -> Self {
        self.minsolutions = b;
        self
    }

    pub fn add(&mut self, instruct: &str) {
        self.ctl
            .add("base", &[], instruct)
            .expect("Failed creating Control.");
    }

    pub fn restrict(&mut self, p: &LiteralSet) {
        let s = p
            .positive()
            .iter()
            .map(|u| format!("v{}", u))
            .chain(p.negative().iter().map(|u| format!("not v{}", u)))
            .join(",");

        self.add(&format!(":- {}.", s));
    }

    pub fn solve(&mut self) -> ClingoResults {
        // ground the base part
        let cfg = self.ctl.configuration().unwrap();

        let parts = vec![Part::new("base", &[]).unwrap()];
        self.ctl
            .ground(&parts)
            .expect("Failed to ground a logic program.");

        // get a solve handle
        let handle = self
            .ctl
            .solve(SolveMode::YIELD, &[])
            .expect("Failed retrieving solve handle.");

        ClingoResults {
            handle: Some(handle),
            halved: false,
        }
    }
}

impl ClingoResults<'_> {
    pub fn set_halved(&mut self) {
        self.halved = true;
    }
}

impl Drop for ClingoResults<'_> {
    fn drop(&mut self) {
        if self.handle.is_none() {
            return;
        }

        self.handle
            .take()
            .unwrap()
            .close()
            .expect("Failed to close solve handle.");
    }
}

impl Iterator for ClingoResults<'_> {
    type Item = ClingoResultModel;

    fn next(&mut self) -> Option<ClingoResultModel> {
        if self.handle.is_none() {
            return None;
        }
        let handle = self.handle.as_mut().unwrap();
        handle.resume().expect("Failed resume on solve handle.");
        if let Ok(Some(model)) = handle.model() {
            return Some(ClingoResultModel {
                number: model.number().unwrap(),
                model_type: model.model_type().unwrap(),
                pattern: model_as_pattern(model, self.halved),
            });
        }
        None
    }
}

impl fmt::Display for ClingoResultModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_string = match self.model_type {
            ModelType::StableModel => "Stable",
            ModelType::BraveConsequences => "Brave",
            ModelType::CautiousConsequences => "Cautious",
        };

        write!(f, "{} {:4}: {}", type_string, self.number, self.pattern)
    }
}

fn model_as_pattern(model: &Model, halved: bool) -> LiteralSet {
    if halved {
        model_as_half_pattern(model)
    } else {
        model_as_full_pattern(model)
    }
}

fn model_as_full_pattern(model: &Model) -> LiteralSet {
    let mut result = LiteralSet::new();

    // retrieve the selected atoms in the model
    let atoms = model
        .symbols(ShowType::ATOMS)
        .expect("Failed to retrieve symbols in the model.");

    for atom in atoms {
        match atom_to_uid(&atom) {
            Ok(u) => result.set_literal(u, true),
            Err(_) => (),
        }
    }

    // retrieve the negated atoms in the model
    let atoms = model
        .symbols(ShowType::COMPLEMENT | ShowType::ATOMS)
        .expect("Failed to retrieve symbols in the model.");

    for atom in atoms {
        match atom_to_uid(&atom) {
            Ok(u) => result.set_literal(u, false),
            Err(_) => (),
        }
    }

    result
}

fn model_as_half_pattern(model: &Model) -> LiteralSet {
    let mut result = LiteralSet::new();

    // retrieve the selected atoms in the model
    let atoms = model
        .symbols(ShowType::ATOMS)
        .expect("Failed to retrieve symbols in the model.");

    for atom in atoms {
        match atom_to_uid(&atom) {
            Ok(u) => result.set_literal(u / 2, u % 2 == 0),
            Err(_) => (),
        }
    }

    result
}

fn atom_to_uid(atom: &Symbol) -> Result<usize, ParseIntError> {
    let name = atom.to_string().unwrap();
    if name.starts_with("v") {
        let s = &name[1..].to_string();
        return s.parse::<usize>();
    }

    name.parse::<usize>()
}
