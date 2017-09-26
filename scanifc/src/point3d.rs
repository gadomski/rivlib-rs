use {Point, Result};
use scanifc_sys;
use std::collections::VecDeque;
use std::path::Path;

// Cribbed from rivlib's examples.
const DEFAULT_WANT: u32 = 1024;
const DEFAULT_SYNC_TO_PPS: bool = true;

/// A stream of 3D points.
///
/// Follows the builder pattern to set the options for the stream.
#[derive(Debug)]
pub struct Stream {
    sync_to_pps: bool,
    uri: Uri,
    want: u32,
}

/// An open stream of points, used for reading.
#[derive(Debug)]
pub struct OpenStream {
    buffer: VecDeque<Point>,
    handle: scanifc_sys::point3dstream_handle,
    want: u32,
}

#[derive(Debug, PartialEq)]
enum Uri {
    File(String),
    Rdtp(String),
}

impl Stream {
    /// Creates a new builder for the provided file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let builder = Stream::from_path("data/scan.rxp");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Stream {
        Stream {
            sync_to_pps: DEFAULT_SYNC_TO_PPS,
            uri: Uri::from_path(path),
            want: DEFAULT_WANT,
        }
    }

    /// Creates a new builder for the provided rdtp uri.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let builder = Stream::from_rdtp("192.168.0.33/current?type=mon");
    /// ```
    pub fn from_rdtp(rdtp: &str) -> Stream {
        Stream {
            sync_to_pps: DEFAULT_SYNC_TO_PPS,
            uri: Uri::from_rdtp(rdtp),
            want: DEFAULT_WANT,
        }
    }

    /// Sets the `sync_to_pps` field.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let builder = Stream::from_path("data/scan.rxp").sync_to_pps(false);
    /// ```
    pub fn sync_to_pps(mut self, sync_to_pps: bool) -> Stream {
        self.sync_to_pps = sync_to_pps;
        self
    }

    /// Sets the number of points requested on each read.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let builder = Stream::from_path("data/scan.rxp").want(1);
    /// ```
    pub fn want(mut self, want: u32) -> Stream {
        self.want = want;
        self
    }

    /// Creates a stream from this builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let stream = Stream::from_path("data/scan.rxp").open().unwrap();
    /// ```
    pub fn open(&self) -> Result<OpenStream> {
        use std::ffi::CString;
        use std::ptr;

        let mut handle: scanifc_sys::point3dstream_handle = ptr::null_mut();
        let uri = CString::new(self.uri.as_str())?;
        scanifc_try!(scanifc_sys::scanifc_point3dstream_open(
            uri.as_ptr(),
            if self.sync_to_pps { 1 } else { 0 },
            &mut handle,
        ));
        Ok(OpenStream {
            buffer: VecDeque::new(),
            handle: handle,
            want: self.want,
        })
    }
}

impl OpenStream {
    /// Consumes this stream and returns a vector of points.
    ///
    /// If this stream is mid-read, the returned points will be only the remaining points in the
    /// stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use scanifc::point3d::Stream;
    /// let stream = Stream::from_path("data/scan.rxp").open().unwrap();
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

impl Iterator for OpenStream {
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

impl Drop for OpenStream {
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
        let stream = Stream::from_path("data/scan.rxp").open().unwrap();
        assert_eq!(24390, stream.into_points().unwrap().len());
    }

    #[test]
    fn builder_sync_to_pps() {
        let stream = Stream::from_path("data/scan.rxp")
            .sync_to_pps(false)
            .open()
            .unwrap();
        assert_eq!(24390, stream.into_points().unwrap().len());
    }

    #[test]
    fn builder_want() {
        let mut stream = Stream::from_path("data/scan.rxp").want(1).open().unwrap();
        assert_eq!(1, stream.read().unwrap().unwrap().len());
    }
}
