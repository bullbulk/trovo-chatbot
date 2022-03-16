use std::{error, fmt};

#[derive(Debug)]
pub struct InvalidResponse {
    pub code: reqwest::StatusCode,
    pub response: reqwest::Response,
}

impl fmt::Display for InvalidResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Caught an invalid response")
    }
}

impl error::Error for InvalidResponse {}

#[derive(Debug)]
pub struct EmptyError;

impl fmt::Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Empty error")
    }
}

impl error::Error for EmptyError {}

