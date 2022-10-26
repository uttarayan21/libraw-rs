use std::{cell::RefCell, rc::Rc};

use libraw_sys::*;

use crate::{LibrawError, Processor};

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
    pub fn set_exif_callback<T: std::fmt::Debug, F>(
        &mut self,
        data: T,
        data_type: DataStreamType,
        callback: F,
    ) -> Result<ExifReader<T>, crate::error::LibrawError>
    where
        F: Fn(&mut T, i32, i32, i32, u32, &mut [u8], i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + 'static,
    {
        let eread = ExifRead {
            callback: Rc::new(Box::new(callback)),
            data: Rc::new(RefCell::new(data)),
            errors: Default::default(),
            data_type,
        };
        let eread = Rc::new(RefCell::new(eread));

        unsafe {
            libraw_set_exifparser_handler(
                self.inner,
                Some(ExifReader::<T>::exif_parser_callback),
                std::mem::transmute(Rc::clone(&eread)),
            );
        };

        Ok(ExifReader(eread))
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ExifReader<T>(Rc<RefCell<ExifRead<T>>>);

pub struct ExifRead<T> {
    data_type: DataStreamType,
    callback: Rc<
        Box<
            dyn Fn(
                &mut T,
                i32,
                i32,
                i32,
                u32,
                &mut [u8],
                i64,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
        >,
    >,
    data: Rc<RefCell<T>>,
    errors: Rc<RefCell<Vec<LibrawError>>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for ExifRead<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExifRead")
            .field("data", &self.data)
            .field("errors", &self.errors)
            .finish()
    }
}

impl<T: std::fmt::Debug> ExifReader<T> {
    pub extern "C" fn exif_parser_callback(
        context: *mut libc::c_void,
        tag: libc::c_int,
        _type: libc::c_int,
        len: libc::c_int,
        ord: libc::c_uint,
        ifp: *mut libc::c_void,
        base: INT64,
    ) {
        let context: Rc<RefCell<ExifRead<T>>> = unsafe { std::mem::transmute(context) };
        unsafe { Rc::increment_strong_count(Rc::as_ptr(&context)) };
        let mut buffer = vec![0_u8; len as usize];

        let context = context.borrow_mut();
        let res = unsafe {
            context.data_type.read()(
                ifp as *mut libc::c_void,
                buffer.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                1,
            )
        };

        if res < 1 {
            context.errors.borrow_mut().push(
                crate::LibrawError::InternalError(
                    crate::error::InternalLibrawError::IoError,
                    format!("libraw_read_datastream read {res} blocks"),
                )
                .into(),
            );
        }

        let mut data = context.data.borrow_mut();
        if let Err(e) =
            (context.callback)(&mut data, tag, _type, len, ord, buffer.as_mut_slice(), base)
        {
            context
                .errors
                .borrow_mut()
                .push(LibrawError::CustomError(e));
        };
    }

    pub fn errors(&self) -> Rc<RefCell<Vec<crate::error::LibrawError>>> {
        Rc::clone(&self.0.borrow().errors)
    }

    pub fn data(self, processor: &mut Processor) -> Result<T, LibrawError> {
        unsafe { libraw_set_exifparser_handler(processor.inner, None, std::ptr::null_mut()) };
        unsafe { Rc::decrement_strong_count(Rc::as_ptr(&self.0)) };
        Ok(Rc::try_unwrap(
            Rc::try_unwrap(self.0)
                .map_err(|_| {
                    LibrawError::CustomError("Multiple References found for ExifReader".into())
                })?
                .into_inner()
                .data,
        )
        .map_err(|_| LibrawError::CustomError("Multiple References found for data".into()))?
        .into_inner())
    }
}
