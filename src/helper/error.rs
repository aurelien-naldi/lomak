//! Simple error types and helpers for consistent error handling.
//!
//! It uses the thiserror crate to reduce boilerplate.
use clingo;
use roxmltree;
use std::error::Error;
use std::fmt;
use std::io;
use std::num;
use thiserror::Error;

use crate::model::io::FormatError;

#[derive(Error, Debug)]
pub enum LomakError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Format error: {0}")]
    Format(#[from] FormatError),

    #[error("Parsing error: {0}")]
    Parse(#[from] ParseError),

    #[error("Clingo error {0:?}")]
    Clingo(#[from] clingo::ClingoError),

    #[error("No model was provided")]
    MissingModel(),

    #[error(transparent)]
    Generic(#[from] GenericError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Integer parsing error: {0}")]
    ParseInt(#[from] num::ParseIntError),

    #[error("Float parsing error: {0}")]
    ParseFloat(#[from] num::ParseFloatError),

    #[error("Error parsing XML document: {0}")]
    ParseXML(#[from] roxmltree::Error),

    #[error("Error parsing text document: {0}")]
    ParseText(#[from] ParseTxtError),
}

#[derive(Error, Debug)]
pub struct GenericError {
    s: String,
}

#[derive(Error, Debug)]
pub struct ParseTxtError {
    source: Box<dyn Error>,
}

impl GenericError {
    pub fn new(s: String) -> Self {
        GenericError { s: s }
    }
}

impl ParseTxtError {
    pub fn new(e: Box<impl Error + 'static>) -> Self {
        ParseTxtError { source: e }
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl fmt::Display for ParseTxtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

pub type LomakResult<T> = Result<T, LomakError>;

pub type EmptyLomakResult = LomakResult<()>;

impl<R: pest::RuleType + 'static> From<pest::error::Error<R>> for LomakError {
    fn from(e: pest::error::Error<R>) -> Self {
        let e = ParseTxtError::new(Box::new(e));
        let e: ParseError = e.into();
        e.into()
    }
}

pub fn generic_error(s: String) -> LomakError {
    LomakError::Generic( GenericError::new(s) )
}