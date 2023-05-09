#[test]
fn exiftest() {
    use libraw_r::data_type::File;
    use libraw_r::*;
    use std::collections::HashMap;
    let p: ProcessorBuilder = ProcessorBuilder::new();
    let p = p.build();
    dbg!("sus");
    let p = p.set_exif_parser_callback(
        |args| {
            dbg!("Hello");
            args.context.insert(args.tag.to_string(), "sus".into());
            dbg!("Hello");
        },
        HashMap::<String, String>::new(),
    );
    let p = p.open::<File>("assets/RAW_NIKON_D3X.NEF").unwrap();
    let mut p = p.recycle().unwrap();
    // dbg!(p.reset_exif_callback());
    // let mut exif = p
    //     .set_exif_callback(0, DataStreamType::File, |args| {
    //         *args.callback_data += 1;
    //         if *args.callback_data == 50 {
    //             return Err("test error".into());
    //         }
    //         Ok(())
    //     })
    //     .unwrap();
    // p.open(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/assets/RAW_NIKON_D3X.NEF"
    // ))
    // .unwrap();
    // p.unpack().unwrap();
    // assert_eq!(1, exif.errors().unwrap().len());
    // assert_eq!(92, exif.data().unwrap());
}
