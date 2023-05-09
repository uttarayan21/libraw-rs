use crate::*;

impl<'p, T, DD, PD, ED> Processor<'p, T, DD, PD, ED> {
    // pub fn thumbnail_processor(self) -> ThumbProcessor<'p, T> {
    //     ThumbProcessor { inner: self }
    // }

    pub fn thumbs_list(&self) -> &sys::libraw_thumbnail_list_t {
        unsafe { &self.inner.inner.as_ref().thumbs_list }
    }

    /// Unpack the thumbnail for the file
    pub fn unpack_thumb(&mut self) -> Result<(), LibrawError> {
        LibrawError::check(unsafe { sys::libraw_unpack_thumb(self.inner.inner.as_ptr()) })?;
        Ok(())
    }

    pub fn unpack_thumb_ex(&mut self, index: libc::c_int) -> Result<(), LibrawError> {
        LibrawError::check(unsafe {
            sys::libraw_unpack_thumb_ex(self.inner.inner.as_ptr(), index)
        })?;
        Ok(())
    }

    /// Get the thumbnail struct from libraw_data_t
    pub fn thumbnail(&self) -> &sys::libraw_thumbnail_t {
        unsafe { &self.inner.inner.as_ref().thumbnail }
    }
}

/// The thumbnail types that might be embedded inside a raw file
#[non_exhaustive]
#[cfg_attr(all(windows, target_env = "msvc"), repr(i32))]
#[cfg_attr(all(windows, target_env = "gnu"), repr(u32))]
#[cfg_attr(unix, repr(u32))]
#[derive(Debug, Copy, Clone)]
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

#[non_exhaustive]
#[cfg_attr(all(windows, target_env = "msvc"), repr(i32))]
#[cfg_attr(all(windows, target_env = "gnu"), repr(u32))]
#[cfg_attr(unix, repr(u32))]
#[derive(Debug, Copy, Clone)]
pub enum InternalThumbnailFormat {
    Unknown = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_UNKNOWN,
    KodakThumb = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_THUMB,
    KodakYCbCr = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_YCBCR,
    KodakRgb = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_RGB,
    Jpeg = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_JPEG,
    Layer = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_LAYER,
    Rollei = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_ROLLEI,
    Ppm = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM,
    Ppm16 = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM16,
    X3f = sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_X3F,
}

impl From<sys::LibRaw_internal_thumbnail_formats> for InternalThumbnailFormat {
    fn from(tformat: sys::LibRaw_internal_thumbnail_formats) -> Self {
        use InternalThumbnailFormat::*;
        match tformat {
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_UNKNOWN => Unknown,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_THUMB => {
                KodakThumb
            }
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_YCBCR => {
                KodakYCbCr
            }
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_KODAK_RGB => KodakRgb,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_JPEG => Jpeg,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_LAYER => Layer,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_ROLLEI => Rollei,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM => Ppm,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_PPM16 => Ppm16,
            sys::LibRaw_internal_thumbnail_formats_LIBRAW_INTERNAL_THUMBNAIL_X3F => X3f,
            _ => Unknown,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Thumbnail<'data> {
    pub format: ThumbnailFormat,
    pub width: u16,
    pub height: u16,
    pub length: u32,
    pub colors: i32,
    pub data: &'data [u8],
}

impl<'data> core::fmt::Debug for Thumbnail<'data> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thumbnail")
            .field("format", &self.format)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("length", &self.length)
            .field("colors", &self.colors)
            .field("data", &format!("Array([..], len: {})", &self.data.len()))
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ThumbnailItem {
    pub format: InternalThumbnailFormat,
    pub width: u16,
    pub height: u16,
    pub flip: u16,
    pub length: u32,
    pub misc: u32,
    pub offset: i64,
}

impl From<&sys::libraw_thumbnail_item_t> for ThumbnailItem {
    fn from(item: &sys::libraw_thumbnail_item_t) -> Self {
        Self {
            format: From::from(item.tformat),
            width: item.twidth,
            height: item.theight,
            flip: item.tflip,
            length: item.tlength,
            misc: item.tmisc,
            offset: item.toffset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Thumblist {
    pub count: usize,
    pub list: smallvec::SmallVec<[ThumbnailItem; 8]>,
}

impl Thumblist {
    pub fn max(&self) -> Option<ThumbnailItem> {
        self.list[..self.count]
            .iter()
            .max_by_key(|item| item.length)
            .copied()
    }
}

impl From<&sys::libraw_thumbnail_list_t> for Thumblist {
    fn from(list: &sys::libraw_thumbnail_list_t) -> Self {
        let count = list.thumbcount as usize;
        let list: smallvec::SmallVec<[ThumbnailItem; 8]> =
            list.thumblist[..count].iter().map(From::from).collect();
        Self { count, list }
    }
}

// pub struct ThumbProcessor<'p, D> {
//     inner: Processor<'p, D>,
// }

// impl<'p, D> std::ops::Deref for ThumbProcessor<'p, D> {
//     type Target = Processor<'p, D>;
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<'p, D> std::ops::DerefMut for ThumbProcessor<'p, D> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl<'thumb> From<&'thumb sys::libraw_thumbnail_t> for Thumbnail<'thumb> {
    fn from(thumb: &'thumb sys::libraw_thumbnail_t) -> Self {
        let data =
            unsafe { std::slice::from_raw_parts(thumb.thumb.cast(), thumb.tlength as usize) };
        Self {
            format: thumb.tformat.into(),
            width: thumb.twidth,
            height: thumb.theight,
            length: thumb.tlength,
            colors: thumb.tcolors,
            data,
        }
    }
}

pub trait Thumb {
    type Error;
    fn count(&self) -> usize;
    fn list(&self) -> Thumblist;
    fn unpack(&mut self, index: impl Into<Option<usize>>) -> Result<(), Self::Error>;
    fn get(&self) -> Result<Thumbnail, Self::Error>;
}

impl<D, DD, PD, ED> Thumb for Processor<'_, D, DD, PD, ED> {
    type Error = LibrawError;
    fn count(&self) -> usize {
        (self.thumbs_list().thumbcount as usize).clamp(0, self.thumbs_list().thumblist.len())
    }

    fn list(&self) -> Thumblist {
        self.thumbs_list().into()
    }

    fn unpack(&mut self, index: impl Into<Option<usize>>) -> Result<(), Self::Error> {
        let mut index = index.into();
        let list = self.list();
        if list.list.is_empty() {
            return Err(LibrawError::MissingThumbnails);
        }

        if index.is_none() {
            index = list
                .list
                .iter()
                .enumerate()
                .max_by_key(|(_, item)| item.length)
                .map(|(index, _)| index);
        }
        if let Some(idx) = index {
            self.unpack_thumb_ex(idx as i32)
        } else {
            Err(LibrawError::MissingThumbnails)
        }
    }

    fn get(&self) -> Result<Thumbnail, Self::Error> {
        let thumb = self.thumbnail();
        if thumb.thumb.is_null() {
            return Err(LibrawError::UninitThumb);
        }
        Ok(thumb.into())
    }
}
