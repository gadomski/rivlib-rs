//! `rxp` data streams.

use std::ffi::CString;
use std::ptr;

use libc::{int32_t, uint64_t};

use Result;
use error::Error;
use scanifc::{attributes, last_error, point3dstream_handle, scanifc_point3dstream_open,
              scanifc_point3dstream_add_demultiplexer, scanifc_point3dstream_set_rangegate,
              scanifc_point3dstream_read, xyz32};

const DEFAULT_POINT_ITERATOR_WANT: u32 = 1000;

/// A 3D pointcloud data stream.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct Stream {
    handle: point3dstream_handle,
    eof: bool,
    sync_to_pps: bool,
}

impl Stream {
    /// Opens a data stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::stream::Stream;
    /// let stream = Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// ```
    pub fn open<T: Into<Vec<u8>>>(uri: T, sync_to_pps: bool) -> Result<Stream> {
        let uri = try!(CString::new(uri)).into_raw();
        let mut h3ds: point3dstream_handle = ptr::null_mut();
        unsafe {
            let retval = scanifc_point3dstream_open(uri, sync_to_pps as i32, &mut h3ds);
            let _ = CString::from_raw(uri);
            if retval != 0 {
                return Err(Error::Scanifc(retval, last_error()));
            }
        }

        Ok(Stream {
            handle: h3ds,
            eof: false,
            sync_to_pps: sync_to_pps,
        })
    }

    /// Adds a demultiplexer to the data stream.
    ///
    /// The demultiplexer will write selected packages from the rxpstream to a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::Stream;
    /// let mut stream = Stream::open("data/130501_232206_cut.rxp", true).unwrap();
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
                return Err(Error::Scanifc(retval, last_error()));
            }
        }
        Ok(())
    }

    /// Override the default rangegate of the sensor.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::stream::Stream;
    /// let mut stream = Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// stream.set_rangegate(1, 0.0, 1.0);
    /// ```
    pub fn set_rangegate(&mut self, zone: u16, near: f32, far: f32) -> Result<()> {
        scanifc_try!(scanifc_point3dstream_set_rangegate(self.handle, zone, near, far));
        Ok(())
    }

    /// Reads some points from this stream.
    ///
    /// Returns an optional vector of points, or `None` if the end of the stream has been reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::stream::Stream;
    /// let mut stream = Stream::open("data/130501_232206_cut.rxp", true).unwrap();
    /// let points = stream.read(10).unwrap();
    /// ```
    pub fn read(&mut self, want: u32) -> Result<Option<Vec<Point>>> {
        if self.eof {
            return Ok(None);
        }
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
        if got == 0 && end_of_frame == 0 {
            self.eof = true;
            return Ok(None);
        }

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
        Ok(Some(points))
    }

    /// Returns true if this stream is synced to pps.
    ///
    /// # Examples
    ///
    /// ```
    /// use rivlib::stream::Stream;
    /// let stream = Stream::open("data/130501_232206_cut.rxp", false).unwrap();
    /// assert!(!stream.sync_to_pps());
    pub fn sync_to_pps(&self) -> bool {
        self.sync_to_pps
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
            match self.stream.read(self.want).unwrap() {
                Some(points) => self.points = points,
                None => return None,
            }
        }
        self.points.pop()
    }
}

/// A 3D point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// The x value of the point.
    pub x: f32,
    /// The y value of the point.
    pub y: f32,
    /// The z value of the point.
    pub z: f32,
    /// Relative amplitude in [dB].
    pub amplitude: f32,
    /// Relative reflectance in [dB].
    pub reflectance: f32,
    /// A measure of pulse shape distortion.
    pub deviation: u16,
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
    pub time: u64,
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
    /// use rivlib::stream::EchoType;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_to_eof() {
        let mut stream = Stream::open("data/130501_232206_cut.rxp", true).unwrap();
        let points = stream.read(177208).unwrap().unwrap();
        assert_eq!(177208, points.len());
        let points = stream.read(1).unwrap();
        assert!(points.is_none());
    }
}
