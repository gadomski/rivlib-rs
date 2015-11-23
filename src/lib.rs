//! Read points from Riegl's rxp format.
//!
//! You need to have RiVLib on your system in order to use this library.  Get RiVLib from [the
//! Riegl website](http://www.riegl.com/).
//!
//! `rxp` is a data layout specification that Riegl uses for streaming discrete point data. The
//! format is used both to store data in files and to stream data over a network.
//!
//! Riegl does not publish the specification for the `rxp` format, so we must use their RiVLib to
//! read `rxp` data.

#![deny(box_pointers, fat_ptr_transmutes, missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts, trivial_numeric_casts, unstable_features, unused_extern_crates, unused_import_braces, unused_qualifications, unused_results)]

extern crate libc;
#[macro_use]
extern crate log;

use std::result;

macro_rules! scanifc_try {
    ($x:expr) => {
        unsafe {
            match $x {
                0 => {},
                n @ _ => return Err(Error::Scanifc(n, last_error())),
            }
        }
    };
}

pub mod error;
mod scanifc;
pub mod stream;

pub use error::Error;
pub use stream::{Stream, Point};

use std::ffi::CStr;
use std::ptr;

use libc::c_char;

use scanifc::last_error;

/// Returns the scanifc library version as a three-tuple.
///
/// # Examples
///
/// ```
/// use rivlib;
/// let (major, minor, build) = rivlib::library_version().unwrap();
/// ```
pub fn library_version() -> Result<(u16, u16, u16)> {
    let mut major = 0u16;
    let mut minor = 0u16;
    let mut build = 0u16;
    scanifc_try!(scanifc::scanifc_get_library_version(&mut major, &mut minor, &mut build));
    Ok((major, minor, build))
}

/// Returns information about the library's build version and tag.
///
/// # Examples
///
/// ```
/// use rivlib;
/// let (build_version, build_tag) = rivlib::library_info().unwrap();
/// ```
pub fn library_info<'a>() -> Result<(&'a str, &'a str)> {
    let mut build_version: *const c_char = ptr::null();
    let mut build_tag: *const c_char = ptr::null();
    scanifc_try!(scanifc::scanifc_get_library_info(&mut build_version, &mut build_tag));
    unsafe {
        Ok((try!(CStr::from_ptr(build_version).to_str()),
        try!(CStr::from_ptr(build_tag).to_str())))
    }
}


/// Crate-specific results.
pub type Result<T> = result::Result<T, Error>;
