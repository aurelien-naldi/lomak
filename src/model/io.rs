use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

use crate::helper::error::{EmptyLomakResult, LomakResult};
use crate::model::{QModel, SharedModel};

mod bnet;
mod boolsim;
mod mnet;
mod sbml;

static FORMATS: [&str;4] = ["bnet", "mnet", "bsim", "sbml"];

/// A Format may provide import and export filters
pub trait Format: TrySaving + TryParsing {
    fn description(&self) -> &str;
}

/// Denotes an object which may be able to save a model, or not.
/// This is a requirement for the definition of a Format. This trait is
/// automatically derived for structs implementing the SavingFormat trait.
/// An empty implementor enables the definition of a format without an
/// export filter.
pub trait TrySaving {
    fn as_saver(&self) -> Result<&dyn SavingFormat, FormatError> {
        Err(FormatError::NoWriter())
    }
}

/// Denotes an object which may be able to load a model, or not.
/// This is a requirement for the definition of a Format. This trait is
/// automatically derived for structs implementing the ParsingFormat trait.
/// An empty implementor enables the definition of a format without an
/// import filter.
pub trait TryParsing {
    fn as_parser(&self) -> Result<&dyn ParsingFormat, FormatError> {
        Err(FormatError::NoParser())
    }
}

impl<T: SavingFormat> TrySaving for T {
    fn as_saver(&self) -> Result<&dyn SavingFormat, FormatError> {
        Ok(self)
    }
}

impl<T: ParsingFormat> TryParsing for T {
    fn as_parser(&self) -> Result<&dyn ParsingFormat, FormatError> {
        Ok(self)
    }
}

/// Trait providing the import filter for Formats.
pub trait ParsingFormat {
    fn parse_file(&self, filename: &str) -> LomakResult<SharedModel> {
        // Load the input file into a local string
        let mut unparsed_file = String::new();
        File::open(filename)?.read_to_string(&mut unparsed_file)?;
        self.parse_str(&unparsed_file)
    }

    fn parse_str(&self, expression: &str) -> LomakResult<SharedModel> {
        let mut model = QModel::default();
        self.parse_into_model(&mut model, expression)?;
        Ok(SharedModel::with(model))
    }

    fn parse_into_model(&self, model: &mut QModel, expression: &str) -> EmptyLomakResult;
}

/// Trait providing the export filter for Formats.
pub trait SavingFormat {
    fn save_file(&self, model: &QModel, filename: &str) -> EmptyLomakResult {
        let f = File::create(filename).expect("Could not create the output file");
        let mut out = BufWriter::new(f);
        self.write_rules(model, &mut out)
    }

    fn write_rules(&self, model: &QModel, out: &mut dyn Write) -> EmptyLomakResult;
}

pub fn get_format(fmt: &str) -> Result<Box<dyn Format>, FormatError> {
    match fmt.to_lowercase().trim() {
        "mnet" => Result::Ok(Box::new(mnet::MNETFormat::default())),
        "bnet" => Result::Ok(Box::new(bnet::BNETFormat::default())),
        "bsim" => Result::Ok(Box::new(boolsim::BoolSimFormat::default())),
        "sbml" => Result::Ok(Box::new(sbml::SBMLFormat::default())),
        _ => Err(FormatError::NotFound(fmt.to_owned())),
    }
}

fn guess_format(filename: &str) -> Result<Box<dyn Format>, FormatError> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .ok_or_else(|| FormatError::NotFound("".to_owned()))
        .and_then(get_format)
}

pub fn load_model(filename: &str, fmt: Option<&str>) -> LomakResult<SharedModel> {
    let f = match fmt {
        None => guess_format(filename),
        Some(s) => get_format(s),
    }?;

    let parser = f.as_parser()?;
    parser.parse_file(filename)
}

pub fn save_model(model: &QModel, filename: &str, fmt: Option<&str>) -> EmptyLomakResult {
    let f = match fmt {
        None => guess_format(filename),
        Some(s) => get_format(s),
    }?;

    let writer = f.as_saver()?;
    writer.save_file(model, filename)
}

pub fn print_formats() {
    println!("Available formats (< read, > write):");
    for name in &FORMATS {
        if let Ok(fmt) = get_format(name) {
            let parser = if fmt.as_parser().is_ok() { "<" } else { " "};
            let saver = if fmt.as_saver().is_ok() { ">" } else { " "};
            println!("  {:16} {}{}  {}", name, parser, saver, fmt.description());
        }

    }
}

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Format \"{0}\" not found")]
    NotFound(String),

    #[error("This format has no parser")]
    NoParser(),

    #[error("This format has no writer")]
    NoWriter(),
}
