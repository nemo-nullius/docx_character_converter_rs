extern crate libc;

use libc::c_char;
use libc::c_int;
use libc::c_void;
use libc::size_t;
use std::ffi::CStr;
use std::ffi::CString;
use std::str;

#[link(name = "clib/opencc")]
extern "C" {
    fn opencc_open(configFileName: *const c_char) -> *mut c_void;
    fn opencc_convert_utf8(
        opencc: *mut c_void,
        input: *const c_char,
        length: size_t,
    ) -> *mut c_char;
    fn opencc_convert_utf8_free(str_converted: *mut c_char);
    fn opencc_error() -> *mut c_char;
    fn opencc_close(opencc: *mut c_void) -> c_int;
}

pub fn convert(config_filepath: &str, text: &str) -> String {
    //println!("1");
    let conf_filepath = CString::new(config_filepath).unwrap();
    let conf_filepath_ccharp: *const c_char = conf_filepath.as_ptr();
    let od = unsafe { opencc_open(conf_filepath_ccharp) };
    // ERROR CHECK
    ////println!("od: {}", od as i8);
    let r_err_raw: *const c_char = unsafe { opencc_error() };
    let r_err: &CStr = unsafe { CStr::from_ptr(r_err_raw) };
    let r_err_slice: &str = r_err.to_str().unwrap();
    //let r_err_string: String = r_err_slice.to_owned();
    ////println!("r_err: {}", r_err_slice);
    if od as i8 == -1 {
        panic!("Config file cannot be found!");
    }
    //let od = unsafe { opencc_open(conf_filepath_ccharp) };
    //println!("2");
    let input = CString::new(text).unwrap();
    //println!("{}", c);
    let r_raw = unsafe { opencc_convert_utf8(od, input.as_ptr(), text.len() as size_t) };
    //println!("3");
    let r_str: &CStr = unsafe { CStr::from_ptr(r_raw) };
    let r_slice: &str = r_str.to_str().unwrap();
    let result = r_slice.to_owned();
    // SOME CLEAN WORK
    unsafe {
        opencc_convert_utf8_free(r_raw);
        let exit_status = opencc_close(od);
        ////println!("Finish status: {}.", exit_status);
    }
    result
}
