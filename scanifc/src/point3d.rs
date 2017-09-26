use {Point, Result};
use scanifc_sys;
use std::collections::VecDeque;
use std::path::Path;

// Cribbed from rivlib's examples.
const DEFAULT_WANT: u32 = 1024;
const DEFAULT_SYNC_TO_PPS: bool = true;

/// A builder for streams.
///
/// Allows configuration of the stream behavior.
#[derive(Debug)]
pub struct Builder {
    sync_to_pps: bool,
    uri: Uri,
}

/// A stream of points, from an rxp file.
#[derive(Debug)]
pub struct Stream {
    buffer: VecDeque<Point>,
    handle: scanifc_sys::point3dstream_handle,
    want: u32,
}

#[derive(Debug, PartialEq)]
enum Uri {
    File(String),
    Rdtp(String),
}

impl Builder {
    /// Creates a new builder for the provided file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Builder;
    /// let builder = Builder::from_path("data/scan.rxp");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Builder {
        Builder {
            sync_to_pps: DEFAULT_SYNC_TO_PPS,
            uri: Uri::from_path(path),
        }
    }

    /// Creates a new builder for the provided rdtp uri.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Builder;
    /// let builder = Builder::from_rdtp("192.168.0.33/current?type=mon");
    /// ```
    pub fn from_rdtp(rdtp: &str) -> Builder {
        Builder {
            sync_to_pps: DEFAULT_SYNC_TO_PPS,
            uri: Uri::from_rdtp(rdtp),
        }
    }

    /// Creates a stream from this builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Builder;
    /// let stream = Builder::from_path("data/scan.rxp").to_stream().unwrap();
    /// ```
    pub fn to_stream(&self) -> Result<Stream> {
        Stream::open(self.uri.as_str(), self.sync_to_pps)
    }
}

impl Stream {
    /// Opens a new stream for the provided uri, with no logger and default buffer size.
    ///
    /// The second parameter is `sync_to_pps`. If true, only grabs points that were collected after
    /// the scanner was synced to an external PPS signal.
    ///
    /// To customize logging and buffer size behavior, use a `Builder`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let stream = Stream::open("file:data/scan.rxp", true).unwrap();
    /// ```
    pub fn open(uri: &str, sync_to_pps: bool) -> Result<Stream> {
        use std::ffi::CString;
        use std::ptr;

        let mut handle: scanifc_sys::point3dstream_handle = ptr::null_mut();
        let uri = CString::new(uri)?;
        scanifc_try!(scanifc_sys::scanifc_point3dstream_open(
            uri.as_ptr(),
            if sync_to_pps { 1 } else { 0 },
            &mut handle,
        ));
        Ok(Stream {
            buffer: VecDeque::new(),
            handle: handle,
            want: DEFAULT_WANT,
        })
    }

    /// Consumes this stream and returns a vector of points.
    ///
    /// If this stream is mid-read, the returned points will be only the remaining points in the
    /// stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let stream = Stream::open("file:data/scan.rxp", true).unwrap();
    /// let points = stream.into_points().unwrap();
    /// ```
    pub fn into_points(self) -> Result<Vec<Point>> {
        self.collect()
    }

    fn fill_buffer(&mut self) -> Result<Option<()>> {
        if let Some(points) = self.read()? {
            self.buffer.extend(points.into_iter());
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    fn read(&mut self) -> Result<Option<Vec<Point>>> {
        let mut pxyz32 = vec![Default::default(); self.want as usize];
        let mut pattributes = vec![Default::default(); self.want as usize];
        let mut ptime = vec![Default::default(); self.want as usize];
        let mut got = 0;
        let mut end_of_frame = 0;

        scanifc_try!(scanifc_sys::scanifc_point3dstream_read(
            self.handle,
            self.want,
            pxyz32.as_mut_ptr(),
            pattributes.as_mut_ptr(),
            ptime.as_mut_ptr(),
            &mut got,
            &mut end_of_frame,
        ));
        Ok(if got == 0 && end_of_frame == 0 {
            None
        } else {
            Some(
                pxyz32
                    .into_iter()
                    .zip(pattributes.into_iter())
                    .zip(ptime.into_iter())
                    .take(got as usize)
                    .map(|((xyz32, attributes), time)| {
                        Point::from((xyz32, attributes, time))
                    })
                    .collect(),
            )
        })
    }
}

impl Iterator for Stream {
    type Item = Result<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.buffer.pop_front() {
            Some(Ok(point))
        } else {
            match self.fill_buffer() {
                Ok(Some(())) => self.next(),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            }
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { scanifc_sys::scanifc_point3dstream_close(self.handle) };
    }
}

impl Uri {
    fn from_path<P: AsRef<Path>>(path: P) -> Uri {
        Uri::File(format!("file:{}", path.as_ref().display()))
    }

    fn from_rdtp(rdtp: &str) -> Uri {
        Uri::Rdtp(format!("rdtp://{}", rdtp))
    }

    fn as_str(&self) -> &str {
        match *self {
            Uri::File(ref s) | Uri::Rdtp(ref s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_from_path() {
        let uri = Uri::from_path("a/path/to/a/file.rxp");
        assert_eq!(Uri::File("file:a/path/to/a/file.rxp".to_string()), uri);
    }

    #[test]
    fn uri_from_rdtp() {
        let uri = Uri::from_rdtp("192.168.0.33/current?type=mon");
        assert_eq!(
            Uri::Rdtp("rdtp://192.168.0.33/current?type=mon".to_string()),
            uri
        );
    }

    #[test]
    fn builder() {
        let stream = Builder::from_path("data/scan.rxp").to_stream().unwrap();
        assert_eq!(24390, stream.into_points().unwrap().len());
    }

    #[test]
    fn stream_read() {
        let stream = Stream::open("file:data/scan.rxp", true).unwrap();
        let points = stream.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(24390, points.len());
    }
}
