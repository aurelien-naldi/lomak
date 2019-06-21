use clingo::*;
use itertools::Itertools;
use regex::Regex;

use crate::func::paths::LiteralSet;
use crate::solver::SolverMode;
use std::num::ParseIntError;

lazy_static! {
    static ref RE_VAR: Regex = Regex::new(r"v[0-9]+").unwrap();
}

pub struct ClingoProblem {
    minsolutions: bool,
    n: u64,
    ctl: Control,
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
            n: 100,
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

    pub fn solve(&mut self) {
        // ground the base part
        let cfg = self.ctl.configuration().unwrap();
        println!("{:?}", cfg);

        let parts = vec![Part::new("base", &[]).unwrap()];
        self.ctl
            .ground(&parts)
            .expect("Failed to ground a logic program.");

        // get a solve handle
        let mut handle = self
            .ctl
            .solve(SolveMode::YIELD, &[])
            .expect("Failed retrieving solve handle.");

        // loop over all models
        loop {
            handle.resume().expect("Failed resume on solve handle.");
            if let Ok(Some(model)) = handle.model() {
                // get model type
                let model_type = model.model_type().unwrap();

                let type_string = match model_type {
                    ModelType::StableModel => "Stable model",
                    ModelType::BraveConsequences => "Brave consequences",
                    ModelType::CautiousConsequences => "Cautious consequences",
                };

                // get running number of model
                let number = model.number().unwrap();

                println!("{}: {}", type_string, number);
                println!("    {}", model_as_pattern(model));

                if self.n > 0 && number >= self.n {
                    println!("Reached the max model");
                    break;
                }
            } else {
                // stop if there are no more models
                println!("No more models");
                break;
            }
        }

        // close the solve handle
        handle
            .get()
            .expect("Failed to get result from solve handle.");
        handle.close().expect("Failed to close solve handle.");
    }
}

fn model_as_pattern(model: &Model) -> LiteralSet {
    let mut result = LiteralSet::new();

    // retrieve the selected atoms in the model
    let atoms = model
        .symbols(ShowType::ATOMS)
        .expect("Failed to retrieve symbols in the model.");

    for atom in atoms {
        match atom_to_uid(&atom) {
            Ok(u) => result.set(u, false),
            Err(_) => (),
        }
    }

    // retrieve the negated atoms in the model
    let atoms = model
        .symbols(ShowType::COMPLEMENT | ShowType::ATOMS)
        .expect("Failed to retrieve symbols in the model.");

    for atom in atoms {
        match atom_to_uid(&atom) {
            Ok(u) => result.set(u, true),
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
