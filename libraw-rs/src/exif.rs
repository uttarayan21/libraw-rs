use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use alloc::sync::Arc;
use libraw_sys::*;

use crate::{LibrawError, Processor};
pub type Callback<T> =
    Box<dyn Fn(ExifCallbackArgs<T>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>>;

#[derive(Debug)]
pub struct ExifCallbackArgs<'a, T> {
    pub callback_data: &'a mut T,
    pub tag: i32,
    pub data_type: DataType,
    pub len: i32,
    pub ord: u32,
    pub data: &'a mut [u8],
    pub base: i64,
}

extern "C" {
    pub fn libraw_read_file_datastream(
        data: *mut libc::c_void,
        ptr: *mut libc::c_void,
        size: usize,
        nmemb: usize,
    ) -> libc::c_int;

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

#[derive(Debug, Clone, Copy)]
pub enum DataStreamType {
    File,
    BigFile,
    Buffer,
}

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Byte = 1,
    Ascii = 2,
    Short = 3,
    Long = 4,
    Rational = 5,
    SByte = 6,
    Undefined = 7,
    SShort = 8,
    SLong = 9,
    SRational = 10,
    Float = 11,
    Double = 12,
}

impl From<libc::c_int> for DataType {
    fn from(value: libc::c_int) -> Self {
        match value {
            1 => DataType::Byte,
            2 => DataType::Ascii,
            3 => DataType::Short,
            4 => DataType::Long,
            5 => DataType::Rational,
            6 => DataType::SByte,
            7 => DataType::Undefined,
            8 => DataType::SShort,
            9 => DataType::SLong,
            10 => DataType::SRational,
            11 => DataType::Float,
            12 => DataType::Double,
            _ => DataType::Undefined,
        }
    }
}

impl From<DataType> for libc::c_int {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Byte => 1,
            DataType::Ascii => 2,
            DataType::Short => 3,
            DataType::Long => 4,
            DataType::Rational => 5,
            DataType::SByte => 6,
            DataType::Undefined => 7,
            DataType::SShort => 8,
            DataType::SLong => 9,
            DataType::SRational => 10,
            DataType::Float => 11,
            DataType::Double => 12,
        }
    }
}

impl DataStreamType {
    fn read(
        &self,
    ) -> unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void, usize, usize) -> libc::c_int
    {
        match self {
            DataStreamType::File => libraw_read_file_datastream,
            DataStreamType::BigFile => libraw_read_bigfile_datastream,
            DataStreamType::Buffer => libraw_read_buffer_datastream,
        }
    }
}

impl Processor {
    /// Sets the data and the callback to parse the exif data.  
    /// The callback is called with the exif data as a byte slice.  
    /// It should return `Ok(())` if the exif data was parsed successfully.  
    /// It should return `Err(LibrawError)` if the exif data could not be parsed.  
    ///
    /// Args:-
    ///    - data: The data we pass to the function to act as a temp storage
    ///    - data_steam_type: What type of data stream we are using (file, bigfile, buffer)  
    ///    - callback: The callback function that will be called with the exif data as a byte slice.  
    ///      Args:-  
    ///
    ///      | Field | Type   | Description |
    ///      |---|---|---|
    ///      | data | &mut T | The data we pass to the function to act as a temp storage |
    ///      | tag  |  i32   | The tag of the exif data |
    ///      | type | i32    | The type of the exif data |
    ///      | len  |  i32   | The length of the exif data |
    ///      | ord  |  u32   | The order of the exif data |
    ///      | data | &[u8]  | The exif data as a byte slice |
    ///      | base | i64    | Not sure  |
    ///
    /// NOTE:-
    ///
    /// Currently this uses `Rc<RefCell<T>>` for the data  
    /// and Rc<Box<T: Fn>> for the callback function  
    /// So if libraw internally uses multithreading for a single image then this might cause UB  
    /// Check <https://www.libraw.org/docs/API-CXX.html#callbacks>  
    ///
    /// Saftey.
    /// BUG:
    /// There's a bug which doens't unset the callback from the libraw when you the data is
    /// dropped.
    /// Since the callback is stored in memory allocated by rust, it will be dropped when the
    /// returned data is dropped. But libraw doesn't know that and will try to access the data.
    /// Possibly causing a SEGFAULT.
    pub fn set_exif_callback<T, F>(
        &mut self,
        data: T,
        data_stream_type: DataStreamType,
        callback: F,
    ) -> Result<ExifReader<T>, crate::error::LibrawError>
    where
        F: Fn(ExifCallbackArgs<T>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + 'static,
    {
        let eread = ExifRead {
            callback: Box::new(callback),
            data: Mutex::new(data),
            errors: Mutex::new(Default::default()),
            data_stream_type,
        };
        let eread = Arc::new(eread);

        unsafe {
            libraw_set_exifparser_handler(
                self.inner.as_ptr(),
                Some(ExifReader::<T>::exif_parser_callback),
                Arc::<ExifRead<T>>::into_raw(Arc::clone(&eread)) as *mut libc::c_void,
            );
        };

        Ok(ExifReader {
            inner: eread,
            libraw_data_t: self.inner,
            libraw_data_t_dropped: self.dropped.clone(),
        })
    }
}

/// Currently we assume that ExifReader<T> won't cross any threads. So there is no chance of any
/// function accessing this data at the same time in multiple threds.
/// But it's entirely possible that libraw internally calls the callback function in multiple
/// threads ( like when using openmp )
#[derive(Debug)]
#[must_use = ".data() method must be called to get back the data"]
pub struct ExifReader<T> {
    inner: Arc<ExifRead<T>>,
    libraw_data_t: NonNull<libraw_data_t>,
    libraw_data_t_dropped: Arc<AtomicBool>,
}

impl<T> Drop for ExifReader<T> {
    fn drop(&mut self) {
        if self.libraw_data_t_dropped.load(Ordering::SeqCst) {
            return;
        }
        unsafe {
            libraw_set_exifparser_handler(self.libraw_data_t.as_ptr(), None, core::ptr::null_mut());
        };
    }
}

pub struct ExifRead<T> {
    data_stream_type: DataStreamType,
    callback: Callback<T>,
    data: Mutex<T>,
    errors: Mutex<Vec<LibrawError>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for ExifRead<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExifRead")
            .field("data", &self.data)
            .field("errors", &self.errors)
            .finish()
    }
}

impl<T> ExifReader<T> {
    extern "C" fn exif_parser_callback(
        context: *mut libc::c_void,
        tag: libc::c_int,
        _type: libc::c_int,
        len: libc::c_int,
        ord: libc::c_uint,
        ifp: *mut libc::c_void,
        base: INT64,
    ) {
        let context: Arc<ExifRead<T>> = unsafe { Arc::from_raw(context as *const ExifRead<T>) };
        let mut buffer = vec![0_u8; len as usize];

        let res = unsafe {
            context.data_stream_type.read()(
                ifp as *mut libc::c_void,
                buffer.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                1,
            )
        };

        if res < 1 {
            if let Ok(mut errors) = context.errors.lock() {
                errors.push(crate::LibrawError::CustomError(
                    format!("libraw_read_datastream read {res} blocks").into(),
                ));
            }
        }

        if let Ok(mut data) = context.data.lock() {
            if let Err(e) = (context.callback)(ExifCallbackArgs::<T> {
                callback_data: &mut data,
                tag: tag & 0x0fffff, // Undo (ifdN + 1 ) << 20
                data_type: _type.into(),
                len,
                ord,
                data: buffer.as_mut_slice(),
                base,
            }) {
                if let Ok(mut errors) = context.errors.lock() {
                    errors.push(crate::LibrawError::CustomError(e));
                }
            };
        }
        core::mem::forget(context); // Don't decrement the refcount for arc we should only
                                    // decrement that when the data function is called or
                                    // ExifReader is dropped
    }

    pub fn errors(&mut self) -> Result<Vec<crate::error::LibrawError>, LibrawError> {
        let mut errors = self.inner.errors.lock().map_err(|_| {
            crate::error::LibrawError::CustomError("Unable to lock the mutex to get errors".into())
        })?;
        let mut ret: Vec<LibrawError> = Default::default();
        core::mem::swap(&mut (*errors), &mut ret);
        Ok(ret)
    }

    // pub fn data(self, processor: &mut Processor) -> Result<T, LibrawError> {
    pub fn data(self) -> Result<T, LibrawError> {
        if !self.libraw_data_t_dropped.load(Ordering::SeqCst) {
            unsafe {
                libraw_set_exifparser_handler(
                    self.libraw_data_t.as_ptr(),
                    None,
                    core::ptr::null_mut(),
                )
            };
        }
        unsafe { Arc::decrement_strong_count(Arc::as_ptr(&self.inner)) };
        let inner = unsafe { core::ptr::read(&self.inner) };
        // Since this needs to be dropped we have to take it out of self before forgetting self
        let dropped = unsafe { core::ptr::read(&self.libraw_data_t_dropped) };
        drop(dropped);
        core::mem::forget(self); // Circumvent drop impl

        Arc::try_unwrap(inner)
            .map_err(|_| LibrawError::CustomError("Multiple references to data found".into()))?
            .data
            .into_inner()
            .map_err(|_| {
                LibrawError::CustomError("Unable to consume the mutex to get owned data".into())
            })
    }
}
