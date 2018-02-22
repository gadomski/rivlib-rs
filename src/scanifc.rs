// TODO do I want to make this public?

use failure;
use {EchoType, Point};
use scanifc_sys;
use std::path::Path;
use std::ffi::CString;

const LAST_ERROR_BUFFER_LEN: usize = 512;

macro_rules! scanifc_try {
    ($expr:expr) => {{
        let result = $expr;
        if result != 0 {
            if let Some(last_error) = last_error() {
                return Err(Error{
                    code: result,
                    message: last_error
                }.into());
            } else {
                return Err(Error{
                    code: result,
                    message: String::new()
                }.into());
            }
        }
    }}
}

#[derive(Debug)]
pub struct Stream {
    handle: *mut scanifc_sys::point3dstream,
}

#[derive(Debug, Fail)]
#[fail(display = "scanifc error, code {}: {}", code, message)]
pub struct Error {
    code: i32,
    message: String,
}

impl Stream {
    pub fn open<P: AsRef<Path>>(path: P, sync_to_pps: bool) -> Result<Stream, failure::Error> {
        use std::ptr;

        let path = CString::new(path.as_ref().to_string_lossy().as_ref())?;
        let sync_to_pps = if sync_to_pps { 1 } else { 0 };
        let mut handle = ptr::null_mut();
        unsafe {
            scanifc_try!(scanifc_sys::scanifc_point3dstream_open(
                path.as_ptr(),
                sync_to_pps,
                &mut handle
            ));
        }
        Ok(Stream { handle: handle })
    }

    pub fn read(&mut self, want: u32) -> Result<Vec<Point>, Error> {
        let want_usize = want as usize;
        let mut points = vec![Default::default(); want_usize];
        let mut attributes = vec![Default::default(); want_usize];
        let mut times = vec![Default::default(); want_usize];
        let mut got = 0;
        // We ignore end of frame b/c it doesn't seem to be set.
        let mut _end_of_frame = 0;
        unsafe {
            scanifc_try!(scanifc_sys::scanifc_point3dstream_read(
                self.handle,
                want,
                points.as_mut_ptr(),
                attributes.as_mut_ptr(),
                times.as_mut_ptr(),
                &mut got,
                &mut _end_of_frame
            ));
        }
        // TODO test point mappings
        let points = points
            .into_iter()
            .zip(attributes)
            .zip(times)
            .take(got as usize)
            .map(|((point, attribute), time)| Point {
                x: point.x,
                y: point.y,
                z: point.z,
                amplitude: attribute.amplitude,
                reflectance: attribute.reflectance,
                deviation: attribute.deviation,
                echo_type: EchoType::from(attribute.flags),
                is_waveform_available: attribute.flags & 8 == 8,
                is_pseudo_echo: attribute.flags & 16 == 16,
                is_sw_target: attribute.flags & 32 == 32,
                with_fresh_pps: attribute.flags & 64 == 64,
                is_time_in_pps_timeframe: attribute.flags & 128 == 128,
                facet_number: ((attribute.flags >> 8) & 3) as u8,
                time: time as f64 * 1e9,
            })
            .collect::<Vec<_>>();
        Ok(points)
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            scanifc_sys::scanifc_point3dstream_close(self.handle);
        }
    }
}

fn last_error() -> Option<String> {
    let mut buffer = vec![0; LAST_ERROR_BUFFER_LEN];
    let mut message_size = 0;
    let result = unsafe {
        scanifc_sys::scanifc_get_last_error(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut message_size,
        )
    };
    if result == 0 {
        CString::new(
            buffer
                .iter()
                .take(message_size as usize)
                .filter_map(|&n| if n >= 0 { Some(n as u8) } else { None })
                .collect::<Vec<_>>(),
        ).ok()
            .and_then(|s| s.into_string().ok())
    } else {
        None
    }
}
