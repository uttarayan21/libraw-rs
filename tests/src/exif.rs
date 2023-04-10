#[test]
fn exiftest() {
    use libraw_r::exif::DataStreamType;
    use libraw_r::*;
    let mut p = Processor::default();
    let mut exif = p
        .set_exif_callback(0, DataStreamType::File, |args| {
            *args.callback_data += 1;
            if *args.callback_data == 50 {
                return Err("test error".into());
            }
            Ok(())
        })
        .unwrap();
    p.open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/RAW_NIKON_D3X.NEF"
    ))
    .unwrap();
    p.unpack().unwrap();
    assert_eq!(1, exif.errors().unwrap().len());
    assert_eq!(92, exif.data().unwrap());
}
