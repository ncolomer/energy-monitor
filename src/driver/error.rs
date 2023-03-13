use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "failed to parse frame")
    }
}

impl Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
pub struct CommError;

impl Display for CommError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "communication with device failed")
    }
}

impl Error for CommError {}
