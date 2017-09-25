extern crate libc;
extern crate scanifc_sys;

use std::ffi::{IntoStringError, NulError};

const LAST_ERROR_BUFFER_SIZE: usize = 256;

macro_rules! scanifc_try {
    ($expr:expr) => ({
        let result = unsafe { $expr };
        if result != 0 {
            let msg = last_error()?;
            return Err(Error::Scanifc(result, msg));
        }
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_version_is_ok() {
        assert!(library_version().is_ok());
    }
}
