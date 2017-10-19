#[cfg(test)]
#[macro_use]
extern crate approx;
#[macro_use]
extern crate quick_error;

mod wrapper;

macro_rules! scanlib_try {
    ($expr:expr, $scanlib:expr) => {{
        let result = $expr;
        if result != 0 {
            return Err(last_error(result, $scanlib));
        }
    }}
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Scanlib(n: i32, message: String) {
            description("scanlib error")
            display("scanlib error code {}: {}", n, message)
        }
        FfiNul(err: std::ffi::NulError) {
            from()
            cause(err)
            description(err.description())
            display("ffi nul error: {}", err)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Inclination {
    pub time: f64,
    pub roll: f64,
    pub pitch: f64,
}

fn last_error(n: i32, scanlib: *const wrapper::Scanlib) -> Error {
    use std::ffi::CStr;
    let message = unsafe {
        CStr::from_ptr(wrapper::scanlib_last_error(scanlib))
            .to_string_lossy()
            .into_owned()
    };
    Error::Scanlib(n, message)
}

pub fn inclincations_from_path<P: AsRef<std::path::Path>>(
    path: P,
    sync_to_pps: bool,
) -> Result<Vec<Inclination>> {
    use std::ffi::CString;

    let path = CString::new(path.as_ref().to_string_lossy().as_ref())?;
    let mut inclinations = std::ptr::null_mut();
    let mut scanlib = std::ptr::null_mut();
    let slice = unsafe {
        wrapper::scanlib_new(&mut scanlib);
        scanlib_try!(
            wrapper::inclinations_from_path(scanlib, path.as_ptr(), sync_to_pps, &mut inclinations),
            scanlib
        );
        let mut p = std::ptr::null();
        let mut len = 0;
        wrapper::inclinations_pointer(inclinations, &mut p);
        wrapper::inclinations_len(inclinations, &mut len);
        std::slice::from_raw_parts(p, len as usize)
    };
    let inclinations_vec = slice.to_vec();
    unsafe {
        wrapper::inclinations_drop(inclinations);
        wrapper::scanlib_drop(scanlib);
    }
    Ok(inclinations_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclincations_from_path_fixture() {
        let inclincations = inclincations_from_path("../scanifc/data/scan.rxp", false).unwrap();
        assert_eq!(36, inclincations.len());
        assert_relative_eq!(-8.442, inclincations[0].roll, epsilon = 1e-3);
        assert_relative_eq!(-0.981, inclincations[0].pitch, epsilon = 1e-3);
        assert_relative_eq!(67.7494, inclincations[35].time, epsilon = 1e-4);
        assert_relative_eq!(-8.451, inclincations[35].roll, epsilon = 1e-3);
        assert_relative_eq!(-1.004, inclincations[35].pitch, epsilon = 1e-3);
    }

    #[test]
    fn inclincations_from_path_dne() {
        assert!(inclincations_from_path("notafile", false).is_err());
    }
}
