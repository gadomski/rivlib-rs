//! Our custom error handling.

use std::error;
use std::ffi::NulError;
use std::fmt;
use std::str::Utf8Error;

/// Crate-specific errors.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around `std::ffi::NulError`.
    Nul(NulError),
    /// An error that occurs during a scanifc call.
    ///
    /// Scanifc errors are retrieved from the scanifc library if a ffi function returns nonzero.
    Scanifc(i32, String),
    /// Wrapper around `std::str::Utf8Error`.
    Utf8(Utf8Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Nul(ref err) => err.description(),
            Error::Scanifc(..) => "scanifc error",
            Error::Utf8(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Nul(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Nul(ref err) => write!(f, "Nul error: {}", err),
            Error::Scanifc(n, ref s) => write!(f, "Scanlib error {}: {}", n, s),
            Error::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
        }
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Error {
        Error::Nul(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::Utf8(err)
    }
}
