use crate::func::expr::Expr;
use crate::model::LQModel;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

mod bnet;
mod mnet;

pub trait Format {
    fn as_parser(&self) -> Option<&dyn ParsingFormat> {
        None
    }

    fn as_saver(&self) -> Option<&dyn SavingFormat> {
        None
    }
}

pub trait ParsingFormat {
    fn parse_file(&self, filename: &str) -> Result<LQModel, io::Error> {
        // Load the input file into a local string
        let mut unparsed_file = String::new();
        File::open(filename)?.read_to_string(&mut unparsed_file)?;
        let mut model = LQModel::new();
        self.parse_rules(&mut model, &unparsed_file);
        Ok(model)
    }

    fn parse_rules(&self, model: &mut LQModel, expression: &String);

    fn parse_formula(&self, model: &mut LQModel, formula: &str) -> Result<Expr, String>;
}

pub trait SavingFormat {
    fn save_file(&self, model: &LQModel, filename: &str) -> Result<(), io::Error> {
        let f = File::create(filename).expect("Could not create the output file");
        let mut out = BufWriter::new(f);
        self.write_rules(model, &mut out)
    }

    fn write_rules(&self, model: &LQModel, out: &mut Write) -> Result<(), io::Error>;
}

pub fn get_format(fmt: &str) -> Result<Box<Format>, io::Error> {
    // TODO: select the right format
    match fmt.to_lowercase().trim() {
        "mnet" => Result::Ok(Box::new(mnet::MNETFormat::new())),
        "bnet" => Result::Ok(Box::new(bnet::BNETFormat::new())),
        _ => Err(io::Error::new(ErrorKind::NotFound, "No matching format")),
    }
}

fn guess_format(filename: &str) -> Result<Box<Format>, io::Error> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not guess the extension",
        ))
        .and_then(get_format)
}

pub fn load_model(filename: &str, fmt: Option<&str>) -> Result<LQModel, io::Error> {
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

pub fn save_model(model: &LQModel, filename: &str, fmt: Option<&str>) -> Result<(), io::Error> {
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
