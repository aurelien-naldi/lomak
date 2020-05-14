use std::borrow::BorrowMut;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use crate::func::expr::Expr;
use crate::model::{new_model, LQModelRef, QModel};

mod bnet;
mod boolsim;
mod mnet;

/// A Format may provide import and export filters
pub trait Format: TrySaving + TryParsing {}

/// Denotes an object which may be able to save a model, or not.
/// This is a requirement for the definition of a Format. This trait is
/// automatically derived for structs implementing the SavingFormat trait.
/// An empty implementor enables the definition of a format without an
/// export filter.
pub trait TrySaving {
    fn as_saver(&self) -> Option<&dyn SavingFormat> {
        None
    }
}

/// Denotes an object which may be able to load a model, or not.
/// This is a requirement for the definition of a Format. This trait is
/// automatically derived for structs implementing the ParsingFormat trait.
/// An empty implementor enables the definition of a format without an
/// import filter.
pub trait TryParsing {
    fn as_parser(&self) -> Option<&dyn ParsingFormat> {
        None
    }
}

impl<T: TrySaving + TryParsing> Format for T {}

impl<T: SavingFormat> TrySaving for T {
    fn as_saver(&self) -> Option<&dyn SavingFormat> {
        Some(self)
    }
}

impl<T: ParsingFormat> TryParsing for T {
    fn as_parser(&self) -> Option<&dyn ParsingFormat> {
        Some(self)
    }
}

/// Trait providing the import filter for Formats.
pub trait ParsingFormat {
    fn parse_file(&self, filename: &str) -> Result<LQModelRef, io::Error> {
        // Load the input file into a local string
        let mut unparsed_file = String::new();
        File::open(filename)?.read_to_string(&mut unparsed_file)?;
        let mut model = new_model();
        let m: &mut dyn QModel = model.borrow_mut();
        self.parse_rules(m, &unparsed_file);
        Ok(model)
    }

    fn parse_rules(&self, model: &mut dyn QModel, expression: &str);

    fn parse_formula(&self, model: &mut dyn QModel, formula: &str) -> Result<Expr, String>;
}

/// Trait providing the export filter for Formats.
pub trait SavingFormat {
    fn save_file(&self, model: &dyn QModel, filename: &str) -> Result<(), io::Error> {
        let f = File::create(filename).expect("Could not create the output file");
        let mut out = BufWriter::new(f);
        self.write_rules(model, &mut out)
    }

    fn write_rules(&self, model: &dyn QModel, out: &mut dyn Write) -> Result<(), io::Error>;
}

pub fn get_format(fmt: &str) -> Result<Box<dyn Format>, io::Error> {
    // TODO: select the right format
    match fmt.to_lowercase().trim() {
        "mnet" => Result::Ok(Box::new(mnet::MNETFormat::new())),
        "bnet" => Result::Ok(Box::new(bnet::BNETFormat::new())),
        "bsim" => Result::Ok(Box::new(boolsim::BoolSimFormat::new())),
        _ => Err(io::Error::new(ErrorKind::NotFound, "No matching format")),
    }
}

fn guess_format(filename: &str) -> Result<Box<dyn Format>, io::Error> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not guess the extension"))
        .and_then(get_format)
}

pub fn load_model(filename: &str, fmt: Option<&str>) -> Result<LQModelRef, io::Error> {
    let f = match fmt {
        None => guess_format(filename),
        Some(s) => get_format(s),
    };

    match f {
        Err(e) => Err(e),
        Ok(f) => match f.as_parser() {
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "No parser for this format",
            )),
            Some(f) => f.parse_file(filename),
        },
    }
}

pub fn save_model(model: &dyn QModel, filename: &str, fmt: Option<&str>) -> Result<(), io::Error> {
    let f = match fmt {
        None => guess_format(filename),
        Some(s) => get_format(s),
    };

    match f {
        Err(e) => Err(e),
        Ok(f) => match f.as_saver() {
            None => Err(io::Error::new(
                ErrorKind::NotFound,
                "No exporter for this format",
            )),
            Some(f) => f.save_file(model, filename),
        },
    }
}

pub fn parse_expr(model: &mut dyn QModel, expr: &str) -> Result<Expr, String> {
    let parser = mnet::MNETFormat::new();
    parser.parse_formula(model, expr)
}
