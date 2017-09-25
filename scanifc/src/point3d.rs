use {Result, Point};
use std::collections::VecDeque;
use scanifc_sys;

// Cribbed from rivlib's examples.
const DEFAULT_WANT: u32 = 1024;

/// A stream of points, from an rxp file.
#[derive(Debug)]
pub struct Stream {
    buffer: VecDeque<Point>,
    handle: scanifc_sys::point3dstream_handle,
    want: u32,
}

/// A builder for streams.
///
/// Allows configuration of the stream behavior.
#[derive(Debug)]
pub struct Builder {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_read() {
        let stream = Stream::open("file:data/scan.rxp", true).unwrap();
        let points = stream.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(24390, points.len());
    }
}
