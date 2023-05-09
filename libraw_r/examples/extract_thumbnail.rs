use libraw_r::data_type::File;
use libraw_r::traits::LRString;
use libraw_r::*;

pub fn main() -> anyhow::Result<()> {
    // for arg in std::env::args().skip(1) {
    //     let p = ProcessorBuilder::new()
    //         .with_params([Params::HalfSize(true)])
    //         .build();
    //     let mut p: Processor<File<'_>> = p.open(&arg)?;
    //     println!(
    //         "Processing {arg} ({}, {})",
    //         p.idata().make.as_ascii(),
    //         p.idata().model.as_ascii(),
    //     );
    //     // p.unpack()?;
    //     p.unpack_thumb()?;
    //     let t = p.thumbnail();
    //     println!("{t:?}");
    // }
    Ok(())
}
