use crate::{
    error::LibrawError, traits::LRString, Flip, ImageFormat, IntoResolution, Orientation,
    Processor, ThumbnailFormat,
};
use el_noches::histmatch;
use image::load;

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
        if unsafe { (*self.inner).image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.inner();

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
        if unsafe { (*self.inner).image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.inner();

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
                // let mut r: Vec<u8> = Vec::with_capacity(processed.data_size as usize / 3);
                // let mut g: Vec<u8> = Vec::with_capacity(processed.data_size as usize / 3);
                // let mut b: Vec<u8> = Vec::with_capacity(processed.data_size as usize / 3);
                let img_array = if self.iparams().make.as_ascii() == "Sony" {
                    let src_width = processed.width;
                    let src_height = processed.height;
                    let refrence_img = &self.get_jpeg()?;
                    let refrence_img_dyn =
                        load(std::io::Cursor::new(refrence_img), image::ImageFormat::Jpeg)?;

                    // Convert the DynamicImage to Vec<u8> of R, G, B.
                    let mut buffer = _processed.as_slice().to_vec();
                    let src_img_channels = histmatch::ImageChannels::new(
                        &mut buffer,
                        src_width as u32,
                        src_height as u32,
                    );
                    let ref_height = refrence_img_dyn.height();
                    let ref_width = refrence_img_dyn.width();
                    let b = match refrence_img_dyn {
                        image::DynamicImage::ImageRgb8(img) => img.into_raw(),
                        image::DynamicImage::ImageRgb16(_img) => unimplemented!(),
                        _ => {
                            return Err(LibrawError::InternalError(
                                crate::error::InternalLibrawError::FileUnsupported,
                                "[Richard Stallman]: What you are refering to as Linux is actually called GNU/Linux or as I've recently started calling it GNU + Linux".into(),
                            ))
                        }
                    };
                    let mut b = b.to_vec();
                    let ref_img_channels =
                        histmatch::ImageChannels::new(&mut b, ref_width, ref_height);
                    histmatch::match_histogram_rgb_array(src_img_channels, ref_img_channels);
                    buffer
                } else {
                    _processed.as_slice().to_vec()
                };

                let mut jpeg = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality).encode(
                    &img_array,
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
        if unsafe { (*self.inner).image.is_null() } {
            self.unpack()?;
        }
        self.dcraw_process()?;
        let flip = self.sizes().flip;
        let _processed = self.dcraw_process_make_mem_image()?;
        let processed = _processed.inner();

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

    pub fn thumbnail_or_processed<F: Fn(&mut Self) -> bool>(
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
