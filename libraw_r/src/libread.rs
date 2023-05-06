extern "C" {
    pub fn libraw_read_file_datastream(
        data: *mut libc::c_void,
        ptr: *mut libc::c_void,
        size: usize,
        nmemb: usize,
    ) -> libc::c_int;

    #[deprecated]
    pub fn libraw_read_bigfile_datastream(
        data: *mut libc::c_void,
        ptr: *mut libc::c_void,
        size: usize,
        nmemb: usize,
    ) -> libc::c_int;

    pub fn libraw_read_buffer_datastream(
        data: *mut libc::c_void,
        ptr: *mut libc::c_void,
        size: usize,
        nmemb: usize,
    ) -> libc::c_int;
}
