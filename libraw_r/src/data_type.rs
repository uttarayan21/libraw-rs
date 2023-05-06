use crate::{EmptyProcessor, LibrawError, Processor};
use std::path::Path;

// HACK: Trick to bypass the not impl From<Infallible> for LibrawError
struct Bypass<T>(pub T);
impl TryInto<EmptyProcessor> for Bypass<EmptyProcessor> {
    type Error = LibrawError;
    fn try_into(self) -> Result<EmptyProcessor, Self::Error> {
        Ok(self.0)
    }
}

impl EmptyProcessor {
    pub fn open<'p, D: DataType>(
        self,
        data: impl Into<D>,
    ) -> Result<Processor<'p, D>, LibrawError> {
        unsafe { data.into().open(Bypass(self)) }
    }
}

impl<T> Processor<'_, T> {
    pub fn open<'p, D: DataType>(
        self,
        data: impl Into<D>,
    ) -> Result<Processor<'p, D>, LibrawError> {
        let ep: EmptyProcessor = self.recycle()?;
        ep.open(data)
    }
}

#[derive(Clone, Copy)]
pub struct File<'p> {
    path: &'p Path,
}

impl<'p, T: ?Sized + AsRef<Path> + 'p> From<&'p T> for File<'p> {
    fn from(path: &'p T) -> File<'p> {
        let path = path.as_ref();
        File { path }
    }
}

#[derive(Clone, Copy)]
pub struct Buffer<'b> {
    buffer: &'b [u8],
}

impl<'b, T: ?Sized + AsRef<[u8]> + 'b> From<&'b T> for Buffer<'b> {
    fn from(buffer: &'b T) -> Buffer<'b> {
        let buffer = buffer.as_ref();
        Buffer { buffer }
    }
}

pub trait DataType {
    /// # Safety
    ///
    /// Calls unsafe C++ ffi functions
    unsafe fn open<'p, P: TryInto<EmptyProcessor>>(
        self,
        p: P,
    ) -> Result<Processor<'p, Self>, LibrawError>
    where
        LibrawError: From<<P as TryInto<EmptyProcessor>>::Error>;

    #[cfg(feature = "exif")]
    /// # Safety
    ///
    /// Calls unsafe C++ ffi functions
    // unsafe fn read(&mut self, );
    fn read(
    ) -> unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void, usize, usize) -> libc::c_int
    {
        todo!()
    }
}

impl DataType for File<'_> {
    unsafe fn open<'p, P: TryInto<EmptyProcessor>>(
        self,
        p: P,
    ) -> Result<Processor<'p, Self>, LibrawError>
    where
        LibrawError: From<<P as TryInto<EmptyProcessor>>::Error>,
    {
        let mut ep = p.try_into()?;
        let path = dunce::canonicalize(self.path)?;
        #[cfg(windows)]
        {
            if let Ok(path) = crate::path_to_widestring(self.path) {
                if LibrawError::check(unsafe {
                    sys::libraw_open_wfile(ep.inner_mut().as_ptr(), path.as_ptr())
                })
                .is_ok()
                {
                    return Ok(Processor {
                        inner: ep,
                        _data: core::marker::PhantomData,
                    });
                }
            }
        }

        LibrawError::check(sys::libraw_open_file(
            ep.inner_mut().as_ptr(),
            crate::path_to_cstr(path)?.as_ptr(),
        ))?;
        Ok(Processor {
            inner: ep,
            _data: core::marker::PhantomData,
        })
    }

    fn read(
    ) -> unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void, usize, usize) -> libc::c_int
    {
        todo!()
    }
}

impl<'b> DataType for Buffer<'b> {
    unsafe fn open<'p, P: TryInto<EmptyProcessor>>(
        self,
        p: P,
    ) -> Result<Processor<'p, Self>, LibrawError>
    where
        LibrawError: From<<P as TryInto<EmptyProcessor>>::Error>,
    {
        let mut ep = p.try_into()?;
        LibrawError::check(sys::libraw_open_buffer(
            ep.inner_mut().as_ptr(),
            self.buffer.as_ptr().cast(),
            self.buffer.len(),
        ))?;
        Ok(Processor {
            inner: ep,
            _data: core::marker::PhantomData,
        })
    }
}
