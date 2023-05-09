#[non_exhaustive]
#[repr(u32)]
#[derive(Debug)]
pub enum ProgressStage {
    Start = sys::LibRaw_progress_LIBRAW_PROGRESS_START,
    Open = sys::LibRaw_progress_LIBRAW_PROGRESS_OPEN,
    Identify = sys::LibRaw_progress_LIBRAW_PROGRESS_IDENTIFY,
    SizeAdjust = sys::LibRaw_progress_LIBRAW_PROGRESS_SIZE_ADJUST,
    LoadRaw = sys::LibRaw_progress_LIBRAW_PROGRESS_LOAD_RAW,
    Raw2Image = sys::LibRaw_progress_LIBRAW_PROGRESS_RAW2_IMAGE,
    RemoveZeroes = sys::LibRaw_progress_LIBRAW_PROGRESS_REMOVE_ZEROES,
    BadPixels = sys::LibRaw_progress_LIBRAW_PROGRESS_BAD_PIXELS,
    DarkFrame = sys::LibRaw_progress_LIBRAW_PROGRESS_DARK_FRAME,
    FoveonInterpolate = sys::LibRaw_progress_LIBRAW_PROGRESS_FOVEON_INTERPOLATE,
    ScaleColors = sys::LibRaw_progress_LIBRAW_PROGRESS_SCALE_COLORS,
    PreInterpolate = sys::LibRaw_progress_LIBRAW_PROGRESS_PRE_INTERPOLATE,
    Interpolate = sys::LibRaw_progress_LIBRAW_PROGRESS_INTERPOLATE,
    MixGreen = sys::LibRaw_progress_LIBRAW_PROGRESS_MIX_GREEN,
    MedianFilter = sys::LibRaw_progress_LIBRAW_PROGRESS_MEDIAN_FILTER,
    Highlights = sys::LibRaw_progress_LIBRAW_PROGRESS_HIGHLIGHTS,
    FujiRotate = sys::LibRaw_progress_LIBRAW_PROGRESS_FUJI_ROTATE,
    Flip = sys::LibRaw_progress_LIBRAW_PROGRESS_FLIP,
    ApplyProfile = sys::LibRaw_progress_LIBRAW_PROGRESS_APPLY_PROFILE,
    ConvertRgb = sys::LibRaw_progress_LIBRAW_PROGRESS_CONVERT_RGB,
    Stretch = sys::LibRaw_progress_LIBRAW_PROGRESS_STRETCH,
    Stage20 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE20,
    Stage21 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE21,
    Stage22 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE22,
    Stage23 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE23,
    Stage24 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE24,
    Stage25 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE25,
    Stage26 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE26,
    Stage27 = sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE27,
    ThumbLoad = sys::LibRaw_progress_LIBRAW_PROGRESS_THUMB_LOAD,
    TReserved1 = sys::LibRaw_progress_LIBRAW_PROGRESS_TRESERVED1,
    TReserved2 = sys::LibRaw_progress_LIBRAW_PROGRESS_TRESERVED2,
    Unknown,
}

#[test]
fn typetest() {
    // Checking whether the LibRaw_progress is a u32
    let _: sys::LibRaw_progress = 0u32;
}

impl From<sys::LibRaw_progress> for ProgressStage {
    fn from(value: sys::LibRaw_progress) -> Self {
        match value {
            sys::LibRaw_progress_LIBRAW_PROGRESS_START => Self::Start,
            sys::LibRaw_progress_LIBRAW_PROGRESS_OPEN => Self::Open,
            sys::LibRaw_progress_LIBRAW_PROGRESS_IDENTIFY => Self::Identify,
            sys::LibRaw_progress_LIBRAW_PROGRESS_SIZE_ADJUST => Self::SizeAdjust,
            sys::LibRaw_progress_LIBRAW_PROGRESS_LOAD_RAW => Self::LoadRaw,
            sys::LibRaw_progress_LIBRAW_PROGRESS_RAW2_IMAGE => Self::Raw2Image,
            sys::LibRaw_progress_LIBRAW_PROGRESS_REMOVE_ZEROES => Self::RemoveZeroes,
            sys::LibRaw_progress_LIBRAW_PROGRESS_BAD_PIXELS => Self::BadPixels,
            sys::LibRaw_progress_LIBRAW_PROGRESS_DARK_FRAME => Self::DarkFrame,
            sys::LibRaw_progress_LIBRAW_PROGRESS_FOVEON_INTERPOLATE => Self::FoveonInterpolate,
            sys::LibRaw_progress_LIBRAW_PROGRESS_SCALE_COLORS => Self::ScaleColors,
            sys::LibRaw_progress_LIBRAW_PROGRESS_PRE_INTERPOLATE => Self::PreInterpolate,
            sys::LibRaw_progress_LIBRAW_PROGRESS_INTERPOLATE => Self::Interpolate,
            sys::LibRaw_progress_LIBRAW_PROGRESS_MIX_GREEN => Self::MixGreen,
            sys::LibRaw_progress_LIBRAW_PROGRESS_MEDIAN_FILTER => Self::MedianFilter,
            sys::LibRaw_progress_LIBRAW_PROGRESS_HIGHLIGHTS => Self::Highlights,
            sys::LibRaw_progress_LIBRAW_PROGRESS_FUJI_ROTATE => Self::FujiRotate,
            sys::LibRaw_progress_LIBRAW_PROGRESS_FLIP => Self::Flip,
            sys::LibRaw_progress_LIBRAW_PROGRESS_APPLY_PROFILE => Self::ApplyProfile,
            sys::LibRaw_progress_LIBRAW_PROGRESS_CONVERT_RGB => Self::ConvertRgb,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STRETCH => Self::Stretch,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE20 => Self::Stage20,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE21 => Self::Stage21,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE22 => Self::Stage22,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE23 => Self::Stage23,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE24 => Self::Stage24,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE25 => Self::Stage25,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE26 => Self::Stage26,
            sys::LibRaw_progress_LIBRAW_PROGRESS_STAGE27 => Self::Stage27,
            sys::LibRaw_progress_LIBRAW_PROGRESS_THUMB_LOAD => Self::ThumbLoad,
            sys::LibRaw_progress_LIBRAW_PROGRESS_TRESERVED1 => Self::TReserved1,
            sys::LibRaw_progress_LIBRAW_PROGRESS_TRESERVED2 => Self::TReserved2,
            _ => Self::Unknown,
        }
    }
}
