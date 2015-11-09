//! Wrapper around Riegl's RiVLib.

#![deny(box_pointers, fat_ptr_transmutes, missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts, trivial_numeric_casts, unstable_features, unused_extern_crates, unused_import_braces, unused_qualifications, unused_results)]

extern crate libc;
#[macro_use]
extern crate log;

use std::error::Error;
use std::ffi;
use std::fmt;
use std::result;
use std::str;

pub mod scanifc;

/// Crate-specific errors.
#[derive(Debug)]
pub enum RivlibError {
    /// Wrapper around `std::ffi::NulError`.
    Nul(ffi::NulError),
    /// An error that occurs during a scanifc call.
    ///
    /// Scanifc errors are retrieved from the scanifc library if a ffi function returns nonzero.
    Scanifc(i32, String),
    /// Wrapper around `std::str::Utf8Error`.
    Utf8(str::Utf8Error),
}

impl Error for RivlibError {
    fn description(&self) -> &str {
        match *self {
            RivlibError::Nul(ref err) => err.description(),
            RivlibError::Scanifc(..) => "scanifc error",
            RivlibError::Utf8(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RivlibError::Nul(ref err) => Some(err),
            RivlibError::Scanifc(..) => None,
            RivlibError::Utf8(ref err) => Some(err),
        }
    }
}

impl fmt::Display for RivlibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RivlibError::Nul(ref err) => write!(f, "Nul error: {}", err),
            RivlibError::Scanifc(n, ref s) => write!(f, "Scanlib error {}: {}", n, s),
            RivlibError::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
        }
    }
}

impl From<ffi::NulError> for RivlibError {
    fn from(err: ffi::NulError) -> RivlibError {
        RivlibError::Nul(err)
    }
}

impl From<str::Utf8Error> for RivlibError {
    fn from(err: str::Utf8Error) -> RivlibError {
        RivlibError::Utf8(err)
    }
}

/// Crate-specific results.
pub type Result<T> = result::Result<T, RivlibError>;
