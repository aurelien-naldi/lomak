use crate::func::expr::Expr;
use crate::model::LQModel;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

mod mnet;

pub trait Format {
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

pub fn get_format(fmt: &str) -> Result<Box<Format>, io::Error> {
    // TODO: select the right format
    Result::Ok(Box::new(mnet::MNETFormat::new()))
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
    let parser = |f: Box<Format>| f.parse_file(filename);
    match fmt {
        None => guess_format(filename),
        Some(s) => get_format(s),
    }
    .and_then(parser)
}
