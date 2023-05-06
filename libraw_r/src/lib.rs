#[macro_use]
pub mod error;
pub mod data_type;
pub mod dcraw;
pub mod libread;
pub mod callback;
#[cfg(feature = "exif")]
pub mod exif;
#[cfg(feature = "jpeg")]
pub mod extra;
// pub mod orientation;
pub mod progress;
pub mod thumbnail;
pub mod traits;

pub use error::LibrawError;

extern crate alloc;
extern crate libraw_sys as sys;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;
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
#[repr(transparent)]
pub struct EmptyProcessor {
    inner: NonNull<sys::libraw_data_t>,
}

impl EmptyProcessor {
    /// Calls libraw_init with the any of the constructor flags
    /// # May panic if the pointer returned by libraw_init is null
    pub fn new(option: LibrawConstructorFlags) -> Self {
        Self::try_new(option).expect("Failed to initialize libraw processor")
    }

    pub fn try_new(option: LibrawConstructorFlags) -> Result<Self, LibrawError> {
        let inner = unsafe { sys::libraw_init(option as u32) };
        if let Some(inner) = NonNull::new(inner) {
            Ok(Self { inner })
        } else {
            Err(LibrawError::CustomError(
                "Got back null pointer from libraw_init(0)".into(),
            ))
        }
    }

    /// Drop the processor and get a handle to the inner type
    ///
    /// Processor also implements DerefMut so you can take that if you want
    ///
    /// # Safety
    /// This skips the drop calls so you can call libraw_close and libraw_free_image yourself
    pub unsafe fn into_inner(self) -> NonNull<sys::libraw_data_t> {
        let inner = self.inner;
        core::mem::forget(self);
        inner
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

    /// All other references should be invalid when we recycle so we take a mutable value to self
    pub fn recycle(&mut self) -> Result<(), LibrawError> {
        unsafe { sys::libraw_recycle(self.inner_mut().as_ptr()) }
        Ok(())
    }
}

/// You can pass the Processor to another thread since it doesn't use any thread_local values
unsafe impl Send for EmptyProcessor {}
/// You can pass the reference to Processor to another thread since it cannot open / close / drop
/// Without a mutable reference to Self
unsafe impl Sync for EmptyProcessor {}

impl Default for EmptyProcessor {
    /// Returns libraw_init(0)
    fn default() -> Self {
        Self::new(LibrawConstructorFlags::None)
    }
}

/// Since EmptyProcessor doesn't have a File / Buffer opened so we can skip the free_image call
impl Drop for EmptyProcessor {
    fn drop(&mut self) {
        unsafe {
            sys::libraw_close(self.inner.as_ptr());
        }
    }
}

pub struct Processor<'p, D: ?Sized = data_type::File<'p>> {
    inner: EmptyProcessor,
    _data: core::marker::PhantomData<&'p D>,
}

/// Since Processor has some file / buffer opened we also need to call libraw_free_image
impl<'p, T: ?Sized> Drop for Processor<'p, T> {
    fn drop(&mut self) {
        unsafe {
            sys::libraw_free_image(self.inner.inner.as_ptr());
        }
    }
}

impl<T> Processor<'_, T> {
    /// Return's the inner EmptyProcessor without calling any destructors
    /// # Safety
    /// This skips the drop calls so you can call libraw_close and libraw_free_image yourself
    pub unsafe fn into_inner(self) -> EmptyProcessor {
        let p = ManuallyDrop::new(self);
        core::ptr::read(&p.inner)
    }

    /// Build Processor with options and params
    pub fn builder() -> ProcessorBuilder {
        ProcessorBuilder::default()
    }

    /// Get the shootinginfo struct from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn shootinginfo(&'_ self) -> &'_ sys::libraw_shootinginfo_t {
        unsafe { &self.inner.inner.as_ref().shootinginfo }
    }

    /// Get the idata struct from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn idata(&'_ self) -> &'_ sys::libraw_iparams_t {
        unsafe { &self.inner.inner.as_ref().idata }
    }

    /// Get the sizes struct from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn sizes(&self) -> &sys::libraw_image_sizes_t {
        unsafe { &self.inner.inner.as_ref().sizes }
    }

    /// Get the iparams from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn iparams(&self) -> &sys::libraw_iparams_t {
        let iparams = unsafe { sys::libraw_get_iparams(self.inner.inner.as_ptr()) };
        assert!(!iparams.is_null());
        unsafe { &*iparams }
    }

    /// Get the lensinfo from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn lensinfo(&self) -> &sys::libraw_lensinfo_t {
        let lensinfo = unsafe { sys::libraw_get_lensinfo(self.inner.inner.as_ptr()) };
        assert!(!lensinfo.is_null());
        unsafe { &*lensinfo }
    }

    /// Get the lensinfo from libraw_data_t
    ///
    /// Safety:
    /// Dereferences a raw pointer
    pub fn makernotes(&self) -> &sys::libraw_makernotes_t {
        unsafe { &self.inner.inner.as_ref().makernotes }
    }

    /// Get the xmpdata from the raw file
    pub fn xmpdata(&self) -> Result<&[u8], LibrawError> {
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
    pub fn imgother(&self) -> &sys::libraw_imgother_t {
        let imgother = unsafe { sys::libraw_get_imgother(self.inner.inner.as_ptr()) };
        assert!(!imgother.is_null());
        unsafe { &*imgother }
    }

    /// Get the output parameters
    pub fn params(&mut self) -> &mut sys::libraw_output_params_t {
        unsafe { &mut self.inner.inner.as_mut().params }
    }

    /// Get the colordata
    pub fn color(&self) -> &sys::libraw_colordata_t {
        unsafe { &self.inner.inner.as_ref().color }
    }

    /// Unpack the raw data and read it to memory
    pub fn unpack(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack(self.inner.inner.as_ptr()) })?;
        Ok(())
    }

    /// Get the maximum colors
    pub fn get_color_maximum(&self) -> Result<i32, LibrawError> {
        let data = unsafe { sys::libraw_get_color_maximum(self.inner.inner.as_ptr()) };
        Ok(data)
    }

    pub fn recycle(self) -> Result<EmptyProcessor, LibrawError> {
        // let mut data = ManuallyDrop::new(self);
        // let data = data.into_inner();
        // data.recycle()?;
        // Ok(data)
        todo!()
    }

    /// Adjusts sizes and changes the resolution according to the flip values
    ///
    /// Also considers 45 degree angles for fuji cameras
    pub fn adjust_sizes_info_only(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_adjust_sizes_info_only(self.inner.inner.as_ptr()) })
    }
}

/// The builder struct for Processor
pub struct ProcessorBuilder {
    inner: NonNull<sys::libraw_data_t>,
}

impl ProcessorBuilder {
    pub fn try_new() -> Result<Self, LibrawError> {
        let inner = unsafe { sys::libraw_init(LibrawConstructorFlags::None as u32) };
        Ok(if let Some(inner) = NonNull::new(inner) {
            Self { inner }
        } else {
            Err(LibrawError::CustomError(
                "Unable to initialize libraw_data_t".into(),
            ))?
        })
    }
    pub fn new() -> Self {
        Self::try_new().expect("Unable to initialize libraw_data_t")
    }

    pub fn build(self) -> EmptyProcessor {
        EmptyProcessor { inner: self.inner }
    }

    pub fn open<'p, D: data_type::DataType>(
        self,
        data: impl Into<D>,
    ) -> Result<Processor<'p, D>, LibrawError> {
        let ep = self.build();
        ep.open(data)
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
        Self::new()
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
