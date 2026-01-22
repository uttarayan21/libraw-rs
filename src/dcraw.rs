use crate::*;
impl Processor {
    pub fn dcraw_process_make_mem_thumb(&mut self) -> Result<ProcessedImage, LibrawError> {
        let mut errc = 0;
        let data = unsafe { sys::libraw_dcraw_make_mem_thumb(self.inner.as_ptr(), &mut errc) };
        assert!(!data.is_null());
        LibrawError::to_result(
            errc,
            ProcessedImage {
                inner: NonNull::new(data).expect("Not Null"),
            },
        )
    }

    pub fn dcraw_process(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_dcraw_process(self.inner.as_ptr()) })?;
        Ok(())
    }

    pub fn dcraw_process_make_mem_image(&mut self) -> Result<ProcessedImage, LibrawError> {
        let mut errc = 0;
        let data = unsafe { sys::libraw_dcraw_make_mem_image(self.inner.as_ptr(), &mut errc) };
        assert!(!data.is_null());
        LibrawError::to_result(
            errc,
            ProcessedImage {
                inner: NonNull::new(data).expect("Not null"),
            },
        )
    }

    pub fn dcraw_ppm_tiff_writer(
        self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), LibrawError> {
        LibrawError::check(unsafe {
            sys::libraw_dcraw_ppm_tiff_writer(self.inner.as_ptr(), path_to_cstr(path)?.as_ptr())
        })?;
        Ok(())
    }
}
