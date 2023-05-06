use crate::progress::ProgressStage;

pub trait DataCallback<D>: Fn(&mut D, &str, i32) {}
impl<D, F: Fn(&mut D, &str, i32)> DataCallback<D> for F {}

pub trait ProgressCallback<D>: Fn(&mut D, ProgressStage, i32, i32) -> i32 {}
impl<D, F: Fn(&mut D, ProgressStage, i32, i32) -> i32> ProgressCallback<D> for F {}

pub trait ExifParserCallback<D>: Fn(&mut D, i32, i32, i32, u32, i32, i64) {}
impl<D, F: Fn(&mut D, i32, i32, i32, u32, i32, i64)> ExifParserCallback<D> for F {}

pub trait LibrawCallback {}

mod __ {
    use super::*;
    pub trait LibrawCallbackHelper<Type> {}
    trait DisjointFn<Args> {}

    impl<F: Fn(&mut D, &str, i32), D> DisjointFn<(&mut D, &str, i32)> for F {}
    impl<F: Fn(&mut D, ProgressStage, i32, i32) -> i32, D>
        DisjointFn<(&mut D, ProgressStage, i32, i32)> for F
    {
    }
    // impl<F: Fn(&mut D, i32, i32, i32, u32, i32, i64), D>
    //     DisjointFn<(&mut D, i32, i32, i32, u32, i32, i64)> for F
    // {
    // }

    // impl<Type, F: Fn(&mut Type, i32, i32, i32, u32, i32, i64)> LibrawCallbackHelper<F> for Type {}
}

pub struct CallbackProcessor<C, D, P> {
    callback: C,
    callback_data: D,
    inner: P,
}

impl<C, D, P> CallbackProcessor<C, D, P> {
    pub fn new(callback: C, callback_data: D, inner: P) -> Self {
        Self {
            callback,
            callback_data,
            inner,
        }
    }
}
