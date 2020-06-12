extern crate guile_3_sys;

use std::ffi::CString;
#[macro_use]
extern crate lazy_static;
use guile_3_sys::{scm_c_define_gsubr, scm_c_define_module, scm_c_export, scm_from_double, SCM};

lazy_static! {
    static ref MOD_NAME: CString = CString::new("json parser").unwrap();
    static ref V1: CString = CString::new("v1").unwrap();
}

#[no_mangle]
pub extern "C" fn v1_wrapper(_x: SCM) -> SCM {
    unsafe { scm_from_double(10.3) }
}

#[no_mangle]
pub extern "C" fn init_json_pasrer(_: *mut std::ffi::c_void) {
    unsafe {
        scm_c_define_gsubr(
            V1.as_ptr() as *const i8,
            1,
            0,
            0,
            v1_wrapper as *mut std::ffi::c_void,
        );
        scm_c_export(V1.as_ptr() as *const i8,
		     0 as *mut std::ffi::c_void);
    }
}

#[no_mangle]
pub extern "C" fn scm_init_json_parser_module() {
    unsafe {
        scm_c_define_module(
            MOD_NAME.as_ptr() as *const i8,
	    Some(init_json_pasrer),
            0 as *mut std::ffi::c_void,
        );
    }
}
