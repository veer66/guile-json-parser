extern crate guile_3_sys;
#[macro_use]
extern crate lazy_static;

use guile_3_sys::{
    scm_t_cell,
    scm_gc_malloc,
    scm_from_double,
    scm_c_define_gsubr, scm_c_define_module, scm_c_export, scm_from_int64, scm_from_uint64,
    scm_from_utf8_stringn, scm_to_utf8_stringn, SCM,
};
use serde_json;
use serde_json::{Number, Value};
use std::ffi::CString;
use serde_json::map::Map;

type N = usize;

const SCM_TC3_IMM24: N = 4;
const SCM_TC8_FLAG: N = SCM_TC3_IMM24 + 0x00;

const fn scm_make_itag8_bits(x: N, tag: N) -> N {
    (x << 8) + tag
}

const fn scm_makiflag_bits(n: N) -> N {
    scm_make_itag8_bits(n, SCM_TC8_FLAG)
}

const SCM_EOL_BITS: N = scm_makiflag_bits(3);
const SCM_BOOL_T_BITS: N = scm_makiflag_bits(4);
const SCM_BOOL_F_BITS: N = scm_makiflag_bits(0);

type ScmTBits = usize;

macro_rules! scm_pack {
    ($e:expr) => {
	($e as ScmTBits) as SCM
    }
}

macro_rules! scm_unpack {
    ($e:expr) => {
	$e as ScmTBits
    }
}

macro_rules! scm_pack_pointer {
    ($e:expr) => {
	scm_pack!($e as ScmTBits)
    }
}

macro_rules! scm_unpack_pointer {
    ($e:expr) => {
	scm_unpack!($e) as *mut ScmTBits
    }
}

macro_rules! scm2ptr {
    ($e:expr) => {
	scm_unpack_pointer!($e) as *mut scm_t_cell
    }
}

// macro_rules! scm_gc_cell_object {
//     ($x:expr, $n:expr) => {
// 	scm2ptr!($x).offset($n) as *mut SCM
//     }
// }

macro_rules! scm_gc_set_cell_object {
    ($x:expr, $n:expr, $v:expr) => {	
	*(scm2ptr!($x) as *mut SCM).offset($n) = $v
    }	
}

macro_rules! scm_gc_set_cell_word {
    ($x:expr, $n:expr, $v:expr) => {
	scm_gc_set_cell_object!($x, $n, scm_pack!($v))
    }
}

fn scm_cell (car: ScmTBits, cdr: ScmTBits) -> SCM {
    unsafe {
	let cell: SCM = scm_pack_pointer!(scm_gc_malloc(std::mem::size_of::<scm_t_cell>() as u64, 0 as *const i8));
	scm_gc_set_cell_word!(cell, 1, cdr);
	scm_gc_set_cell_word!(cell, 0, car);
	cell
    }
}

fn scm_cons (x: SCM, y: SCM) -> SCM {
    scm_cell(scm_unpack!(x), scm_unpack!(y))
}

const SCM_EOL: SCM = scm_pack!(SCM_EOL_BITS);
const SCM_BOOL_T: SCM = scm_pack!(SCM_BOOL_T_BITS);
const SCM_BOOL_F: SCM = scm_pack!(SCM_BOOL_F_BITS);

lazy_static! {
    static ref MOD_NAME: CString = CString::new("json parser").unwrap();
    static ref READ_STRING: CString = CString::new("read-string").unwrap();
}

fn convert_null() -> SCM {
    SCM_EOL
}

fn convert_bool(b: bool) -> SCM {
    if b {
        SCM_BOOL_T
    } else {
        SCM_BOOL_F
    }
}

fn convert_number(n: &Number) -> SCM {
    if let Some(i) = n.as_i64() {
        unsafe { scm_from_int64(i) }
    } else if let Some(u) = n.as_u64() {
        unsafe { scm_from_uint64(u) }
    } else if let Some(f) = n.as_f64() {
	unsafe { scm_from_double(f) }
    } else {
	unsafe { scm_from_utf8_stringn("FAIL".as_ptr() as *const i8, 4) }
    }
}

fn convert_string(s: &str) -> SCM {
    unsafe {
	let s_u8 = s.as_bytes();
	scm_from_utf8_stringn(s_u8.as_ptr() as *const i8, s_u8.len() as u64)
    }
}

fn convert_array(a: &[Value]) -> SCM {
    let mut l: SCM = SCM_EOL;
    for i in a.iter().rev() {
	let v = convert(i);
	l = scm_cons(v, l);
    }
    l
}

fn convert_object(m: &Map<String, Value>) -> SCM {
    let mut alist: SCM = SCM_EOL;
    for (k, v) in m.iter() {
	let k_ = convert_string(k);
	let v_ = convert(&v);
	let elem = scm_cons(k_, v_);
	alist = scm_cons(elem, alist);
    }
    alist
}

fn convert(v: &Value) -> SCM {
    match v {
        Value::Null => convert_null(),
        Value::Bool(b) => convert_bool(*b),
        Value::Number(n) => convert_number(n),
        Value::String(s) => convert_string(&s),
        Value::Array(a) => convert_array(&a),
        Value::Object(m) => convert_object(m),
    }
}

#[no_mangle]
pub extern "C" fn read_string(raw_str: SCM) -> SCM {
    unsafe {
        let mut len = 0u64;
        let len_ptr: *mut u64 = &mut len;
        let utf8_str = scm_to_utf8_stringn(raw_str, len_ptr) as *const u8;
        let s = String::from_utf8_lossy(std::slice::from_raw_parts(utf8_str, len as usize));
        match serde_json::from_str(&s) {
            Ok(v) => convert(&v),
            Err(e) => {
                eprintln!("ERROR {:?}\n", e);
                scm_from_utf8_stringn("FAIL".as_ptr() as *const i8, 4)
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn init_json_pasrer(_: *mut std::ffi::c_void) {
    unsafe {
        scm_c_define_gsubr(
            READ_STRING.as_ptr() as *const i8,
            1,
            0,
            0,
            read_string as *mut std::ffi::c_void,
        );
        scm_c_export(
            READ_STRING.as_ptr() as *const i8,
            0 as *mut std::ffi::c_void,
        );
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

#[cfg(test)]
mod tests {}
