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

pub mod error;
pub mod scanifc;

/// Crate-specific results.
pub type Result<T> = result::Result<T, error::Error>;
