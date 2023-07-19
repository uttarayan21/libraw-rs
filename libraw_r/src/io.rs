// ```c++
// class DllDef LibRaw_abstract_datastream
// {
// public:
//   LibRaw_abstract_datastream() { };
//   virtual ~LibRaw_abstract_datastream(void) { }
//   virtual int valid() = 0;
//   virtual int read(void *, size_t, size_t) = 0;
//   virtual int seek(INT64, int) = 0;
//   virtual INT64 tell() = 0;
//   virtual INT64 size() = 0;
//   virtual int get_char() = 0;
//   virtual char *gets(char *, int) = 0;
//   virtual int scanf_one(const char *, void *) = 0;
//   virtual int eof() = 0;
// #ifdef LIBRAW_OLD_VIDEO_SUPPORT
//   virtual void *make_jas_stream() = 0;
// #endif
//   virtual int jpeg_src(void *);
//   virtual void buffering_off() {}
//   /* reimplement in subclass to use parallel access in xtrans_load_raw() if
//    * OpenMP is not used */
//   virtual int lock() { return 1; } /* success */
//   virtual void unlock() {}
//   virtual const char *fname() { return NULL; };
// #ifdef LIBRAW_WIN32_UNICODEPATHS
//   virtual const wchar_t *wfname() { return NULL; };
// #endif
// };
// ```
pub trait LibrawDatastream: Read + Seek + Sized {
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    unsafe fn read(this: *mut Self, buffer: *const libc::c_void, sz: usize, nmemb: usize) -> i32 {
        assert!(!this.is_null());
        let this = &mut *this;
        let to_read = sz * nmemb;
        if to_read < 1 {
            return 0;
        }
        if this
            .read_exact(core::slice::from_raw_parts_mut(
                buffer.cast::<u8>().cast_mut(),
                to_read,
            ))
            .is_err()
        {
            -1i32
        } else {
            to_read as i32
        }
    }
    /// # Safety
    ///
    /// Sus
    unsafe fn seek(this: *mut Self, offset: i64, whence: u32) -> i32 {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        match whence {
            sys::SEEK_SET => this.seek(std::io::SeekFrom::Start(offset as u64)).ok(),
            sys::SEEK_CUR => this.seek(std::io::SeekFrom::Current(offset)).ok(),
            sys::SEEK_END => this.seek(std::io::SeekFrom::End(offset)).ok(),
            _ => return 0,
        };
        0
    }
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    unsafe fn tell(this: *mut Self) -> i64 {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        this.stream_position().map(|f| f as i64).unwrap_or(-1)
    }
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    unsafe fn eof(this: *mut Self) -> i32 {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        <Self as Eof>::eof(this).map(|f| f as i32).unwrap_or(0) - 1
    }

    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    unsafe fn size(this: *mut Self) -> i64 {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        <Self as Eof>::len(this).map(|f| f as i64).unwrap_or(-1)
    }

    /// # Safety
    ///
    /// Reads a char from the buffer and casts it as i32 and in case of error returns -1
    unsafe fn get_char(this: *mut Self) -> libc::c_int {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        let mut buf = [0u8];
        if this.read_exact(&mut buf).is_err() {
            -1
        } else {
            buf[0] as libc::c_int
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer and is called from ffi.
    /// This function is like fgets(3)
    /// The C++ function which wraps this
    /// `char *LibRaw_buffer_datastream::gets(char *s, int sz)`
    unsafe fn gets(
        this: *mut Self,
        buffer: *mut libc::c_char,
        size: libc::c_int,
    ) -> *const libc::c_char {
        assert!(!this.is_null());
        let this = unsafe { &mut *this };
        let mut buf = vec![0u8; size as usize];
        if this.read_exact(&mut buf).is_err() {
            return std::ptr::null();
        }
        todo!()
    }
}

use core::ops::{Deref, DerefMut};
// pub trait Libraw
use std::io::{Read, Seek};

pub trait Eof: Seek {
    fn len(&mut self) -> ::std::io::Result<u64> {
        use ::std::io::SeekFrom;
        let old_pos = ::std::io::Seek::seek(self, SeekFrom::Current(0))?;
        let len = ::std::io::Seek::seek(self, SeekFrom::End(0))?;
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }
        Ok(len)
    }

    fn is_empty(&mut self) -> ::std::io::Result<bool> {
        Ok(self.len()? == 0)
    }

    fn eof(&mut self) -> ::std::io::Result<bool> {
        Ok(self.len()? == ::std::io::Seek::seek(self, ::std::io::SeekFrom::Current(0))?)
    }
}

impl<T: Seek> Eof for T {}

/// Abstract Datastream
///
/// Using the rust version of the abstract datastream
pub struct AbstractDatastream<T: Read + Seek + Sized> {
    inner: T,
}

impl<T: Read + Seek + Sized> AbstractDatastream<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Read + Seek + Sized> DerefMut for AbstractDatastream<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Read + Seek + Sized> Read for AbstractDatastream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Seek + Sized> Seek for AbstractDatastream<T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek + Sized> Deref for AbstractDatastream<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
