#[test]
pub fn check_progress() {
    use libraw_r::*;
    let mut p = Processor::default();
    let r = p
        .set_progress_callback(
            |_| {
                // println!("Progress");
                0
            },
            (),
        )
        .expect("Failed to set progress callback");
    p.open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/RAW_NIKON_D3X.NEF"
    ))
    .expect("Failed to open file");
    p.unpack().expect("Failed to unpack");
    r.cancel();
    let e = dbg!(p.dcraw_process().unwrap_err());
    assert_eq!(
        e.libraw_err_type().expect("Not InternalError"),
        libraw_r::error::InternalLibrawError::CancelledByCallback
    )
}
