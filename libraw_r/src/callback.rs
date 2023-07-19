use alloc::sync::{Arc, Weak};
// uno_deadlockslot::Mutex;
use parking_lot::Mutex;
use core::mem::ManuallyDrop;

use crate::exif::{ByteOrder, ExifType};
use crate::progress::ProgressStage;
use crate::EmptyProcessor;
// use core::ops::{Deref, DerefMut};

pub struct Callbacks<
    DD,
    PD,
    ED,
    DC = Box<dyn Fn(DataCallbackArgs<DD>)>,
    PC = Box<dyn Fn(ProgressCallbackArgs<PD>) -> i32>,
    EC = Box<dyn Fn(ExifParserCallbackArgs<ED>)>,
> where
    DC: DataCallback<DD>,
    PC: ProgressCallback<PD>,
    EC: ExifParserCallback<ED>,
{
    pub data_callback: Option<Arc<CallbackData<DC, DD>>>,
    pub progress_callback: Option<Arc<CallbackData<PC, PD>>>,
    pub exif_parser_callback: Option<Arc<CallbackData<EC, ED>>>,
}

impl<D, PD, ED, DC: DataCallback<D>, PC: ProgressCallback<PD>, EC: ExifParserCallback<ED>> Default
    for Callbacks<D, PD, ED, DC, PC, EC>
{
    fn default() -> Callbacks<D, PD, ED, DC, PC, EC> {
        Self {
            data_callback: None,
            progress_callback: None,
            exif_parser_callback: None,
        }
    }
}

pub struct CallbackData<CB, D> {
    callback: CB,
    data: Mutex<D>,
}

pub struct DataCallbackArgs<'d, D: 'd> {
    pub data: &'d mut D,
    pub path: &'d str,
    pub error: i32,
}

pub trait DataCallback<D>: Fn(DataCallbackArgs<D>) {}
impl<D, F: Fn(DataCallbackArgs<D>)> DataCallback<D> for F {}

pub struct ProgressCallbackArgs<'d, D: 'd> {
    pub data: &'d mut D,
    pub stage: ProgressStage,
    pub iteration: libc::c_int,
    pub expected: libc::c_int,
}

pub trait ProgressCallback<D>: Fn(ProgressCallbackArgs<D>) -> i32 {}
impl<D, F: Fn(ProgressCallbackArgs<D>) -> i32> ProgressCallback<D> for F {}

pub struct ExifParserCallbackArgs<'d, D: 'd> {
    pub context: &'d mut D,
    pub tag: libc::c_int,
    pub exif_type: ExifType,
    pub len: libc::c_int,
    pub ord: ByteOrder,
    pub ifp: &'d [u8],
    pub base: i64,
}

pub trait ExifParserCallback<D>: Fn(ExifParserCallbackArgs<D>) {}
impl<D, F: Fn(ExifParserCallbackArgs<D>)> ExifParserCallback<D> for F {}

impl<DD, PD, ED> EmptyProcessor<DD, PD, ED> {
    // pub fn set_data_callback<NDD, NDC: DataCallback<NDD> + 'static>(
    //     self,
    //     callback: NDC,
    //     data: NDD,
    // ) -> EmptyProcessor<NDD, PD, ED> {
    //     let mut s = ManuallyDrop::new(self);
    //     let dc = core::mem::take(&mut s.callbacks.data_callback);
    //     drop(dc);
    //     let pc = core::mem::take(&mut s.callbacks.progress_callback);
    //     let ec = core::mem::take(&mut s.callbacks.exif_parser_callback);
    //     let dc: CallbackData<Box<dyn Fn(DataCallbackArgs<NDD>)>, NDD> = CallbackData {
    //         callback: Box::new(callback),
    //         data: parking_lot::Mutex::new(data),
    //     };
    //     let dc = Arc::new(dc);

    //     // using into_raw because we are not going to free the memory unless we call the finall
    //     unsafe {
    //         sys::libraw_set_dataerror_handler(
    //             s.inner.as_ptr(),
    //             Some(data_callback::<NDC, NDD>),
    //             Arc::downgrade(&dc).as_ptr().cast::<libc::c_void>() as *mut _,
    //             // Arc::into_raw(dc).cast::<libc::c_void>() as *mut _,
    //         );
    //     }
    //     EmptyProcessor {
    //         inner: s.inner,
    //         callbacks: Callbacks::<NDD, PD, ED> {
    //             data_callback: Some(dc),
    //             progress_callback: pc,
    //             exif_parser_callback: ec,
    //         },
    //     }
    // }

    // // Since this takes a &mut reference to self it's not possible for other functions to run with
    // // this parallel with this one.
    // pub fn reset_data_callback(&mut self) -> Option<DD>
    // where
    //     DD: Default,
    // {
    //     if let Some(ref mut cb) = self.callbacks.data_callback {
    //         // This is horribly unsafe don't do this.
    //         // We take the weak pointer and cast it to an Arc and since we are sure that we create
    //         // only one instance of the Arc
    //         // However this drops the arc pointer so we must forget it
    //         // let cb = cb.as_ptr();
    //         // let cb = unsafe { Arc::from_raw(cb) };
    //         // let mut cb = ManuallyDrop::new(cb); // Skip dropping since this is basically a weak
    //         // pointer and we don't want to drop the data by
    //         // decrementing the actual strong count
    //         if let Some(cb) = Arc::get_mut(cb) {
    //             let data: &mut DD = &mut cb.data.lock();
    //             Some(core::mem::take(data))
    //         } else {
    //             None
    //         }
    //     } else {
    //         None
    //     }
    // }

    // pub fn remove_data_callback(mut self) -> Self {
    //     let adc = core::mem::take(&mut self.callbacks.data_callback);
    //     drop(adc); // Since we drop the arc pointer we can on the next call of the callback modify
    //                // the callback and set it to null

    //     unsafe {
    //         sys::libraw_set_dataerror_handler(self.inner.as_ptr(), None, core::ptr::null_mut());
    //     }
    //     self
    // }

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
    pub fn set_exif_parser_callback<
        NED: std::fmt::Debug,
        NEC: ExifParserCallback<NED> + 'static,
    >(
        self,
        callback: NEC,
        data: NED,
    ) -> EmptyProcessor<DD, PD, NED> {
        let mut s = ManuallyDrop::new(self);
        let dc = core::mem::take(&mut s.callbacks.data_callback);
        let pc = core::mem::take(&mut s.callbacks.progress_callback);
        let ec = core::mem::take(&mut s.callbacks.exif_parser_callback);
        drop(ec);
        let ec: CallbackData<Box<dyn Fn(ExifParserCallbackArgs<NED>)>, NED> = CallbackData {
            callback: Box::new(callback),
            data: Mutex::new(data),
        };
        let ec = Arc::new(ec);
        let weak_ec = Arc::downgrade(&ec);
        dbg!(weak_ec.strong_count());
        dbg!(weak_ec.weak_count());

        dbg!(&ec.data);

        unsafe {
            sys::libraw_set_exifparser_handler(
                s.inner.as_ptr(),
                Some(exif_parser_callback::<NEC, NED>),
                weak_ec.as_ptr().cast::<libc::c_void>() as *mut _,
            );
        }
        core::mem::forget(weak_ec);
        EmptyProcessor {
            inner: s.inner,
            callbacks: Callbacks::<DD, PD, NED> {
                data_callback: dc,
                progress_callback: pc,
                exif_parser_callback: Some(ec),
            },
        }
    }

    pub fn reset_exif_callback(&mut self) -> Option<ED>
    where
        ED: Default,
    {
        if let Some(ref mut cb) = self.callbacks.exif_parser_callback {
            if let Some(cb) = Arc::get_mut(cb) {
                dbg!("Locking");
                let data: &mut ED = &mut cb.data.lock();
                Some(core::mem::take(data))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn remove_exif_callback(mut self) -> Self {
        // if let Some(ref mut wec) = self.callbacks.exif_parser_callback {
        //     if let Some(aec) = wec.upgrade() {
        //         drop(aec);
        //     } else {
        //         println!("Unable to drop the exif callback");
        //     }
        // }
        let aec = core::mem::take(&mut self.callbacks.exif_parser_callback);
        drop(aec);

        unsafe {
            sys::libraw_set_exifparser_handler(self.inner.as_ptr(), None, core::ptr::null_mut());
        }
        self
    }

    // pub fn set_progress_callback<NPD, NPC: ProgressCallback<NPD> + 'static>(
    //     self,
    //     callback: NPC,
    //     data: NPD,
    // ) -> EmptyProcessor<DD, NPD, ED> {
    //     let mut s = ManuallyDrop::new(self);
    //     let dc = core::mem::take(&mut s.callbacks.data_callback);
    //     let pc = core::mem::take(&mut s.callbacks.progress_callback);
    //     drop(pc);
    //     let ec = core::mem::take(&mut s.callbacks.exif_parser_callback);

    //     let pc: CallbackData<Box<dyn Fn(ProgressCallbackArgs<NPD>) -> i32>, NPD> = CallbackData {
    //         callback: Box::new(callback),
    //         data: parking_lot::Mutex::new(data),
    //     };
    //     let pc = Arc::new(pc);

    //     unsafe {
    //         sys::libraw_set_progress_handler(
    //             s.inner.as_ptr(),
    //             Some(progress_callback::<NPC, NPD>),
    //             Arc::downgrade(&pc).as_ptr().cast::<libc::c_void>() as *mut _,
    //         );
    //     }
    //     EmptyProcessor {
    //         inner: s.inner,
    //         callbacks: Callbacks::<DD, NPD, ED> {
    //             data_callback: dc,
    //             progress_callback: Some(pc),
    //             exif_parser_callback: ec,
    //         },
    //     }
    // }

    // pub fn reset_progress_callback(&mut self) -> Option<PD>
    // where
    //     PD: Default,
    // {
    //     if let Some(ref mut cb) = self.callbacks.progress_callback {
    //         if let Some(cb) = Arc::get_mut(cb) {
    //             let data: &mut PD = &mut cb.data.lock();
    //             Some(core::mem::take(data))
    //         } else {
    //             None
    //         }
    //     } else {
    //         None
    //     }
    // }

    // pub fn remove_progress_callback(mut self) -> Self {
    //     let wdc = core::mem::take(&mut self.callbacks.progress_callback);
    //     drop(wdc);

    //     unsafe {
    //         sys::libraw_set_progress_handler(self.inner.as_ptr(), None, core::ptr::null_mut());
    //     }
    //     self.callbacks.data_callback = None;
    //     self
    // }
}

unsafe extern "C" fn data_callback<DC: DataCallback<D>, D>(
    data: *mut libc::c_void,
    path: *const libc::c_char,
    error: libc::c_int,
) {
    assert!(!data.is_null());
    let data: &mut CallbackData<DC, D> = &mut *data.cast();
    let path = std::ffi::CStr::from_ptr(path)
        .to_str()
        .expect("Unable to get str from path");
    (data.callback)(DataCallbackArgs {
        data: &mut data.data.lock(),
        path,
        error,
    })
}

unsafe extern "C" fn progress_callback<PC: ProgressCallback<PD>, PD>(
    data: *mut libc::c_void,
    stage: sys::LibRaw_progress,
    iteration: libc::c_int,
    expected: libc::c_int,
) -> libc::c_int {
    assert!(!data.is_null());
    let mut odata: Arc<CallbackData<PC, PD>> = Arc::from_raw(data.cast());
    let data = Arc::get_mut(&mut odata).expect("Unable to get mut ref from Arc");

    let ret = (data.callback)(ProgressCallbackArgs {
        data: &mut data.data.lock(),
        stage: ProgressStage::from(stage),
        iteration,
        expected,
    });
    core::mem::forget(odata);
    ret
}

unsafe extern "C" fn exif_parser_callback<EC: ExifParserCallback<ED>, ED: std::fmt::Debug>(
    context: *mut libc::c_void,
    tag: libc::c_int,
    type_: libc::c_int,
    len: libc::c_int,
    ord: libc::c_uint,
    ifp: *mut libc::c_void,
    base: sys::INT64,
) {
    assert!(!context.is_null());
    let wik = Weak::from_raw(context.cast());
    let context: Arc<CallbackData<EC, ED>> = wik.upgrade().unwrap();
    drop(wik);
    let ifp = std::slice::from_raw_parts(ifp.cast::<u8>(), len.clamp(0, i32::MAX) as usize);
    let c = context.data.try_lock();
    if context.data.is_locked() {
        dbg!("Data is locked");
    }

    drop(c);
    // assert_eq!(
    //     // core::mem::size_of::<std::collections::HashMap::<String, String>>(),
    //     core::mem::size_of_val(&parking_lot::Mutex::new(std::collections::HashMap::<
    //         String,
    //         String,
    //     >::new())),
    //     core::mem::size_of_val(&context.data)
    // );
    (context.callback)(ExifParserCallbackArgs {
        context: &mut context.data.lock(),
        tag,
        exif_type: ExifType::from(type_),
        len,
        ord: ByteOrder::from_ord(ord),
        ifp,
        base,
    });
    // core::mem::forget(Arc::downgrade(&ocontext));
    // dbg!(Arc::strong_count(&ocontext));
}
