use thiserror::Error;
use std::convert::From;
use serde::de;
pub mod api;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to create an HTTP client")]
    ClientError(String),
    #[error("Error occurred during a request")]
    RequestError(String),
    #[error("Deserializing reponse failed with the following error: {0}")]
    DeserializeError(String),
    #[error("Unable to format an input date string")]
    FormatDateError(String),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        let val_print = value.to_string();
        if value.is_builder() {
            return Error::ClientError(val_print);
        }
        if value.is_request() {
            return Error::RequestError(val_print);
        }
        Error::DeserializeError(val_print)
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self where T:std::fmt::Display {
        Error::DeserializeError(msg.to_string())
    }
}

impl From<time::error::Format> for Error {
    fn from(value: time::error::Format) -> Self {
        Error::FormatDateError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
