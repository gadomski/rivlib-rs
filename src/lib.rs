//! Interface into Riegl's RiVLib.
//!
//! # Examples
//!
//! Use a `Reader` to extract data from rxp files:
//!
//! ```
//! use rivlib::Reader;
//! let reader = Reader::from_path("data/scan.rxp");
//! ```
//!
//! Use `.points()` to get an iterator over the file's points:
//!
//! ```
//! let mut reader = rivlib::Reader::from_path("data/scan.rxp");
//! let points = reader.points().unwrap().filter_map(|p| p.ok()).collect::<Vec<_>>();
//! ```
//!
//! Use `.inclinations()` to get an iterator over the file's inclination readings:
//!
//! ```
//! let mut reader = rivlib::Reader::from_path("data/scan.rxp");
//! let inclinations = reader.inclinations().unwrap().filter_map(|i| i.ok()).collect::<Vec<_>>();
//! ```

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unstable_features, unused_import_braces, unused_qualifications)]

#[macro_use]
extern crate failure;
extern crate scanifc_sys;
extern crate scanlib;

mod point;
mod reader;
mod scanifc;

pub use scanlib::Inclination;
pub use point::{EchoType, Point};
pub use reader::Reader;
