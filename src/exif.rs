use libraw_sys::*;

use crate::traits::LRString;

extern "C" {
    pub fn libraw_read_datastream(
        data: *mut libc::c_void,
        ptr: *mut libc::c_void,
        size: usize,
        nmemb: usize,
    ) -> libc::c_int;
}

pub extern "C" fn exif_parser_callback(
    context: *mut libc::c_void,
    tag: libc::c_int,
    _type: libc::c_int,
    len: libc::c_int,
    _ord: libc::c_uint,
    ifp: *mut libc::c_void,
    _base: INT64,
) {
    if !(tag == 0x9003 || tag == 0x9004) {
        return;
    }
    let mut buffer = vec![0_i8; len as usize];

    let res = unsafe {
        libraw_read_datastream(
            ifp as *mut libc::c_void,
            buffer.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
            buffer.len(),
            1,
        )
    };
    if res != 1 {
        return;
    }

    let context: &mut std::collections::HashMap<libc::c_int, String> =
        unsafe { std::mem::transmute(context) };
    context.insert(tag, buffer.as_slice().as_ascii().into());
}
