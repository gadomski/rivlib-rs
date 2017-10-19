use Inclination;
use std::os::raw::c_char;

pub enum Scanlib {}
pub enum Inclinations {}

extern "C" {
    pub fn scanlib_new(scanlib: *mut *mut Scanlib) -> i32;
    pub fn scanlib_last_error(scanlib: *const Scanlib) -> *const c_char;
    pub fn scanlib_drop(scanlib: *mut Scanlib);

    pub fn inclinations_from_path(
        scanlib: *mut Scanlib,
        path: *const c_char,
        sync_to_pps: bool,
        pointer: *mut *mut Inclinations,
    ) -> i32;
    pub fn inclinations_pointer(inclinations: *mut Inclinations, pointer: *mut *const Inclination);
    pub fn inclinations_len(inclinations: *mut Inclinations, len: *mut u64);
    pub fn inclinations_drop(inclinations: *mut Inclinations);
}
