#[macro_use]
pub mod error;
pub mod dcraw;
pub mod defaults;
#[cfg(feature = "exif")]
pub mod exif;
pub mod orientation;
pub mod progress;
pub mod traits;

use alloc::sync::Arc;
pub use error::LibrawError;

extern crate alloc;
extern crate libraw_sys as sys;
use core::ptr::NonNull;
use core::sync::atomic::AtomicBool;
use semver::Version;
use std::ffi::CString;
use std::ops::Drop;
use std::path::Path;

/// Returns the version of libraw
pub const fn version() -> Version {
    Version {
        major: sys::LIBRAW_MAJOR_VERSION as u64,
        minor: sys::LIBRAW_MINOR_VERSION as u64,
        patch: sys::LIBRAW_PATCH_VERSION as u64,
        pre: semver::Prerelease::EMPTY,
        build: semver::BuildMetadata::EMPTY,
    }
}

/// A struct wrapping the libraw_data_t type
pub struct Processor {
    inner: NonNull<sys::libraw_data_t>,
    dropped: Arc<AtomicBool>,
}

/// You can pass the Processor to another thread since it doesn't use any thread_local values
unsafe impl Send for Processor {}
/// You can pass the reference to Processor to another thread since it cannot open / close / drop
/// Without a mutable reference to Self
unsafe impl Sync for Processor {}

// impl Deref for Processor {
//     type Target = *mut sys::libraw_data_t;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl DerefMut for Processor {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl Drop for Processor {
    fn drop(&mut self) {
        unsafe {
            sys::libraw_free_image(self.inner.as_ptr());
            sys::libraw_close(self.inner.as_ptr());
        }
        self.dropped
            .store(true, core::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for Processor {
    /// Returns libraw_init(0)
    fn default() -> Self {
        Self::new(LibrawConstructorFlags::None)
    }
}

impl Processor {
    pub fn thumbs_list(&self) -> &sys::libraw_thumbnail_list_t {
        unsafe { &self.inner.as_ref().thumbs_list }
    }
    pub fn unpack_thumb_ex(&mut self, index: libc::c_int) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack_thumb_ex(self.inner.as_ptr(), index) })?;
        Ok(())
    }

    /// Drop the processor and get a handle to the inner type
    ///
    /// Processor also implements DerefMut so you can take that if you want
    pub fn into_inner(self) -> NonNull<sys::libraw_data_t> {
        self.inner
    }

    /// Get mutable inner
    ///
    /// # Safety
    /// Since This returns a &mut reference to the inner type, you can do anything with it
    /// including calling libraw_close and libraw_free_image
    /// This is unsafe because you can cause UB by doing this
    /// If you want to drop the processor, use Processor::drop
    pub unsafe fn inner_mut(&mut self) -> &mut NonNull<sys::libraw_data_t> {
        &mut self.inner
    }

    /// Build Processor with options and params
    pub fn builder() -> ProcessorBuilder {
        ProcessorBuilder::default()
    }

    /// Calls libraw_init with the any of the constructor flags
    /// # May panic
    pub fn new(option: LibrawConstructorFlags) -> Self {
        let inner = unsafe { sys::libraw_init(option as u32) };
        assert!(!inner.is_null());
        Self {
            inner: NonNull::new(inner).expect("Failed to initialize libraw"),
            dropped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn try_new(option: LibrawConstructorFlags) -> Result<Self, LibrawError> {
        let inner = unsafe { sys::libraw_init(option as u32) };
        if inner.is_null() {
            Err(LibrawError::CustomError(
                "Got back null pointer from libraw_init(0)".into(),
            ))
        } else {
            Ok(Self {
                inner: NonNull::new(inner).expect("Failed to initialize libraw"),
                dropped: Arc::new(AtomicBool::new(false)),
            })
        }
    }

    /// Calls libraw_open_file
    ///
    /// Fallback to libraw_open_wfile on windows if the open fails
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<(), LibrawError> {
        if !path.as_ref().exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").into());
        }

        self.recycle()?;

        #[cfg(unix)]
        {
            let c_path = path_to_cstr(&path)?;
            LibrawError::check(unsafe {
                sys::libraw_open_file(self.inner.as_ptr(), c_path.as_ptr())
            })
        }

        #[cfg(windows)]
        {
            let c_path = path_to_widestring(&path)?;
            LibrawError::check(unsafe {
                sys::libraw_open_wfile(self.inner.as_ptr(), c_path.as_ptr())
            })
        }
    }

    #[cfg(windows)]
    pub fn open_fallback(&mut self, path: impl AsRef<Path>) -> Result<(), LibrawError> {
        if !path.as_ref().exists() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Raw file not found").into(),
            );
        }
        let c_path = path_to_cstr(&path)?;
        LibrawError::check(unsafe { sys::libraw_open_file(self.inner.as_ptr(), c_path.as_ptr()) })
    }

    pub fn open_buffer(&mut self, buffer: impl AsRef<[u8]>) -> Result<(), LibrawError> {
        self.recycle()?;
        let buffer = buffer.as_ref();
        LibrawError::check(unsafe {
            sys::libraw_open_buffer(
                self.inner.as_ptr(),
                buffer.as_ptr() as *const libc::c_void,
                buffer.len(),
            )
        })
    }

    /// Get the shootinginfo struct from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn shootinginfo(&'_ self) -> &'_ sys::libraw_shootinginfo_t {
        unsafe { &self.inner.as_ref().shootinginfo }
    }

    /// Get the idata struct from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn idata(&'_ self) -> &'_ sys::libraw_iparams_t {
        unsafe { &self.inner.as_ref().idata }
    }

    /// Get the sizes struct from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn sizes(&'_ self) -> &'_ sys::libraw_image_sizes_t {
        unsafe { &self.inner.as_ref().sizes }
    }

    /// Get the iparams from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn iparams(&'_ self) -> &'_ sys::libraw_iparams_t {
        let iparams = unsafe { sys::libraw_get_iparams(self.inner.as_ptr()) };
        assert!(!iparams.is_null());
        unsafe { &*iparams }
    }

    /// Get the lensinfo from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn lensinfo(&'_ self) -> &'_ sys::libraw_lensinfo_t {
        let lensinfo = unsafe { sys::libraw_get_lensinfo(self.inner.as_ptr()) };
        assert!(!lensinfo.is_null());
        unsafe { &*lensinfo }
    }

    /// Get the lensinfo from libraw_data_t
    ///
    /// Saftey:
    /// Dereferences a raw pointer
    pub fn makernotes(&'_ self) -> &'_ sys::libraw_makernotes_t {
        unsafe { &self.inner.as_ref().makernotes }
    }

    /// Get the xmpdata from the raw file
    pub fn xmpdata(&'_ self) -> Result<&'_ [u8], LibrawError> {
        let iparams = self.iparams();
        if iparams.xmplen == 0 || iparams.xmpdata.is_null() {
            return Err(LibrawError::XMPMissing);
        }
        let xmp = unsafe {
            std::slice::from_raw_parts(
                std::mem::transmute(iparams.xmpdata),
                iparams.xmplen as usize,
            )
        };
        Ok(xmp)
    }

    /// Get imgother by calling libraw_get_imgother
    pub fn imgother(&'_ self) -> &'_ sys::libraw_imgother_t {
        let imgother = unsafe { sys::libraw_get_imgother(self.inner.as_ptr()) };
        assert!(!imgother.is_null());
        unsafe { &*imgother }
    }

    /// Get the thumbnail struct from libraw_data_t
    pub fn thumbnail(&'_ self) -> &'_ sys::libraw_thumbnail_t {
        unsafe { &self.inner.as_ref().thumbnail }
    }

    /// Get the output parameters
    pub fn params(&'_ mut self) -> &'_ mut sys::libraw_output_params_t {
        unsafe { &mut self.inner.as_mut().params }
    }

    /// Get the colordata
    pub fn color(&'_ self) -> &'_ sys::libraw_colordata_t {
        unsafe { &self.inner.as_ref().color }
    }

    /// Unpack the thumbnail for the file
    pub fn unpack_thumb(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack_thumb(self.inner.as_ptr()) })?;
        Ok(())
    }

    /// Unpack the raw data and read it to memory
    pub fn unpack(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack(self.inner.as_ptr()) })?;
        Ok(())
    }

    /// Get the maximum colors
    pub fn get_color_maximum(&self) -> Result<i32, LibrawError> {
        let data = unsafe { sys::libraw_get_color_maximum(self.inner.as_ptr()) };
        Ok(data)
    }

    /// All other references should be invalid when we recycle so we take a mutable value to self
    pub fn recycle(&mut self) -> Result<(), LibrawError> {
        unsafe { sys::libraw_recycle(self.inner.as_ptr()) };
        Ok(())
    }

    /// Adjusts sizes and changes the resolution according to the flip values
    ///
    /// Also considers 45 degree angles for fuji cameras
    pub fn adjust_sizes_info_only(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_adjust_sizes_info_only(self.inner.as_ptr()) })
    }
}

#[cfg(feature = "jpeg")]
impl Processor {
    /// Returns a jpeg thumbnail
    /// resolution: Option<(width, height)>
    /// This will not generate a thumbnail if it is not present
    /// By default libraw rotates the thumbnail so that the image has correct orientation
    /// So no need for doing flips
    /// Consider ~20ms
    pub fn get_jpeg(&mut self) -> Result<Vec<u8>, LibrawError> {
        // First check if unpack_thumb has already been called.
        // If yes then don't call it

        // Check if (*inner).thumbnail.thumb is null
        if unsafe { (*self.inner.as_ptr()).thumbnail.thumb.is_null() } {
            self.unpack_thumb()?;
        }
        let flip = self.sizes().flip;
        let thumbnail = self.thumbnail();
        let thumbnail_data = unsafe {
            std::slice::from_raw_parts(thumbnail.thumb as *const u8, thumbnail.tlength as usize)
        };

        match ThumbnailFormat::from(thumbnail.tformat) {
            ThumbnailFormat::Jpeg => {
                // Since the buffer is already a jpeg buffer return it as-is
                //
                // Don't use a Vec since a Vec's internal memory representation is entirely dependent
                // on the allocator used which might(is) be different in c/c++/rust
                let jpeg = thumbnail_data.to_vec();
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
            ThumbnailFormat::Bitmap => {
                // Since this is a bitmap we have to generate the thumbnail from the rgb data from
                // here
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new(&mut jpeg).encode(
                    thumbnail_data,
                    thumbnail.twidth as u32,
                    thumbnail.theight as u32,
                    image::ColorType::Rgb8,
                )?;
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
            _ => Err(LibrawError::UnsupportedThumbnail),
        }
    }

    /// Get the jpeg without rotation
    pub fn get_jpeg_no_rotation(&mut self) -> Result<Vec<u8>, LibrawError> {
        // First check if unpack_thumb has already been called.
        // If yes then don't call it

        // Check if (*inner).thumbnail.thumb is null
        if unsafe { self.inner.as_ref().thumbnail.thumb.is_null() } {
            self.unpack_thumb()?;
        }
        let thumbnail = self.thumbnail();
        let thumbnail_data = unsafe {
            std::slice::from_raw_parts(thumbnail.thumb as *const u8, thumbnail.tlength as usize)
        };

        match ThumbnailFormat::from(thumbnail.tformat) {
            ThumbnailFormat::Jpeg => {
                // Since the buffer is already a jpeg buffer return it as-is
                //
                // Don't use a Vec since a Vec's internal memory representation is entirely dependent
                // on the allocator used which might(is) be different in c/c++/rust
                let jpeg = thumbnail_data.to_vec();
                Ok(jpeg)
            }
            ThumbnailFormat::Bitmap => {
                // Since this is a bitmap we have to generate the thumbnail from the rgb data from
                // here
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new(&mut jpeg).encode(
                    thumbnail_data,
                    thumbnail.twidth as u32,
                    thumbnail.theight as u32,
                    image::ColorType::Rgb8,
                )?;
                Ok(jpeg)
            }
            _ => Err(LibrawError::UnsupportedThumbnail),
        }
    }

    /// This will generate a thumbnail from the raw buffer
    /// It is **slower** than jpeg_thumb since it will unpack the rgb data
    ///
    /// resize_jpeg if it is true and the underlying data is a jpeg file then it will be resized to
    /// match the provided resolution
    /// Consider ~100ms
    pub fn to_jpeg(&mut self, quality: u8) -> Result<Vec<u8>, LibrawError> {
        // Since this image is possibly has a flip

        // Now check if libraw_unpack has been called already
        // If it has been call inner.image shouldn't be null
        if unsafe { self.inner.as_ref().image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.raw();

        // let data = unsafe {
        //     std::slice::from_raw_parts(
        //         processed.data.as_ptr() as *const u8,
        //         processed.data_size as usize,
        //     )
        // };

        match ImageFormat::from(processed.type_) {
            ImageFormat::Bitmap => {
                let colortype = match processed.bits {
                    8 => image::ColorType::Rgb8,
                    16 => image::ColorType::Rgb16,
                    _ => return Err(LibrawError::InvalidColor(processed.bits)),
                };
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode(
                    _processed.as_slice(),
                    processed.width as u32,
                    processed.height as u32,
                    colortype,
                )?;
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
            ImageFormat::Jpeg => {
                // structure contain in-memory image of JPEG file. Only type, data_size and data fields are valid (and nonzero);
                let jpeg = _processed.as_slice().to_vec();
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
        }
    }

    /// Get the original without any rotation
    pub fn to_jpeg_no_rotation(&mut self, quality: u8) -> Result<Vec<u8>, LibrawError> {
        // Since this image is possibly has a flip

        // Now check if libraw_unpack has been called already
        // If it has been call inner.image shouldn't be null
        if unsafe { self.inner.as_ref().image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.raw();

        // let data = unsafe {
        //     std::slice::from_raw_parts(
        //         processed.data.as_ptr() as *const u8,
        //         processed.data_size as usize,
        //     )
        // };

        match ImageFormat::from(processed.type_) {
            ImageFormat::Bitmap => {
                let colortype = match processed.bits {
                    8 => image::ColorType::Rgb8,
                    16 => image::ColorType::Rgb16,
                    _ => return Err(LibrawError::InvalidColor(processed.bits)),
                };
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode(
                    _processed.as_slice(),
                    processed.width as u32,
                    processed.height as u32,
                    colortype,
                )?;
                Ok(jpeg)
            }
            ImageFormat::Jpeg => {
                // structure contain in-memory image of JPEG file. Only type, data_size and data fields are valid (and nonzero);
                let jpeg = _processed.as_slice().to_vec();
                Ok(jpeg)
            }
        }
    }

    /// Same as to_jpeg but with resize to resolution
    /// This will be even slower than to_jpeg since it also has to resize
    /// Consider ~200ms
    pub fn to_jpeg_with_resolution(
        &mut self,
        resolution: impl IntoResolution,
        resize_jpeg: bool,
        quality: u8,
    ) -> Result<Vec<u8>, LibrawError> {
        // Now check if libraw_unpack has been called already
        // If it has been call inner.image shouldn't be null
        if unsafe { self.inner.as_ref().image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.raw();

        // let data = unsafe {
        //     std::slice::from_raw_parts(
        //         processed.data.as_ptr() as *const u8,
        //         processed.data_size as usize,
        //     )
        // };
        let res = resolution.into_resolution();
        match ImageFormat::from(processed.type_) {
            ImageFormat::Bitmap => {
                let mut jpeg = std::io::Cursor::new(Vec::new());
                let dynimg = match processed.bits {
                    8 => image::DynamicImage::from(
                        image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
                            processed.width.into(),
                            processed.height.into(),
                            _processed.as_slice().to_vec(),
                        )
                        .ok_or(LibrawError::EncodingError)?,
                    ),
                    16 => image::DynamicImage::from(
                        image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(
                            processed.width.into(),
                            processed.height.into(),
                            _processed.as_slice().to_vec(),
                        )
                        .ok_or(LibrawError::EncodingError)?,
                    ),
                    _ => return Err(LibrawError::InvalidColor(processed.bits)),
                };
                dynimg.write_to(&mut jpeg, image::ImageOutputFormat::Jpeg(quality))?;
                let jpeg = jpeg.into_inner();
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
            ImageFormat::Jpeg => {
                // structure contain in-memory image of JPEG file. Only type, data_size and data fields are valid (and nonzero);
                let mut jpeg = _processed.as_slice().to_vec();
                if resize_jpeg {
                    use image::io::Reader;
                    use std::io::Cursor;
                    let dynimg = Reader::new(&mut Cursor::new(jpeg.drain(..)))
                        .with_guessed_format()?
                        .decode()?
                        .thumbnail(res.width, res.height);
                    dynimg.write_to(
                        &mut Cursor::new(&mut jpeg),
                        image::ImageOutputFormat::Jpeg(quality),
                    )?;
                }
                let jpeg = Orientation::from(Flip::from(flip)).add_to(jpeg)?;
                Ok(jpeg)
            }
        }
    }

    /// This will first try get_jpeg and then fallback to to_jpeg
    /// Might take from 5 ~ 500 ms depending on the image
    pub fn jpeg(&mut self, quality: u8) -> Result<Vec<u8>, LibrawError> {
        let jpg = self.get_jpeg();
        if jpg.is_ok() {
            jpg
        } else {
            self.to_jpeg_no_rotation(quality)
        }
    }

    /// This will first try get_jpeg and then fallback to to_jpeg but won't modify any exif data
    /// in it
    /// Might take from 5 ~ 500 ms depending on the image
    #[inline]
    pub fn jpeg_no_rotation(&mut self, quality: u8) -> Result<Vec<u8>, LibrawError> {
        let jpg = self.get_jpeg_no_rotation();
        if jpg.is_ok() {
            jpg
        } else {
            self.to_jpeg_no_rotation(quality)
        }
    }
    pub fn jpeg_min_size(&mut self, quality: u8, threshold: u32) -> Result<Vec<u8>, LibrawError> {
        let t = self.thumbnail();
        if u32::from(t.theight * t.twidth) > threshold && self.unpack_thumb().is_ok() {
            self.get_jpeg()
        } else {
            self.to_jpeg_no_rotation(quality)
        }
    }

    pub fn jpeg_thumb_or_else_post_process<F: Fn(&mut Self) -> bool>(
        &mut self,
        quality: u8,
        f: F,
    ) -> Result<Vec<u8>, LibrawError> {
        if f(self) {
            self.get_jpeg()
        } else {
            self.to_jpeg_no_rotation(quality)
        }
    }
}

/// The builder struct for Processor
pub struct ProcessorBuilder {
    inner: NonNull<sys::libraw_data_t>,
}

impl ProcessorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Processor {
        Processor {
            inner: self.inner,
            dropped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_params<P: IntoIterator<Item = Params>>(mut self, params: P) -> Self {
        let libraw_params = unsafe { &mut self.inner.as_mut().params };
        use Params::*;
        for param in params {
            match param {
                Greybox(v) => libraw_params.greybox = v,
                Cropbox(v) => libraw_params.cropbox = v,
                Aber(v) => libraw_params.aber = v,
                Gamm(v) => libraw_params.gamm = v,
                UserMul(v) => libraw_params.user_mul = v,
                Bright(v) => libraw_params.bright = v,
                Threshold(v) => libraw_params.threshold = v,
                HalfSize(v) => libraw_params.half_size = v as i32,
                FourColorRgb(v) => libraw_params.four_color_rgb = v,
                Highlight(v) => libraw_params.highlight = v,
                UseAutoWb(v) => libraw_params.use_auto_wb = v as i32,
                UseCameraWb(v) => libraw_params.use_camera_wb = v as i32,
                UseCameraMatrix(v) => libraw_params.use_camera_matrix = v as i32,
                OutputColor(v) => libraw_params.output_color = v,
                OutputBps(v) => libraw_params.output_bps = v,
                OutputTiff(v) => libraw_params.output_tiff = v,
                OutputFlags(v) => libraw_params.output_flags = v,
                UserFlip(v) => libraw_params.user_flip = v,
                UserQual(v) => libraw_params.user_qual = v,
                UserBlack(v) => libraw_params.user_black = v,
                UserCblack(v) => libraw_params.user_cblack = v,
                UserSat(v) => libraw_params.user_sat = v,
                MedPasses(v) => libraw_params.med_passes = v,
                AutoBrightThr(v) => libraw_params.auto_bright_thr = v,
                AdjustMaximumThr(v) => libraw_params.adjust_maximum_thr = v,
                NoAutoBright(v) => libraw_params.no_auto_bright = v,
                UseFujiRrotate(v) => libraw_params.use_fuji_rotate = v,
                GreenMatching(v) => libraw_params.green_matching = v,
                DcbIterations(v) => libraw_params.dcb_iterations = v,
                DcbEnhanceFl(v) => libraw_params.dcb_enhance_fl = v,
                FbddNoiserd(v) => libraw_params.fbdd_noiserd = v,
                ExpCorrec(v) => libraw_params.exp_correc = v,
                ExpShift(v) => libraw_params.exp_shift = v,
                ExpPreser(v) => libraw_params.exp_preser = v,
                NoAutoScale(v) => libraw_params.no_auto_scale = v,
                NoInterpolation(v) => libraw_params.no_interpolation = v,
            }
        }
        self
    }
}
impl Default for ProcessorBuilder {
    fn default() -> Self {
        let inner = unsafe { sys::libraw_init(LibrawConstructorFlags::None as u32) };
        assert!(!inner.is_null());
        Self {
            inner: NonNull::new(inner).expect("non null"),
        }
    }
}

#[derive(Debug)]
pub struct ProcessedImage {
    inner: NonNull<sys::libraw_processed_image_t>,
}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_dcraw_clear_mem(self.inner.as_ptr()) }
    }
}

// impl Deref for ProcessedImage {
//     type Target = *mut sys::libraw_processed_image_t;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl DerefMut for ProcessedImage {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl ProcessedImage {
    pub fn raw(&self) -> &sys::libraw_processed_image_t {
        unsafe { self.inner.as_ref() }
    }
    pub fn as_slice_u8(&self) -> &[u8] {
        self.as_slice::<u8>()
    }
    pub fn as_slice_u16(&self) -> &[u16] {
        self.as_slice::<u16>()
    }

    pub fn as_slice<T>(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.inner.as_ref().data.as_ptr() as *const T,
                self.inner.as_ref().data_size as usize / std::mem::size_of::<T>(),
            )
        }
    }
    pub fn width(&self) -> u32 {
        self.raw().width.into()
    }
    pub fn height(&self) -> u32 {
        self.raw().height.into()
    }
    pub fn type_(&self) -> ImageFormat {
        ImageFormat::from(self.raw().type_)
    }
    pub fn bits(&self) -> u16 {
        self.raw().bits
    }
    pub fn colors(&self) -> u16 {
        self.raw().colors
    }
    pub fn size(&self) -> usize {
        self.raw().data_size as usize
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Params {
    Greybox([u32; 4]),
    Cropbox([u32; 4]),
    Aber([f64; 4]),
    Gamm([f64; 6]),
    UserMul([f32; 4usize]),
    Bright(f32),
    Threshold(f32),
    HalfSize(bool),
    FourColorRgb(i32),
    Highlight(i32),
    UseAutoWb(bool),
    UseCameraWb(bool),
    UseCameraMatrix(bool),
    OutputColor(i32),
    // OutputProfile: *mut libc::c_char,
    // CameraProfile: *mut libc::c_char,
    // BadPixels: *mut libc::c_char,
    // DarkFrame: *mut libc::c_char,
    OutputBps(i32),
    OutputTiff(i32),
    OutputFlags(i32),
    UserFlip(i32),
    UserQual(i32),
    UserBlack(i32),
    UserCblack([i32; 4usize]),
    UserSat(i32),
    MedPasses(i32),
    AutoBrightThr(f32),
    AdjustMaximumThr(f32),
    NoAutoBright(i32),
    UseFujiRrotate(i32),
    GreenMatching(i32),
    DcbIterations(i32),
    DcbEnhanceFl(i32),
    FbddNoiserd(i32),
    ExpCorrec(i32),
    ExpShift(f32),
    ExpPreser(f32),
    NoAutoScale(i32),
    NoInterpolation(i32),
}

#[non_exhaustive]
#[cfg_attr(all(windows, target_env = "msvc"), repr(i32))]
#[cfg_attr(all(windows, target_env = "gnu"), repr(u32))]
#[cfg_attr(unix, repr(u32))]
pub enum LibrawConstructorFlags {
    None = sys::LibRaw_constructor_flags_LIBRAW_OPTIONS_NONE,
    // Depending on the version of libraw this is not generated
    NoMemErrCallBack = 1,
    // On some versions of libraw this is misspelled opions
    NoDataErrCallBack = sys::LibRaw_constructor_flags_LIBRAW_OPTIONS_NO_DATAERR_CALLBACK,
}

/// The thumbnail types that might be embedded inside a raw file
#[non_exhaustive]
#[cfg_attr(all(windows, target_env = "msvc"), repr(i32))]
#[cfg_attr(all(windows, target_env = "gnu"), repr(u32))]
#[cfg_attr(unix, repr(u32))]
pub enum ThumbnailFormat {
    Unknown = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_UNKNOWN,
    Jpeg = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG,
    Bitmap = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP,
    Bitmap16 = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP16,
    Layer = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_LAYER,
    Rollei = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_ROLLEI,
    H265 = sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_H265,
}

impl From<sys::LibRaw_thumbnail_formats> for ThumbnailFormat {
    fn from(tformat: sys::LibRaw_thumbnail_formats) -> Self {
        use ThumbnailFormat::*;
        match tformat {
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_UNKNOWN => Unknown,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG => Jpeg,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP => Bitmap,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP16 => Bitmap16,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_LAYER => Layer,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_ROLLEI => Rollei,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_H265 => H265,
            _ => Unknown,
        }
    }
}

/// The format the raw file might be encoded in
#[non_exhaustive]
#[cfg_attr(all(windows, target_env = "msvc"), repr(i32))]
#[cfg_attr(all(windows, target_env = "gnu"), repr(u32))]
#[cfg_attr(unix, repr(u32))]
pub enum ImageFormat {
    Jpeg = sys::LibRaw_image_formats_LIBRAW_IMAGE_JPEG,
    Bitmap = sys::LibRaw_image_formats_LIBRAW_IMAGE_BITMAP,
}

impl From<sys::LibRaw_image_formats> for ImageFormat {
    fn from(format: sys::LibRaw_image_formats) -> Self {
        use ImageFormat::*;
        match format {
            sys::LibRaw_image_formats_LIBRAW_IMAGE_JPEG => Jpeg,
            sys::LibRaw_image_formats_LIBRAW_IMAGE_BITMAP => Bitmap,
            _ => unimplemented!("Please use the correct bindings for this version of libraw"),
        }
    }
}

#[cfg(unix)]
fn path_to_cstr(path: impl AsRef<Path>) -> Result<CString, std::ffi::NulError> {
    use std::os::unix::ffi::OsStrExt;
    let path = path.as_ref().as_os_str().as_bytes();
    CString::new(path)
}
#[cfg(windows)]
fn path_to_cstr(path: impl AsRef<Path>) -> Result<CString, std::ffi::NulError> {
    let path = path.as_ref().display().to_string();
    let path = path.as_bytes();
    CString::new(path)
}

#[cfg(windows)]
fn path_to_widestring(
    path: impl AsRef<Path>,
) -> Result<widestring::U16CString, widestring::error::NulError<u16>> {
    let path = path.as_ref().as_os_str();
    Ok(widestring::U16CString::from_os_str(path)?)
}

/// Represents the resolution for an image
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub trait IntoResolution {
    fn into_resolution(self) -> Resolution;
}

impl IntoResolution for (u32, u32) {
    fn into_resolution(self) -> Resolution {
        Resolution {
            width: self.0,
            height: self.1,
        }
    }
}
impl IntoResolution for (u16, u16) {
    fn into_resolution(self) -> Resolution {
        Resolution {
            width: self.0 as u32,
            height: self.1 as u32,
        }
    }
}
impl IntoResolution for [u32; 2] {
    fn into_resolution(self) -> Resolution {
        Resolution {
            width: self[0],
            height: self[1],
        }
    }
}
impl IntoResolution for [u16; 2] {
    fn into_resolution(self) -> Resolution {
        Resolution {
            width: self[0] as u32,
            height: self[1] as u32,
        }
    }
}

/// exif::Tag::Orientation
/// Possible values 1,2,3,4,5,6,7,8
/// 2, 5, 7 and 4 are mirrored images and not implemented
#[derive(Debug, Eq, PartialEq)]
pub struct Orientation(pub u8);
impl PartialEq<u8> for Orientation {
    fn eq(&self, other: &u8) -> bool {
        &self.0 == other
    }
}
impl PartialEq<Orientation> for u8 {
    fn eq(&self, other: &Orientation) -> bool {
        self == &other.0
    }
}

impl std::ops::Add for Orientation {
    type Output = Self;
    fn add(self, rhs: Orientation) -> Self::Output {
        Self(match (self.0, rhs.0) {
            (1, o) => o,
            (o, 1) => o,

            (2, 2) => 1,
            (2, 3) => 4,
            (2, 4) => 3,
            (2, 5) => 6,
            (2, 6) => 5,
            (2, 7) => 8,
            (2, 8) => 7,

            (3, 2) => 4,
            (3, 3) => 1,
            (3, 4) => 2,
            (3, 5) => 7,
            (3, 6) => 8,
            (3, 7) => 5,
            (3, 8) => 6,

            (4, 2) => 3,
            (4, 3) => 2,
            (4, 4) => 1,
            (4, 5) => 8,
            (4, 6) => 7,
            (4, 7) => 6,
            (4, 8) => 5,

            (5, 2) => 8,
            (5, 3) => 7,
            (5, 4) => 6,
            (5, 5) => 1,
            (5, 6) => 4,
            (5, 7) => 3,
            (5, 8) => 2,

            (6, 2) => 7,
            (6, 3) => 8,
            (6, 4) => 5,
            (6, 5) => 2,
            (6, 6) => 3,
            (6, 7) => 4,
            (6, 8) => 1,

            (7, 2) => 6,
            (7, 3) => 5,
            (7, 4) => 8,
            (7, 5) => 3,
            (7, 6) => 2,
            (7, 7) => 1,
            (7, 8) => 4,

            (8, 2) => 5,
            (8, 3) => 6,
            (8, 4) => 7,
            (8, 5) => 4,
            (8, 6) => 1,
            (8, 7) => 2,
            (8, 8) => 3,

            (_, _) => 1,
        })
    }
}
impl std::ops::Neg for Orientation {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(match self.0 {
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 8,
            7 => 7,
            8 => 6,
            o => o,
        })
    }
}
impl Orientation {
    pub const NONE: Self = Self(1);
    pub const CW180: Self = Self(3);
    pub const CW90: Self = Self(6);
    pub const CW270: Self = Self(8);
    pub const CCW90: Self = Self(8);

    #[cfg(feature = "jpeg")]
    pub fn add_to(self, mut buffer: Vec<u8>) -> Result<Vec<u8>, LibrawError> {
        use img_parts::ImageEXIF;
        if self.0 > 8 {
            return Err(
                std::io::Error::new(std::io::ErrorKind::Other, "Flip greater than 8").into(),
            );
        }

        let mut jpeg =
            img_parts::jpeg::Jpeg::from_bytes(img_parts::Bytes::from_iter(buffer.drain(..)))?;
        Orientation::__remove_xmp(&mut jpeg);
        jpeg.set_exif(Some(Self::exif_data_with_orientation(self.0).into()));
        jpeg.encoder().write_to(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(feature = "jpeg")]
    fn __remove_xmp(jpeg: &mut img_parts::jpeg::Jpeg) {
        jpeg.segments_mut().retain(|segment| {
            !(segment.marker() == 0xe1 && segment.contents().starts_with(b"http://ns.adobe.com/"))
        });
    }

    /// This encodes the orientation into a raw exif container data
    #[cfg(feature = "jpeg")]
    fn exif_data_with_orientation(o: u8) -> Vec<u8> {
        vec![
            0x4d, 0x4d, 0x0, 0x2a, 0x0, 0x0, 0x0, 0x8, 0x0, 0x1, 0x1, 0x12, 0x0, 0x3, 0x0, 0x0,
            0x0, 0x1, 0x0, o, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ]
    }
}

/// libraw_data_t.sizes.flip
/// Possible values 0, 3, 5, 6
#[derive(Debug, Eq, PartialEq)]
pub struct Flip(pub i32);
impl Flip {
    pub const NONE: Self = Self(0);
    pub const CW180: Self = Self(3);
    pub const CW90: Self = Self(6);
    pub const CW270: Self = Self(5);
    pub const CCW90: Self = Self(5);
}

impl From<i32> for Flip {
    fn from(flip: i32) -> Self {
        Self(flip)
    }
}

impl From<Flip> for Orientation {
    fn from(flip: Flip) -> Self {
        match flip {
            Flip::NONE => Orientation::NONE,
            Flip::CW90 => Orientation::CW90,
            Flip::CW180 => Orientation::CW180,
            Flip::CW270 => Orientation::CW270,
            _ => Orientation::NONE,
        }
    }
}
