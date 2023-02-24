use crate::*;

pub fn half_size() -> Processor {
    Processor::builder()
        .with_params([Params::HalfSize(true)])
        .build()
}
pub fn half_size_auto_wb() -> Processor {
    Processor::builder()
        .with_params([Params::HalfSize(true), Params::UseAutoWb(true)])
        .build()
}
pub fn half_size_camera_wb() -> Processor {
    Processor::builder()
        .with_params([Params::HalfSize(true), Params::UseCameraWb(true)])
        .build()
}
pub fn half_size_auto_camera_wb() -> Processor {
    Processor::builder()
        .with_params([
            Params::HalfSize(true),
            Params::UseCameraWb(true),
            Params::UseAutoWb(true),
        ])
        .build()
}
pub fn auto_camera_wb() -> Processor {
    Processor::builder()
        .with_params([Params::UseCameraWb(true), Params::UseAutoWb(true)])
        .build()
}
