use std::io;
use std::num;
use thiserror::Error;
use std::fmt;
use clingo;
use crate::model::io::FormatError;


#[derive(Error,Debug)]
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
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Integer parsing error: {0}")]
    ParseInt(#[from] num::ParseIntError),

    #[error("Float parsing error: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
}

pub type LomakResult<T> = Result<T, LomakError>;

pub type EmptyLomakResult = LomakResult<()>;
