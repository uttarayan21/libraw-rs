use crate::{sys, Path};

#[derive(Debug, thiserror::Error)]
pub enum LibrawError {
    #[error("{0} {1}")]
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
    #[cfg(feature = "jpeg")]
    #[error("Failed to encode the processed image into and rgb image")]
    EncodingError,
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
