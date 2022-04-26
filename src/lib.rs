use libraw_sys as sys;
use std::ffi::CString;
use std::ops::{Deref, DerefMut, Drop};
use std::path::Path;

pub struct Processor {
    inner: *mut sys::libraw_data_t,
}

impl Deref for Processor {
    type Target = *mut sys::libraw_data_t;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Processor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for Processor {
    fn drop(&mut self) {
        unsafe {
            sys::libraw_free_image(self.inner);
            sys::libraw_close(self.inner);
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new(LibrawConstructorFlags::None)
    }
}

impl Processor {
    pub fn into_inner(self) -> *mut sys::libraw_data_t {
        self.inner
    }

    pub fn builder() -> ProcessorBuilder {
        ProcessorBuilder::default()
    }

    pub fn new(option: LibrawConstructorFlags) -> Self {
        let inner = unsafe { sys::libraw_init(option as u32) };
        assert!(!inner.is_null());
        Self { inner }
    }

    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<(), LibrawError> {
        if !path.as_ref().exists() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Raw file not found").into(),
            );
        }

        let c_path = path_to_cstr(&path)?;
        #[allow(clippy::let_and_return)]
        // let ret = LibrawError::check(unsafe { sys::libraw_open_file(self.inner, c_path.as_ptr()) });
        let ret = LibrawError::check_with_context(
            unsafe { sys::libraw_open_file(self.inner, c_path.as_ptr()) },
            &path,
        );
        // Windows only fallback to open_wfile
        #[cfg(windows)]
        {
            if ret.is_err() {
                warn!("Failed to open file using libraw_open_file in windows");
                warn!("Fallback to open_wfile");
                let wchar_path = path_to_widestring(&path)?;
                return LibrawError::check_with_context(
                    unsafe { sys::libraw_open_wfile(self.inner, wchar_path.as_ptr()) },
                    &path,
                );
            }
        }
        ret
    }

    #[cfg(windows)]
    pub fn open_fallback(&mut self, path: impl AsRef<Path>) -> Result<(), LibrawError> {
        if !path.as_ref().exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Raw file not found",
            ))?;
        }
        let c_path = path_to_cstr(&path)?;
        LibrawError::check_with_context(
            unsafe { sys::libraw_open_file(self.inner, c_path.as_ptr()) },
            &path,
        )
    }

    pub fn shootinginfo(&'_ self) -> &'_ sys::libraw_shootinginfo_t {
        unsafe { &(*(self.inner)).shootinginfo }
    }
    pub fn sizes(&'_ self) -> &'_ sys::libraw_image_sizes_t {
        unsafe { &(*(self.inner)).sizes }
    }
    pub fn iparams(&'_ self) -> &'_ sys::libraw_iparams_t {
        let iparams = unsafe { sys::libraw_get_iparams(self.inner) };
        assert!(!iparams.is_null());
        unsafe { &*iparams }
    }

    pub fn lensinfo(&'_ self) -> &'_ sys::libraw_lensinfo_t {
        let lensinfo = unsafe { sys::libraw_get_lensinfo(self.inner) };
        assert!(!lensinfo.is_null());
        unsafe { &*lensinfo }
    }

    pub fn imgother(&'_ self) -> &'_ sys::libraw_imgother_t {
        let imgother = unsafe { sys::libraw_get_imgother(self.inner) };
        assert!(!imgother.is_null());
        unsafe { &*imgother }
    }

    pub fn thumbnail(&'_ self) -> &'_ sys::libraw_thumbnail_t {
        unsafe { &(*self.inner).thumbnail }
    }

    pub fn params(&'_ mut self) -> &'_ mut sys::libraw_output_params_t {
        unsafe { &mut (*self.inner).params }
    }

    pub fn unpack_thumb(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack_thumb(self.inner) })?;
        Ok(())
    }

    pub fn unpack(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack(self.inner) })?;
        Ok(())
    }

    pub fn dcraw_process_make_mem_thumb(&mut self) -> Result<ProcessedImage, LibrawError> {
        let mut errc = 0;
        let data = unsafe { sys::libraw_dcraw_make_mem_thumb(self.inner, &mut errc) };
        assert!(!data.is_null());
        LibrawError::to_result(errc, ProcessedImage { inner: data })
    }

    pub fn dcraw_process(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_dcraw_process(self.inner) })?;
        Ok(())
    }

    pub fn dcraw_process_make_mem_image(&mut self) -> Result<ProcessedImage, LibrawError> {
        let mut errc = 0;
        let data = unsafe { sys::libraw_dcraw_make_mem_image(self.inner, &mut errc) };
        assert!(!data.is_null());
        LibrawError::to_result(errc, ProcessedImage { inner: data })
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
        if unsafe { (*self.inner).thumbnail.thumb.is_null() } {
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
                Ok(thumbnail_data.to_vec())
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
        let params = self.params();
        params.half_size = 1;
        params.use_camera_wb = 1;
        // Since this image is possibly has a flip

        // Now check if libraw_unpack has been called already
        // If it has been call inner.image shouldn't be null
        if unsafe { (*self.inner).image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let processed = self.dcraw_process_make_mem_image()?;
        let processed = processed.raw();

        let data = unsafe {
            std::slice::from_raw_parts(
                processed.data.as_ptr() as *const u8,
                processed.data_size as usize,
            )
        };

        match ImageFormat::from(processed.type_) {
            ImageFormat::Bitmap => {
                let colortype = match processed.bits {
                    8 => image::ColorType::Rgb8,
                    16 => image::ColorType::Rgb16,
                    _ => return Err(LibrawError::InvalidColor(processed.bits)),
                };
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode(
                    data,
                    processed.width as u32,
                    processed.height as u32,
                    colortype,
                )?;
                Orientation::from(Flip::from(flip)).add_to(&mut jpeg)?;
                Ok(jpeg)
            }
            ImageFormat::Jpeg => {
                // structure contain in-memory image of JPEG file. Only type, data_size and data fields are valid (and nonzero);
                let mut jpeg = data.to_vec();
                Orientation::from(Flip::from(flip)).add_to(&mut jpeg)?;
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
        let params = self.params();
        params.half_size = 1;
        params.use_camera_wb = 1;

        // Now check if libraw_unpack has been called already
        // If it has been call inner.image shouldn't be null
        if unsafe { (*self.inner).image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let processed = self.dcraw_process_make_mem_image()?;
        let processed = processed.raw();

        let data = unsafe {
            std::slice::from_raw_parts(
                processed.data.as_ptr() as *const u8,
                processed.data_size as usize,
            )
        };
        let res = resolution.into_resolution();
        match ImageFormat::from(processed.type_) {
            ImageFormat::Bitmap => {
                let colortype = match processed.bits {
                    8 => image::ColorType::Rgb8,
                    16 => image::ColorType::Rgb16,
                    _ => return Err(LibrawError::InvalidColor(processed.bits)),
                };
                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode(
                    data,
                    processed.width.into(),
                    processed.height.into(),
                    colortype,
                )?;
                Orientation::from(Flip::from(flip)).add_to(&mut jpeg)?;
                Ok(jpeg)
            }
            ImageFormat::Jpeg => {
                // structure contain in-memory image of JPEG file. Only type, data_size and data fields are valid (and nonzero);
                let mut jpeg = data.to_vec();
                if resize_jpeg {
                    use image::io::Reader;
                    use std::io::Cursor;
                    let dynimg = Reader::new(&mut Cursor::new(jpeg.drain(..)))
                        .with_guessed_format()?
                        .decode()?
                        .thumbnail(res.width, res.height);
                    dynimg.write_to(
                        &mut Cursor::new(&mut jpeg),
                        image::ImageOutputFormat::Jpeg(80),
                    )?;
                }
                Orientation::from(Flip::from(flip)).add_to(&mut jpeg)?;
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
            self.to_jpeg(quality)
        }
    }
}

pub struct ProcessorBuilder {
    inner: *mut sys::libraw_data_t,
}

impl ProcessorBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> Processor {
        Processor { inner: self.inner }
    }
    pub fn with_params(self, params: Vec<Params>) -> Self {
        let libraw_params = unsafe { &mut (*self.inner).params };
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
                HalfSize(v) => libraw_params.half_size = v,
                FourColorRgb(v) => libraw_params.four_color_rgb = v,
                Highlight(v) => libraw_params.highlight = v,
                UseAutoWb(v) => libraw_params.use_auto_wb = v,
                UseCameraWb(v) => libraw_params.use_camera_wb = v,
                UseCameraMatrix(v) => libraw_params.use_camera_matrix = v,
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
        Self { inner }
    }
}

#[derive(Debug)]
pub struct ProcessedImage {
    inner: *mut sys::libraw_processed_image_t,
}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_dcraw_clear_mem(self.inner) }
    }
}

impl Deref for ProcessedImage {
    type Target = *mut sys::libraw_processed_image_t;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ProcessedImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ProcessedImage {
    pub fn raw(&self) -> &sys::libraw_processed_image_t {
        unsafe { &*self.inner }
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
    HalfSize(i32),
    FourColorRgb(i32),
    Highlight(i32),
    UseAutoWb(i32),
    UseCameraWb(i32),
    UseCameraMatrix(i32),
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
#[repr(u32)]
pub enum LibrawConstructorFlags {
    None = sys::LibRaw_constructor_flags_LIBRAW_OPTIONS_NONE,
    // Depending on the version of libraw this is not generated
    NoMemErrCallBack = 1,
    // On some versions of libraw this is misspelled opions
    NoDataErrCallBack = sys::LibRaw_constructor_flags_LIBRAW_OPTIONS_NO_DATAERR_CALLBACK,
}

// #[non_exhaustive]
// #[repr(u32)]
// #[allow(dead_code)]
// pub enum InternalThumbnailFormat {
//     Unknown = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_UNKNOWN,
//     KodakThumb = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_THUMB,
//     KodakYcbcr = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_YCBCR,
//     KodakRgb = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_RGB,
//     Jpeg = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_JPEG,
//     Layer = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_LAYER,
//     Rollei = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_ROLLEI,
//     Ppm = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM,
//     PPM16 = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM16,
//     X3F = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_X3F,
// }

/// The thumbnail types that might be embedded inside a raw file
#[non_exhaustive]
#[repr(u32)]
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
    fn from(tformat: sys::LibRaw_internal_thumbnail_formats) -> Self {
        use ThumbnailFormat::*;
        match tformat {
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_UNKNOWN => Unknown,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG => Jpeg,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP => Bitmap,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP16 => Bitmap16,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_LAYER => Layer,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_ROLLEI => Rollei,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_H265 => H265,
            _ => unimplemented!("Please use the correct bindings for this version of libraw"), // Consider other numbers unknown for now
        }
    }
}

/// The format the raw file might be encoded in
#[non_exhaustive]
#[repr(u32)]
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
    use std::os::windows::ffi::OsStrExt;
    // let path = path.as_ref().display().to_string();
    // CString::new(path)
    let path = path.as_ref().as_os_str().as_bytes();
    CString::new(path)
}

#[cfg(windows)]
fn path_to_widestring(
    path: impl AsRef<Path>,
) -> Result<widestring::U16CString, widestring::NulError<u16>> {
    let path = path.as_ref().as_os_str();
    widestring::U16CString::from_os_str(path)
}

#[derive(Debug, thiserror::Error)]
pub enum LibrawError {
    #[error("{0}")]
    InternalError(InternalLibrawError, String),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    NulError(#[from] std::ffi::NulError),
    #[cfg(windows)]
    #[error("{0}")]
    WidestringError(#[from] widestring::NulError<u16>),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("Unsupported Thumbnail")]
    UnsupportedThumbnail,
    #[error("Invalid Number of bits ({0}) for colortype")]
    InvalidColor(u16),
    #[cfg(feature = "jpeg")]
    #[error("{0}")]
    ImgPartsError(#[from] img_parts::Error),
}

impl LibrawError {
    pub fn to_result<T>(code: i32, data: T) -> Result<T, Self> {
        Ok(InternalLibrawError::to_result(code, data)?)
    }

    pub fn check(code: i32) -> Result<(), Self> {
        Ok(InternalLibrawError::check(code)?)
    }

    pub fn check_with_context(code: i32, file: impl AsRef<Path>) -> Result<(), Self> {
        let e = InternalLibrawError::check(code);
        if let Err(e) = e {
            Err(Self::InternalError(e, file.as_ref().display().to_string()))
        } else {
            Ok(())
        }
    }
}

impl From<InternalLibrawError> for LibrawError {
    fn from(e: InternalLibrawError) -> Self {
        LibrawError::InternalError(e, String::new())
    }
}

/// Error Codes from LibRaw
///
/// Check https://www.libraw.org/docs/API-datastruct.html#LibRaw_errors for reference
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InternalLibrawError {
    UnspecifiedError = sys::LibRaw_errors_LIBRAW_UNSPECIFIED_ERROR,
    FileUnsupported = sys::LibRaw_errors_LIBRAW_FILE_UNSUPPORTED,
    RequestForNonexistentImage = sys::LibRaw_errors_LIBRAW_REQUEST_FOR_NONEXISTENT_IMAGE,
    OutOfOrderCall = sys::LibRaw_errors_LIBRAW_OUT_OF_ORDER_CALL,
    NoThumbnail = sys::LibRaw_errors_LIBRAW_NO_THUMBNAIL,
    UnsupportedThumbnail = sys::LibRaw_errors_LIBRAW_UNSUPPORTED_THUMBNAIL,
    InputClosed = sys::LibRaw_errors_LIBRAW_INPUT_CLOSED,
    NotImplemented = sys::LibRaw_errors_LIBRAW_NOT_IMPLEMENTED,
    UnsufficientMemory = sys::LibRaw_errors_LIBRAW_UNSUFFICIENT_MEMORY,
    DataError = sys::LibRaw_errors_LIBRAW_DATA_ERROR,
    IoError = sys::LibRaw_errors_LIBRAW_IO_ERROR,
    CancelledByCallback = sys::LibRaw_errors_LIBRAW_CANCELLED_BY_CALLBACK,
    BadCrop = sys::LibRaw_errors_LIBRAW_BAD_CROP,
    TooBig = sys::LibRaw_errors_LIBRAW_TOO_BIG,
    MempoolOverflow = sys::LibRaw_errors_LIBRAW_MEMPOOL_OVERFLOW,
}

impl From<std::io::Error> for InternalLibrawError {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

impl From<std::ffi::NulError> for InternalLibrawError {
    fn from(_: std::ffi::NulError) -> Self {
        Self::UnspecifiedError
    }
}

#[cfg(windows)]
impl From<widestring::NulError<u16>> for InternalLibrawError {
    fn from(_: widestring::NulError<u16>) -> Self {
        Self::UnspecifiedError
    }
}

impl std::fmt::Display for InternalLibrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = *self as i32;
        let message =
            unsafe { std::ffi::CStr::from_ptr(sys::libraw_strerror(code)) }.to_string_lossy();
        write!(f, "Error Code: {}, Error Message: {}", code, message)
    }
}

impl InternalLibrawError {
    pub const SUCCESS: i32 = sys::LibRaw_errors_LIBRAW_SUCCESS;
    pub fn is_fatal(&self) -> bool {
        (*self as i32) < -100000
    }
    #[inline]
    pub fn to_result<T>(code: i32, data: T) -> Result<T, Self> {
        if code == Self::SUCCESS {
            Ok(data)
        } else {
            Err(InternalLibrawError::from(code))
        }
    }
    pub fn is_ok(code: i32) -> bool {
        code == Self::SUCCESS
    }
    pub fn is_err(code: i32) -> bool {
        code != Self::SUCCESS
    }
    #[inline]
    pub fn check(code: i32) -> Result<(), Self> {
        if code == Self::SUCCESS {
            Ok(())
        } else {
            Err(Self::from(code))
        }
    }
}

impl From<i32> for InternalLibrawError {
    fn from(e: i32) -> Self {
        use InternalLibrawError::*;
        match e {
            // e if e == Success as i32 => Success,
            e if e == UnspecifiedError as i32 => UnspecifiedError,
            e if e == FileUnsupported as i32 => FileUnsupported,
            e if e == RequestForNonexistentImage as i32 => RequestForNonexistentImage,
            e if e == OutOfOrderCall as i32 => OutOfOrderCall,
            e if e == NoThumbnail as i32 => NoThumbnail,
            e if e == UnsupportedThumbnail as i32 => UnsupportedThumbnail,
            e if e == InputClosed as i32 => InputClosed,
            e if e == NotImplemented as i32 => NotImplemented,
            e if e == UnsufficientMemory as i32 => UnsufficientMemory,
            e if e == DataError as i32 => DataError,
            e if e == IoError as i32 => IoError,
            e if e == CancelledByCallback as i32 => CancelledByCallback,
            e if e == BadCrop as i32 => BadCrop,
            e if e == TooBig as i32 => TooBig,
            e if e == MempoolOverflow as i32 => MempoolOverflow,
            e if e == Self::SUCCESS => panic!("This call was a success"),
            _ => unreachable!("If the error is reached then libraw has added new error types"),
        }
    }
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

impl Orientation {
    pub const NONE: Self = Self(1);
    pub const CW180: Self = Self(3);
    pub const CW90: Self = Self(6);
    pub const CW270: Self = Self(8);
    pub const CCW90: Self = Self(8);

    #[cfg(feature = "jpeg")]
    pub fn add_to<B>(self, buffer: B) -> Result<(), LibrawError>
    where
        B: AsRef<[u8]> + std::io::Write,
    {
        use img_parts::ImageEXIF;
        if self.0 > 8 {
            return Err(
                std::io::Error::new(std::io::ErrorKind::Other, "Flip greater than 8").into(),
            );
        }

        let mut jpeg =
            img_parts::jpeg::Jpeg::from_bytes(img_parts::Bytes::copy_from_slice(buffer.as_ref()))?;
        // img_parts::jpeg::Jpeg::from_bytes(img_parts::Bytes::from_iter(buffer.drain(..)))?;
        jpeg.set_exif(Some(Self::exif_data_with_orientation(self.0).into()));
        jpeg.encoder().write_to(buffer)?;
        Ok(())
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
