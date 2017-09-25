use Result;
use scanifc_sys;

/// A stream of points, from an rxp file.
#[derive(Debug)]
pub struct Stream {
    handle: scanifc_sys::point3dstream_handle,
}

impl Stream {
    /// Opens a new stream for the provided uri.
    ///
    /// The second parameter is `sync_to_pps`. If true, only grabs points that were collected after
    /// the scanner was synced to an external PPS signal.
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
        Ok(Stream { handle: handle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_open() {
        Stream::open("file:data/scan.rxp", true).unwrap();
    }
}
