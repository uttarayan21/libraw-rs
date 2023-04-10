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

use core::ptr::NonNull;
use core::sync::atomic::AtomicBool;
use std::sync::Mutex;

use alloc::sync::Arc;
use sys::{libraw_set_progress_handler, progress_callback, LibRaw_progress};

use crate::LibrawError;

// We don't know what libraw is doing internally so better be safe and use Send + Sync
type ProgressCallback<T> = Box<dyn Fn(ProgressCallbackArgs<T>) -> i32 + Send + Sync>;
pub struct ProgressCallbackArgs<'a, T: Send + Sync> {
    pub data: &'a mut T,
    pub stage: LibRaw_progress,
    pub iteration: i32,
    pub expected: i32,
}

#[must_use = "ProgressMonitor must be used else it will get immediately dropped and the functions will no loner have a callback"]
pub struct ProgressMonitor<T: Send + Sync> {
    inner: Arc<ProgressData<T>>,
    libraw_data_t: NonNull<sys::libraw_data_t>,
    libraw_data_t_dropped: Arc<AtomicBool>,
}

impl<T: Send + Sync> Drop for ProgressMonitor<T> {
    fn drop(&mut self) {
        // We need to make sure that the libraw_data_t is not dropped before setting the null
        // callback
        if !self
            .libraw_data_t_dropped
            .load(core::sync::atomic::Ordering::SeqCst)
        {
            unsafe {
                libraw_set_progress_handler(self.libraw_data_t.as_ptr(), None, std::ptr::null_mut())
            }
        }
    }
}
// Since we are passing a reference of this data to the callback, we need to make sure that the
// data is not dropped while the callback is still running.
//
// We also need to figure out how to make it so that this doesn't get dropped as long as the set
// progress callback is valid.
pub struct ProgressData<T: Send + Sync> {
    callback: ProgressCallback<T>, // We don't need mutable access to the callback
    data: Mutex<T>, // We need mutable access to the data from the callback and normal access
    // from the outside
    cancel: Arc<AtomicBool>, // AtomicBool is thread safe by itself so no Mutex/RwLock needed
}

impl<T: Send + Sync> ProgressData<T> {
    // Libraw calls this callback which in turn gets the data and inside it is a pointer to the
    // acutal callback used
    extern "C" fn progress_callback(
        data: *mut libc::c_void,
        stage: LibRaw_progress,
        iteration: i32,
        expected: i32,
    ) -> i32 {
        // Only use a immutable pointer to the data since we don't need mutable access to it
        // and everything inside it is wrapped in Arc / Arc<Mutex> so it is thread safe
        let progress_data: Arc<ProgressData<T>> =
            unsafe { Arc::from_raw(data as *const ProgressData<T>) };

        // Return non-zero for cancelling the processing using callback
        if progress_data
            .cancel
            .load(core::sync::atomic::Ordering::Relaxed)
        {
            return 1;
        }

        let mut data = match progress_data.data.lock() {
            Ok(data) => data,
            Err(_) => return 1,
        };

        let ret = (progress_data.callback)(ProgressCallbackArgs {
            data: &mut *data,
            stage,
            iteration,
            expected,
        });
        drop(data);
        core::mem::forget(progress_data);
        ret
    }
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
    ) -> Result<ProgressMonitor<T>, crate::error::LibrawError>
    where
        F: Fn(ProgressCallbackArgs<T>) -> i32 + Send + Sync + 'static,
        T: Send + Sync,
    {
        let progress_data = ProgressData {
            callback: Box::new(callback),
            data: Mutex::new(data),
            cancel: Arc::new(AtomicBool::new(false)),
        };
        let inner = Arc::new(progress_data);

        unsafe {
            libraw_set_progress_handler(
                self.inner.as_ptr(),
                Some(ProgressData::<T>::progress_callback),
                Arc::<ProgressData<T>>::into_raw(Arc::clone(&inner)) as *mut libc::c_void,
            );
        }

        Ok(ProgressMonitor {
            inner,
            libraw_data_t: self.inner,
            libraw_data_t_dropped: self.dropped.clone(),
        })
    }
}

// On dropping the ProgressMonitor, we need to set the callback to None and the data pointer to null
// so that libraw doesn't try to call the callback anymore
impl<T: Send + Sync> ProgressMonitor<T> {
    pub fn cancel(&self) {
        self.inner
            .cancel
            .store(true, core::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner
            .cancel
            .load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn data(self) -> Result<T, LibrawError> {
        if !self
            .libraw_data_t_dropped
            .load(core::sync::atomic::Ordering::SeqCst)
        {
            unsafe {
                libraw_set_progress_handler(self.libraw_data_t.as_ptr(), None, std::ptr::null_mut())
            };
        }
        unsafe { Arc::decrement_strong_count(Arc::as_ptr(&self.inner)) };
        let inner = unsafe { core::ptr::read(&self.inner) };
        let dropped = unsafe { core::ptr::read(&self.libraw_data_t_dropped) };
        drop(dropped);
        core::mem::forget(self);

        Arc::try_unwrap(inner)
            .map_err(|_| LibrawError::CustomError("Failed to unwrap Arc".into()))?
            .data
            .into_inner()
            .map_err(|_| LibrawError::CustomError("Failed to unwrap Mutex".into()))
    }
}
