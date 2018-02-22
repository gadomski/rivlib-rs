//! Interface into Riegl's RiVLib.

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
