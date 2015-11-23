//! Wrapper around the scanifc library.
//!
//! `scanifc` is used to retrieve point data from a stream.

use std::ffi::CString;

use libc::{c_char, c_int, c_float, int32_t, uint16_t, uint32_t, uint64_t};

const MESSAGE_BUFFER_SIZE: uint32_t = 256;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct xyz32 {
    pub x: c_float,
    pub y: c_float,
    pub z: c_float,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct attributes {
    pub amplitude: c_float,
    pub reflectance: c_float,
    pub deviation: uint16_t,
    pub flags: uint16_t,
    pub background_radiatino: c_float,
}

#[allow(non_camel_case_types)]
enum point3dstream {}
#[allow(non_camel_case_types)]
pub type point3dstream_handle = *mut point3dstream;

pub fn last_error() -> String {
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

#[link(name = "scanifc-mt")]
extern {
    pub fn scanifc_get_library_version(major: *mut uint16_t,
                                       minor: *mut uint16_t,
                                       build: *mut uint16_t)
                                       -> c_int;
    pub fn scanifc_get_library_info(build_version: *mut *const c_char,
                                    build_tag: *mut *const c_char)
                                    -> c_int;
    fn scanifc_get_last_error(message_buffer: *mut c_char,
                              message_buffer_size: uint32_t,
                              message_size: *mut uint32_t)
                              -> c_int;
    pub fn scanifc_point3dstream_open(uri: *const c_char,
                                      sync_to_pps: int32_t,
                                      h3ds: *mut point3dstream_handle)
                                      -> c_int;
    // scanifc_point3dstream_open_with_logging
    // scanifc_point3dstream_open_rms
    pub fn scanifc_point3dstream_add_demultiplexer(h3ds: point3dstream_handle,
                                                   filename: *const c_char,
                                                   selections: *const c_char,
                                                   classes: *const c_char)
                                                   -> c_int;
    pub fn scanifc_point3dstream_set_rangegate(h3ds: point3dstream_handle,
                                               zone: uint16_t,
                                               near: c_float,
                                               far: c_float)
                                               -> c_int;
    pub fn scanifc_point3dstream_read(h3ds: point3dstream_handle,
                                      want: uint32_t,
                                      pxyz32: *mut xyz32,
                                      pattributes: *mut attributes,
                                      ptime: *mut uint64_t,
                                      got: *mut uint32_t,
                                      end_of_frame: *mut int32_t)
                                      -> c_int;
}
