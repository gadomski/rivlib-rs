#[macro_use]
extern crate failure;
extern crate libc;

mod inclination;

pub use inclination::Inclination;
use std::path::Path;
use std::ptr;
use std::slice;

macro_rules! scanlib_try {
    ($expr:expr) => {{
        let result = $expr;
        if result != 0 {
            return Err(Error(result).into());
        }
    }}
}

/// An error from our scanlib iterface.
///
/// TODO make the error reporting better.
#[derive(Debug, Fail)]
#[fail(display = "scanlib error: {}", _0)]
pub struct Error(i32);

/// A pointcloud, used for reading data from an rxp file.
#[derive(Debug)]
pub struct Pointcloud {
    stream: *mut ffi::Stream,
}

/// The data returned from one read of a pointcloud.
#[derive(Debug)]
pub struct Data {
    /// The inclination data.
    pub inclinations: Vec<Inclination>,
}

impl Pointcloud {
    /// Opens a pointcloud for the provided path and sync_to_pps setting.
    ///
    /// # Examples
    ///
    /// ```
    /// let pointcloud = scanlib::Pointcloud::from_path("../data/scan.rxp", false).unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        sync_to_pps: bool,
    ) -> Result<Pointcloud, failure::Error> {
        use std::ffi::CString;

        let path = CString::new(path.as_ref().to_string_lossy().as_ref())?;
        let sync_to_pps = if sync_to_pps { 1 } else { 0 };
        let mut stream = ptr::null_mut();
        unsafe {
            scanlib_try!(ffi::stream_new(path.as_ptr(), sync_to_pps, &mut stream));
        }
        Ok(Pointcloud { stream: stream })
    }

    /// Reads data from the rxp stream.
    ///
    /// Returns a data object that holds all data read during this iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut pointcloud = scanlib::Pointcloud::from_path("../data/scan.rxp", false).unwrap();
    /// let data = pointcloud.read().unwrap();
    /// ```
    pub fn read(&self) -> Result<Option<Data>, Error> {
        let mut inclinations = ptr::null();
        let mut inclinations_len = 0;
        let mut end_of_input = 0;
        unsafe {
            scanlib_try!(ffi::stream_read(
                self.stream,
                &mut inclinations,
                &mut inclinations_len,
                &mut end_of_input
            ));
        }
        if end_of_input == 1 {
            Ok(None)
        } else {
            unsafe {
                Ok(Some(Data {
                    inclinations: slice::from_raw_parts(inclinations, inclinations_len)
                        .into_iter()
                        .map(|&i| i.into())
                        .collect(),
                }))
            }
        }
    }

    /// Reads inclination data from the rxp stream.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut pointcloud = scanlib::Pointcloud::from_path("../data/scan.rxp", false).unwrap();
    /// let inclinations = pointcloud.read_inclinations().unwrap().unwrap();
    /// ```
    pub fn read_inclinations(&mut self) -> Result<Option<Vec<Inclination>>, Error> {
        self.read().map(|o| o.map(|data| data.inclinations))
    }
}

impl Drop for Pointcloud {
    fn drop(&mut self) {
        unsafe {
            ffi::stream_del(self.stream);
        }
    }
}

mod ffi {
    use libc;

    #[derive(Debug, Copy, Clone)]
    #[repr(C)]
    pub struct Stream {
        _unused: [u8; 0],
    }

    #[derive(Debug, Copy, Clone)]
    #[repr(C)]
    pub struct Inclination {
        time: f64,
        roll: f32,
        pitch: f32,
    }

    impl From<Inclination> for ::Inclination {
        fn from(incl: Inclination) -> ::Inclination {
            ::Inclination {
                time: incl.time,
                roll: incl.roll,
                pitch: incl.pitch,
            }
        }
    }

    extern "C" {
        pub fn stream_new(
            path: *const libc::c_char,
            sync_to_pps: i32,
            stream: *mut *mut Stream,
        ) -> i32;

        pub fn stream_read(
            stream: *mut Stream,
            inclinations: *mut *const Inclination,
            inclinations_len: &mut libc::size_t,
            end_of_input: &mut i32,
        ) -> i32;

        pub fn stream_del(stream: *mut Stream) -> i32;
    }
}
