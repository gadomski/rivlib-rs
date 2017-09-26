extern crate libc;
extern crate scanifc_sys;
#[cfg(test)]
extern crate tempfile;

#[macro_use]
mod macros;
pub mod point3d;
mod point;

pub use point::Point;

use std::ffi::{IntoStringError, NulError};

// This number was cribbed from the rivlib example.
const LAST_ERROR_BUFFER_SIZE: usize = 512;

/// Our custom error enum.
#[derive(Debug)]
pub enum Error {
    /// An error occurred while getting the last error.
    ///
    /// At this point, all we can do is return the error code from get_last_error.
    GetLastError(libc::c_int),
    /// Wrapper around `std::ffi::IntoStringError`.
    FfiIntoString(IntoStringError),
    /// Wrapper around `std::ffi::NulError`.
    FfiNulError(NulError),
    /// The last error message can't be turned into a string nicely.
    LastErrorMessage(Vec<libc::c_char>),
    /// A internal scanifc error.
    ///
    /// The message is provided by the scanifc library.
    Scanifc(libc::c_int, String),
}

/// Our custom result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The version of the scanifc library.
#[derive(Clone, Copy, Debug, Default)]
pub struct Version {
    /// The major version number.
    pub major: u16,
    /// The minor version number.
    pub minor: u16,
    /// The build version.
    pub build: u16,
}

/// Returns the version number from the library.
///
/// # Examples
///
/// ```
/// let version = scanifc::library_version().unwrap();
/// println!("Version: {}.{}.{}", version.major, version.minor, version.build);
/// ```
pub fn library_version() -> Result<Version> {
    let mut version = Version::default();
    scanifc_try!(scanifc_sys::scanifc_get_library_version(
        &mut version.major,
        &mut version.minor,
        &mut version.build,
    ));
    Ok(version)
}

/// Returns extended version information that allows traceability of the SCM system.
///
/// # Examples
///
/// ```
/// let version = scanifc::library_build_version();
/// ```
pub fn library_build_version() -> Result<String> {
    library_info().map(|(version, _)| version)
}

/// Returns additional information about the build.
///
/// # Examples
///
/// ```
/// let tag = scanifc::library_build_tag();
/// ```
pub fn library_build_tag() -> Result<String> {
    library_info().map(|(_, tag)| tag)
}

/// Returns the last error message recorded by the scanifc library.
///
/// # Examples
///
/// ```
/// let message = scanifc::last_error().unwrap();
/// ```
pub fn last_error() -> Result<String> {
    use std::ffi::CString;

    let mut buffer = vec![0; LAST_ERROR_BUFFER_SIZE];
    let mut message_size = 0;
    let result = unsafe {
        scanifc_sys::scanifc_get_last_error(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut message_size,
        )
    };
    if result != 0 {
        return Err(Error::GetLastError(result));
    }
    let c_string = CString::new(buffer
        .iter()
        .take(message_size as usize)
        .map(|&n| if n < 0 {
            Err(Error::LastErrorMessage(buffer.clone()))
        } else {
            Ok(n as u8)
        })
        .collect::<Result<Vec<u8>>>()?)?;
    c_string.into_string().map_err(Error::from)
}

impl From<IntoStringError> for Error {
    fn from(err: IntoStringError) -> Error {
        Error::FfiIntoString(err)
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Error {
        Error::FfiNulError(err)
    }
}

fn library_info() -> Result<(String, String)> {
    use std::ptr;
    use std::ffi::CStr;

    let mut version: *const libc::c_char = ptr::null();
    let mut tag: *const libc::c_char = ptr::null();
    scanifc_try!(scanifc_sys::scanifc_get_library_info(
        &mut version,
        &mut tag,
    ));
    let version = unsafe { CStr::from_ptr(version) };
    let tag = unsafe { CStr::from_ptr(tag) };
    Ok((
        version.to_string_lossy().into_owned(),
        tag.to_string_lossy().into_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_version_is_ok() {
        assert!(library_version().is_ok());
    }

    #[test]
    fn library_build_version_is_ok() {
        assert!(library_build_version().is_ok());
    }

    #[test]
    fn library_build_tag_is_ok() {
        assert!(library_build_tag().is_ok());
    }
}
