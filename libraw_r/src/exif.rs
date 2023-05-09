#[repr(u32)]
pub enum ByteOrder {
    BigEndian = 0x4d4d,
    LittleEndian = 0x4949,
    Unknown,
}

impl ByteOrder {
    pub const MM: ByteOrder = ByteOrder::BigEndian;
    pub const II: ByteOrder = ByteOrder::LittleEndian;

    pub fn from_ord(ord: u32) -> Self {
        match ord {
            0x4949 => ByteOrder::LittleEndian,
            0x4d4d => ByteOrder::BigEndian,
            _ => ByteOrder::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct ExifCallbackArgs<'a, T> {
    pub callback_data: &'a mut T,
    pub tag: i32,
    pub data_type: ExifType,
    pub len: i32,
    pub ord: u32,
    pub data: &'a mut [u8],
    pub base: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum ExifType {
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

impl From<libc::c_int> for ExifType {
    fn from(value: libc::c_int) -> Self {
        match value {
            1 => ExifType::Byte,
            2 => ExifType::Ascii,
            3 => ExifType::Short,
            4 => ExifType::Long,
            5 => ExifType::Rational,
            6 => ExifType::SByte,
            7 => ExifType::Undefined,
            8 => ExifType::SShort,
            9 => ExifType::SLong,
            10 => ExifType::SRational,
            11 => ExifType::Float,
            12 => ExifType::Double,
            _ => ExifType::Undefined,
        }
    }
}

impl From<ExifType> for libc::c_int {
    fn from(value: ExifType) -> Self {
        match value {
            ExifType::Byte => 1,
            ExifType::Ascii => 2,
            ExifType::Short => 3,
            ExifType::Long => 4,
            ExifType::Rational => 5,
            ExifType::SByte => 6,
            ExifType::Undefined => 7,
            ExifType::SShort => 8,
            ExifType::SLong => 9,
            ExifType::SRational => 10,
            ExifType::Float => 11,
            ExifType::Double => 12,
        }
    }
}


//impl<D> Processor<'_, D> {
//    pub fn set_exif_callback<T, F>(
//        &mut self,
//        data: T,
//        callback: F,
//    ) -> Result<(), crate::error::LibrawError>
//    where
//        F: Fn(ExifCallbackArgs<T>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
//            + 'static,
//    {
//        todo!()
//    }
//}

// /// Currently we assume that ExifReader<T> won't cross any threads. So there is no chance of any
// /// function accessing this data at the same time in multiple threds.
// /// But it's entirely possible that libraw internally calls the callback function in multiple
// /// threads ( like when using openmp )
// #[derive(Debug)]
// #[must_use = ".data() method must be called to get back the data"]
// pub struct ExifReader<T> {
//     inner: Arc<ExifRead<T>>,
//     libraw_data_t: NonNull<libraw_data_t>,
//     libraw_data_t_dropped: Arc<AtomicBool>,
// }

// impl<T> Drop for ExifReader<T> {
//     fn drop(&mut self) {
//         if self.libraw_data_t_dropped.load(Ordering::SeqCst) {
//             return;
//         }
//         unsafe {
//             libraw_set_exifparser_handler(self.libraw_data_t.as_ptr(), None, core::ptr::null_mut());
//         };
//     }
// }

// pub struct ExifRead<T> {
//     data_stream_type: DataStreamType,
//     callback: Callback<T>,
//     data: Mutex<T>,
//     errors: Mutex<Vec<LibrawError>>,
// }

// impl<T: std::fmt::Debug> std::fmt::Debug for ExifRead<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ExifRead")
//             .field("data", &self.data)
//             .field("errors", &self.errors)
//             .finish()
//     }
// }

// impl<T> ExifReader<T> {
//     extern "C" fn exif_parser_callback(
//         context: *mut libc::c_void,
//         tag: libc::c_int,
//         _type: libc::c_int,
//         len: libc::c_int,
//         ord: libc::c_uint,
//         ifp: *mut libc::c_void,
//         base: INT64,
//     ) {
//         let context: Arc<ExifRead<T>> = unsafe { Arc::from_raw(context as *const ExifRead<T>) };
//         let mut buffer = vec![0_u8; len as usize];

//         let res = unsafe {
//             context.data_stream_type.read()(
//                 ifp as *mut libc::c_void,
//                 buffer.as_mut_slice().as_mut_ptr() as *mut libc::c_void,
//                 buffer.len(),
//                 1,
//             )
//         };

//         if res < 1 {
//             if let Ok(mut errors) = context.errors.lock() {
//                 errors.push(crate::LibrawError::CustomError(
//                     format!("libraw_read_datastream read {res} blocks").into(),
//                 ));
//             }
//         }

//         if let Ok(mut data) = context.data.lock() {
//             if let Err(e) = (context.callback)(ExifCallbackArgs::<T> {
//                 callback_data: &mut data,
//                 tag: tag & 0x0fffff, // Undo (ifdN + 1 ) << 20
//                 data_type: _type.into(),
//                 len,
//                 ord,
//                 data: buffer.as_mut_slice(),
//                 base,
//             }) {
//                 if let Ok(mut errors) = context.errors.lock() {
//                     errors.push(crate::LibrawError::CustomError(e));
//                 }
//             };
//         }
//         core::mem::forget(context); // Don't decrement the refcount for arc we should only
//                                     // decrement that when the data function is called or
//                                     // ExifReader is dropped
//     }

//     pub fn errors(&mut self) -> Result<Vec<crate::error::LibrawError>, LibrawError> {
//         let mut errors = self.inner.errors.lock().map_err(|_| {
//             crate::error::LibrawError::CustomError("Unable to lock the mutex to get errors".into())
//         })?;
//         let mut ret: Vec<LibrawError> = Default::default();
//         core::mem::swap(&mut (*errors), &mut ret);
//         Ok(ret)
//     }

//     // pub fn data(self, processor: &mut Processor) -> Result<T, LibrawError> {
//     pub fn data(self) -> Result<T, LibrawError> {
//         if !self.libraw_data_t_dropped.load(Ordering::SeqCst) {
//             unsafe {
//                 libraw_set_exifparser_handler(
//                     self.libraw_data_t.as_ptr(),
//                     None,
//                     core::ptr::null_mut(),
//                 )
//             };
//         }
//         unsafe { Arc::decrement_strong_count(Arc::as_ptr(&self.inner)) };
//         let inner = unsafe { core::ptr::read(&self.inner) };
//         // Since this needs to be dropped we have to take it out of self before forgetting self
//         let dropped = unsafe { core::ptr::read(&self.libraw_data_t_dropped) };
//         drop(dropped);
//         core::mem::forget(self); // Circumvent drop impl

//         Arc::try_unwrap(inner)
//             .map_err(|_| LibrawError::CustomError("Multiple references to data found".into()))?
//             .data
//             .into_inner()
//             .map_err(|_| {
//                 LibrawError::CustomError("Unable to consume the mutex to get owned data".into())
//             })
//     }
// }
