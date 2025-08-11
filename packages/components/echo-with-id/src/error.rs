use std::string::FromUtf8Error;

use thiserror::Error;

pub type EchoResult<T> = std::result::Result<T, EchoError>;

#[derive(Error, Debug)]
pub enum EchoError {
    #[error("{0:?}")]
    UtfParse(#[from] FromUtf8Error),
}
