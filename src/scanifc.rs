//! Wrapper around the scanifc library.
//!
//! `scanifc` is used to retrieve point data from a stream.

use std::ffi::{CStr, CString};
use std::iter::IntoIterator;
use std::ptr;

use libc::{c_char, c_int, c_float, int32_t, uint16_t, uint32_t, uint64_t};

use super::{Result, RivlibError};

macro_rules! scanifc_try {
    ($x:expr) => {
        unsafe {
            match $x {
                0 => {},
                n @ _ => return Err(RivlibError::Scanifc(n, last_error())),
            }
        }
    };
}

const MESSAGE_BUFFER_SIZE: uint32_t = 256;
const DEFAULT_POINT_ITERATOR_WANT: u32 = 1000;

/// Returns the scanifc library version as a three-tuple.
///
/// # Examples
///
/// ```
/// use rivlib::scanifc;
/// let (major, minor, build) = scanifc::library_version().unwrap();
/// ```
pub fn library_version() -> Result<(u16, u16, u16)> {
    let mut major = 0u16;
    let mut minor = 0u16;
    let mut build = 0u16;
    scanifc_try!(scanifc_get_library_version(&mut major, &mut minor, &mut build));
    Ok((major, minor, build))
}

/// Returns information about the library's build version and tag.
///
/// # Examples
///
/// ```
/// use rivlib::scanifc;
/// let (build_version, build_tag) = scanifc::library_info().unwrap();
/// ```
pub fn library_info<'a>() -> Result<(&'a str, &'a str)> {
    let mut build_version: *const c_char = ptr::null();
    let mut build_tag: *const c_char = ptr::null();
    scanifc_try!(scanifc_get_library_info(&mut build_version, &mut build_tag));
    unsafe {
        Ok((try!(CStr::from_ptr(build_version).to_str()),
            try!(CStr::from_ptr(build_tag).to_str())))
    }
}

/// A 3D pointcloud data stream.
#[derive(Clone, Copy, Debug)]
pub struct Stream {
    handle: point3dstream_handle,
}

impl Stream {
    /// Opens a data stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::scanifc;
    /// let stream = scanifc::Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// ```
    pub fn open<T: Into<Vec<u8>>>(uri: T, sync_to_pps: bool) -> Result<Stream> {
        let uri = try!(CString::new(uri)).into_raw();
        let mut h3ds: point3dstream_handle = ptr::null_mut();
        unsafe {
            let retval = scanifc_point3dstream_open(uri, sync_to_pps as i32, &mut h3ds);
            let _ = CString::from_raw(uri);
            if retval != 0 {
                return Err(RivlibError::Scanifc(retval, last_error()));
            }
        }

        Ok(Stream { handle: h3ds })
    }

    /// Adds a demultiplexer to the data stream.
    ///
    /// The demultiplexer will write selected packages from the rxpstream to a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::scanifc;
    /// let mut stream = scanifc::Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// stream.add_demultiplexer("/dev/null", "beam_geometry", "all");
    /// ```
    pub fn add_demultiplexer(&mut self,
                             filename: &str,
                             selections: &str,
                             classes: &str)
                             -> Result<()> {
        let filename = try!(CString::new(filename)).into_raw();
        let selections = try!(CString::new(selections)).into_raw();
        let classes = try!(CString::new(classes)).into_raw();
        unsafe {
            let retval = scanifc_point3dstream_add_demultiplexer(self.handle,
                                                                 filename,
                                                                 selections,
                                                                 classes);
            let _ = CString::from_raw(filename);
            let _ = CString::from_raw(selections);
            let _ = CString::from_raw(classes);
            if retval != 0 {
                return Err(RivlibError::Scanifc(retval, last_error()));
            }
        }
        Ok(())
    }

    /// Override the default rangegate of the sensor.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::scanifc;
    /// let mut stream = scanifc::Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// stream.set_rangegate(1, 0.0, 1.0);
    /// ```
    pub fn set_rangegate(&mut self, zone: u16, near: f32, far: f32) -> Result<()> {
        scanifc_try!(scanifc_point3dstream_set_rangegate(self.handle, zone, near, far));
        Ok(())
    }

    /// Reads some points from this stream.
    ///
    /// Returns a two-tuple. The first element is a vector of `Point`s, the second is a boolean
    /// indicating if the end of the stream has been reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::scanifc;
    /// let mut stream = scanifc::Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// let (points, end_of_frame) = stream.read(10).unwrap();
    /// ```
    pub fn read(&mut self, want: u32) -> Result<(Vec<Point>, bool)> {
        let mut pxyz32: Vec<xyz32> = Vec::with_capacity(want as usize);
        let mut pattributes: Vec<attributes> = Vec::with_capacity(want as usize);
        let mut ptime: Vec<uint64_t> = Vec::with_capacity(want as usize);
        let mut got = 0u32;
        let mut end_of_frame: int32_t = 0;
        scanifc_try!(scanifc_point3dstream_read(self.handle,
                                                want,
                                                pxyz32.as_mut_ptr(),
                                                pattributes.as_mut_ptr(),
                                                ptime.as_mut_ptr(),
                                                &mut got,
                                                &mut end_of_frame));

        unsafe {
            pxyz32.set_len(got as usize);
            pattributes.set_len(got as usize);
            ptime.set_len(got as usize);
        }

        let points: Vec<_> = pxyz32.iter()
                                   .zip(pattributes.iter())
                                   .zip(ptime.iter())
                                   .map(|((p, a), t)| {
                                       Point {
                                           x: p.x,
                                           y: p.y,
                                           z: p.z,
                                           amplitude: a.amplitude,
                                           reflectance: a.reflectance,
                                           deviation: a.deviation,
                                           echo_type: EchoType::from_u16(a.flags & 4),
                                           reserved: (a.flags >> 2) & 1 == 1,
                                           waveform_available: (a.flags >> 3) & 1 == 1,
                                           pseudo_echo: (a.flags >> 4) & 1 == 1,
                                           calculated_target: (a.flags >> 5) & 1 == 1,
                                           new_pps: (a.flags >> 6) & 1 == 1,
                                           pps: (a.flags >> 7) & 1 == 1,
                                           facet: Facet(((a.flags >> 8) & 3) as u8),
                                           reserved2: (a.flags >> 10) as u8,
                                           time: *t,
                                       }
                                   })
                                   .collect();
        Ok((points, end_of_frame != 0))
    }
}

impl IntoIterator for Stream {
    type Item = Point;
    type IntoIter = PointIterator;
    fn into_iter(self) -> Self::IntoIter {
        PointIterator {
            stream: self,
            points: Vec::new(),
            want: DEFAULT_POINT_ITERATOR_WANT,
        }
    }
}

/// An iterater over a stream's points.
#[derive(Debug)]
pub struct PointIterator {
    stream: Stream,
    points: Vec<Point>,
    want: u32,
}

impl Iterator for PointIterator {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        if self.points.is_empty() {
            let (points, eof) = self.stream.read(self.want).unwrap();
            if eof {
                return None;
            }
            self.points = points;
        }
        self.points.pop()
    }
}

/// A 3D point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// The x value of the point.
    pub x: c_float,
    /// The y value of the point.
    pub y: c_float,
    /// The z value of the point.
    pub z: c_float,
    /// Relative amplitude in [dB].
    pub amplitude: c_float,
    /// Relative reflectance in [dB].
    pub reflectance: c_float,
    /// A measure of pulse shape distortion.
    pub deviation: uint16_t,
    /// What kind of echo is this point?
    pub echo_type: EchoType,
    /// Reserved.
    pub reserved: bool,
    /// Is there a waveform available for this point?
    pub waveform_available: bool,
    /// Is this point a pseudo echo with fixed range 0.1m?
    pub pseudo_echo: bool,
    /// Is this point a sw calculated target?
    pub calculated_target: bool,
    /// Is this PPS not older than 1.5 sec?
    pub new_pps: bool,
    /// Is this time in the PPS timeframe?
    pub pps: bool,
    /// The facet number.
    pub facet: Facet,
    /// Some more reserved bits
    pub reserved2: u8,
    /// The time of this point.
    pub time: uint64_t,
}

/// A type of echo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EchoType {
    /// Single echo.
    Single,
    /// First echo.
    First,
    /// Interior echo.
    Interior,
    /// Last echo.
    Last,
}

impl EchoType {
    /// Maps a u16 onto an `EchoType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::scanifc::EchoType;
    /// assert_eq!(EchoType::Single, EchoType::from_u16(0));
    /// ```
    pub fn from_u16(n: u16) -> EchoType {
        match n {
            0 => EchoType::Single,
            1 => EchoType::First,
            2 => EchoType::Interior,
            3 => EchoType::Last,
            _ => unreachable!(),
        }
    }
}

/// The facet of this point
#[derive(Clone, Copy, Debug)]
pub struct Facet(u8);

fn last_error() -> String {
    let mut message_buffer: Vec<u8> = Vec::with_capacity(MESSAGE_BUFFER_SIZE as usize);
    let mut message_size: uint32_t = 0;
    unsafe {
        if scanifc_get_last_error(message_buffer.as_mut_ptr() as *mut c_char,
                                  MESSAGE_BUFFER_SIZE,
                                  &mut message_size) != 0 {
            panic!("`scanifc_get_last_error` threw an error, panicing.");
        }
    }
    if message_size > MESSAGE_BUFFER_SIZE {
        warn!("Message size was larger than message buffer size, truncating message ({} > {})",
              message_size,
              MESSAGE_BUFFER_SIZE);
    }
    CString::new(message_buffer).unwrap().to_str().unwrap().to_string()
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct xyz32 {
    x: c_float,
    y: c_float,
    z: c_float,
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct attributes {
    amplitude: c_float,
    reflectance: c_float,
    deviation: uint16_t,
    flags: uint16_t,
    background_radiatino: c_float,
}

#[allow(non_camel_case_types)]
enum point3dstream {}
#[allow(non_camel_case_types)]
type point3dstream_handle = *mut point3dstream;

#[link(name = "scanifc-mt")]
extern {
    fn scanifc_get_library_version(major: *mut uint16_t,
                                   minor: *mut uint16_t,
                                   build: *mut uint16_t)
                                   -> c_int;
    fn scanifc_get_library_info(build_version: *mut *const c_char,
                                build_tag: *mut *const c_char)
                                -> c_int;
    fn scanifc_get_last_error(message_buffer: *mut c_char,
                              message_buffer_size: uint32_t,
                              message_size: *mut uint32_t)
                              -> c_int;
    fn scanifc_point3dstream_open(uri: *const c_char,
                                  sync_to_pps: int32_t,
                                  h3ds: *mut point3dstream_handle)
                                  -> c_int;
    // scanifc_point3dstream_open_with_logging
    // scanifc_point3dstream_open_rms
    fn scanifc_point3dstream_add_demultiplexer(h3ds: point3dstream_handle,
                                               filename: *const c_char,
                                               selections: *const c_char,
                                               classes: *const c_char)
                                               -> c_int;
    fn scanifc_point3dstream_set_rangegate(h3ds: point3dstream_handle,
                                           zone: uint16_t,
                                           near: c_float,
                                           far: c_float)
                                           -> c_int;
    fn scanifc_point3dstream_read(h3ds: point3dstream_handle,
                                  want: uint32_t,
                                  pxyz32: *mut xyz32,
                                  pattributes: *mut attributes,
                                  ptime: *mut uint64_t,
                                  got: *mut uint32_t,
                                  end_of_frame: *mut int32_t)
                                  -> c_int;
}
