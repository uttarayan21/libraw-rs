// pub type progress_callback = ::core::option::Option<
//     unsafe extern "C" fn(
//         data: *mut libc::c_void,
//         stage: LibRaw_progress,
//         iteration: libc::c_int,
//         expected: libc::c_int,
//     ) -> libc::c_int,
// >;

// extern "C" {
//     pub fn libraw_set_progress_handler(
//         arg1: *mut libraw_data_t,
//         cb: progress_callback,
//         datap: *mut libc::c_void,
//     );
// }
#![allow(warnings)]

use core::sync::atomic::AtomicBool;
use std::sync::Mutex;

use alloc::sync::Arc;
use sys::{libraw_set_progress_handler, progress_callback, LibRaw_progress};

// We don't know what libraw is doing internally so better be safe and use Send + Sync
type ProgressCallback<T> = Box<dyn Fn(&mut T, LibRaw_progress, i32, i32) -> i32 + Send + Sync>;

// Since we are passing a reference of this data to the callback, we need to make sure that the
// data is not dropped while the callback is still running.
//
// We also need to figure out how to make it so that this doesn't get dropped as long as the set
// progress callback is valid.
pub struct ProgressData<T: Send + Sync> {
    callback: Arc<ProgressCallback<T>>, // We don't need mutable access to the callback
    data: Arc<Mutex<T>>, // We need mutable access to the data from the callback and normal access
    // from the outside
    cancel: Arc<AtomicBool>, // AtomicBool is thread safe by itself so no Mutex/RwLock needed
}

impl crate::Processor {
    // The callback is called on progress update and a &mut ptr of the data is passed along with
    // it.
    //
    // The actual data pointer we pass to the callback is a pointer to a data structure containing the owned
    // data, the Box<Callback>
    //
    // This way the data and the callback stays in rust and can be dropped safely
    pub fn set_progress_callback<T, F>(
        &mut self,
        callback: F,
        data: T,
    ) -> Result<ProgressData<T>, crate::error::LibrawError>
    where
        F: Fn(&mut T, LibRaw_progress, i32, i32) -> i32 + Send + Sync + 'static,
        T: Send + Sync,
    {
        let mut progress_data = ProgressData {
            callback: Arc::new(Box::new(callback)),
            data: Arc::new(Mutex::new(data)),
            cancel: Arc::new(AtomicBool::new(false)),
        };

        let data_ptr = core::ptr::addr_of_mut!(progress_data) as *mut libc::c_void;

        // Libraw calls this callback which in turn gets the data and inside it is a pointer to the
        // acutal callback used
        extern "C" fn progress_callback<T: Send + Sync>(
            data: *mut libc::c_void,
            stage: LibRaw_progress,
            iteration: i32,
            expected: i32,
        ) -> i32 {
            // Only use a immutable pointer to the data since we don't need mutable access to it
            // and everything inside it is wrapped in Arc / Arc<Mutex> so it is thread safe
            let progress_data = unsafe { &*(data as *const ProgressData<T>) };

            let mut data = match progress_data.data.lock() {
                Ok(data) => data,
                Err(_) => return 0,
            };

            (progress_data.callback)(&mut *data, stage, iteration, expected)
        }

        unsafe {
            libraw_set_progress_handler(self.inner, Some(progress_callback::<T>), data_ptr);
        }

        Ok(progress_data)
    }
}

// On dropping the ProgressData, we need to set the callback to None and the data pointer to null
// so that libraw doesn't try to call the callback anymore
