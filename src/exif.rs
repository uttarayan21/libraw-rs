use std::{cell::RefCell, rc::Rc};

use libraw_sys::*;

use crate::{LibrawError, Processor};
pub type Callback<T> = Box<
    dyn Fn(
        &mut T,
        i32,
        DataType,
        i32,
        u32,
        &mut [u8],
        i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
>;

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
    /// The callback should return Ok(()) if the exif data was parsed successfully.
    /// The callback should return Err(LibrawError) if the exif data could not be parsed.
    ///
    /// Args:
    ///    data: The data we pass to the function to act as a temp storage
    ///    data_steam_type: What type of data stream we are using (file, bigfile, buffer)
    ///    callback: The callback function that will be called with the exif data as a byte slice.
    ///       Args:
    ///         data: &mut T => The data we pass to the function to act as a temp storage
    ///         tag:  i32    => The tag of the exif data
    ///         type: i32    => The type of the exif data
    ///         len:  i32    => The length of the exif data
    ///         ord:  u32    => The order of the exif data
    ///         data: &[u8]  => The exif data as a byte slice
    ///         base: i64    => Not sure
    pub fn set_exif_callback<T: std::fmt::Debug, F>(
        &mut self,
        data: T,
        data_stream_type: DataStreamType,
        callback: F,
    ) -> Result<ExifReader<T>, crate::error::LibrawError>
    where
        F: Fn(
                &mut T,
                i32,
                DataType,
                i32,
                u32,
                &mut [u8],
                i64,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + 'static,
    {
        let eread = ExifRead {
            callback: Rc::new(Box::new(callback)),
            data: Rc::new(RefCell::new(data)),
            errors: Default::default(),
            data_stream_type,
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
    data_stream_type: DataStreamType,
    callback: Rc<Callback<T>>,
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
    unsafe extern "C" fn exif_parser_callback(
        context: *mut libc::c_void,
        tag: libc::c_int,
        _type: libc::c_int,
        len: libc::c_int,
        ord: libc::c_uint,
        ifp: *mut libc::c_void,
        base: INT64,
    ) {
        let context: Rc<RefCell<ExifRead<T>>> = std::mem::transmute(context);
        Rc::increment_strong_count(Rc::as_ptr(&context));
        let mut buffer = vec![0_u8; len as usize];

        let context = context.borrow_mut();
        let res = context.data_stream_type.read()(
            ifp as *mut libc::c_void,
            buffer.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
            buffer.len(),
            1,
        );

        if res < 1 {
            context
                .errors
                .borrow_mut()
                .push(crate::LibrawError::InternalError(
                    crate::error::InternalLibrawError::IoError,
                    format!("libraw_read_datastream read {res} blocks"),
                ));
        }

        let mut data = context.data.borrow_mut();
        if let Err(e) = (context.callback)(
            &mut data,
            tag & 0x0fffff, // Undo (ifdN + 1 ) << 20
            _type.into(),
            len,
            ord,
            buffer.as_mut_slice(),
            base,
        ) {
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
